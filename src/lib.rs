//! Statistic 代码统计 — OpenIDEA 插件。
//!
//! WASM 侧只读项目文件、在本机计算行数并把结果推给沙箱面板；界面由
//! `web/index.html` 渲染。不联网、不执行程序、不写项目文件。
//!
//! 状态放在 `host::storage`：插件实例会被 IDE 回收再重建，`init` 也未必
//! 先于其它视图消息到达，所以每个入口都先 `ensure_loaded()`。

mod prefs;
mod scan;

use openidea_plugin::{
    Error, Invocation, Plugin, StatusItem, host, register_plugin,
    views::{View, ViewEvent, Viewport, Views},
};
use prefs::{DEFAULT_EXCLUDES, Prefs};
use scan::{
    Acc, Report, count_lines, group_number, is_binary, language_of, relative_path, should_exclude,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// 与 plugin.toml 的 `[[contributes.views]] id` 一致。
const VIEW: &str = "statistic";
const KEY_STATE: &str = "jiusi.statistic:state";
/// 与 plugin.toml 的 `[[contributes.statusBar]] id` 一致。
const STATUS_ID: &str = "jiusi.statistic.lines";
/// 单次扫描最多处理的文件数，防止超大仓库卡住命令预算。
const MAX_FILES: u32 = 20_000;
/// 每处理这么多文件续一次长任务预算、检查一次取消。
const REPORT_EVERY: usize = 200;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct State {
    report: Option<Report>,
    group: String,
    sort: String,
    /// 面板里改过的排除目录；`None` 表示用设置面板的默认值。
    exclude: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            report: None,
            group: "language".to_string(),
            sort: "lines".to_string(),
            exclude: None,
        }
    }
}

impl State {
    fn normalize(mut self) -> Self {
        if self.group != "language" && self.group != "directory" {
            self.group = "language".to_string();
        }
        if !matches!(self.sort.as_str(), "lines" | "code" | "files" | "name") {
            self.sort = "lines".to_string();
        }
        self
    }
}

#[derive(Default)]
pub struct Statistic {
    state: State,
    /// 最近一次扫描的错误，展示在面板底部（不覆盖上次的统计结果）。
    error: Option<String>,
    busy: bool,
    loaded: bool,
}

impl Statistic {
    fn load(&mut self) {
        self.state = host::storage::get(KEY_STATE)
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str::<State>(&s).ok())
            .unwrap_or_default()
            .normalize();
        self.error = None;
        self.busy = false;
        self.loaded = true;
    }

    fn ensure_loaded(&mut self) {
        if !self.loaded {
            self.load();
        }
    }

    fn save(&self) {
        if let Ok(s) = serde_json::to_string(&self.state) {
            let _ = host::storage::set(KEY_STATE, &s);
        }
    }

    fn prefs(&self) -> Prefs {
        Prefs::load().with_override(self.state.exclude.as_deref())
    }

    /// 设置面板的默认值，或面板里覆盖后的值（回显给输入框）。
    fn effective_exclude(&self) -> String {
        self.state.exclude.clone().unwrap_or_else(|| {
            host::setting_string("excludeDirs").unwrap_or_else(|| DEFAULT_EXCLUDES.to_string())
        })
    }

    fn snapshot(&self) -> Value {
        json!({
            "type": "snapshot",
            "report": serde_json::to_value(&self.state.report).unwrap_or(Value::Null),
            "group": self.state.group,
            "sort": self.state.sort,
            "exclude": self.effective_exclude(),
            "busy": self.busy,
            "error": self.error.clone(),
            "maxFiles": MAX_FILES,
        })
    }

    fn push(&self) {
        let _ = host::post_view_message(VIEW, &self.snapshot().to_string());
    }

    fn update_status(&self) {
        if !self.prefs().show_status_bar {
            host::remove_status_item(STATUS_ID);
            return;
        }
        let text = match &self.state.report {
            Some(r) => format!("{} 行", group_number(r.lines.total)),
            None => "—".to_string(),
        };
        host::set_status_item(
            STATUS_ID,
            &StatusItem {
                text,
                tooltip: Some("点击重新统计项目代码行数".to_string()),
                command: Some("refresh".to_string()),
            },
        );
    }

    /// 扫描第一个打开的项目。`Ok(None)` 表示用户取消了，保留上次结果。
    fn run_scan(&self, prefs: &Prefs) -> Result<Option<Report>, String> {
        let projects = host::projects().map_err(friendly)?;
        let project = projects
            .first()
            .ok_or_else(|| "当前没有打开的项目，请先打开一个项目再统计".to_string())?;
        let root = project.root().map_err(friendly)?;
        let files = project.find_files("**/*", MAX_FILES).map_err(friendly)?;

        let task = host::begin_task("Statistic: 正在统计代码行数", true);
        let total = files.len();
        let mut acc = Acc::default();

        for (i, path) in files.iter().enumerate() {
            if i % REPORT_EVERY == 0 {
                task.report(i as u32, Some(total as u32), Some("正在读取项目文件"));
                if task.is_cancelled() {
                    return Ok(None);
                }
            }
            let rel = relative_path(&root, path);
            if should_exclude(&rel, &prefs.excludes, prefs.include_hidden) {
                continue;
            }
            let text = match project.read_file(path) {
                Ok(text) => text,
                Err(_) => {
                    acc.skip();
                    continue;
                }
            };
            if is_binary(&text) || text.len() > prefs.max_file_bytes {
                acc.skip();
                continue;
            }
            let spec = language_of(&rel);
            let lines = count_lines(&text, spec);
            acc.add_file(&rel, spec.name, text.len(), lines);
        }

        task.report(total as u32, Some(total as u32), Some("统计完成"));
        let normalized = root.replace('\\', "/");
        let name = normalized
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("");
        let label = if name.is_empty() { "项目" } else { name };
        Ok(Some(acc.finish(label, files.len() >= MAX_FILES as usize)))
    }

    fn scan(&mut self) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.push();
        let prefs = self.prefs();
        let result = self.run_scan(&prefs);
        self.busy = false;
        match result {
            Ok(Some(report)) => {
                self.state.report = Some(report);
                self.error = None;
            }
            // 用户取消：保留上一次的结果，也不显示错误。
            Ok(None) => self.error = None,
            Err(message) => self.error = Some(message),
        }
        self.save();
        self.update_status();
        self.push();
    }

    fn on_message(&mut self, text: &str) {
        let msg: Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => return,
        };
        let kind = msg.get("type").and_then(Value::as_str).unwrap_or("");
        if kind != "init" {
            self.ensure_loaded();
        }
        match kind {
            "init" => {
                self.load();
                self.update_status();
                if self.state.report.is_none() {
                    self.scan();
                } else {
                    self.push();
                }
            }
            "refresh" => self.scan(),
            "setGroup" => {
                if let Some(v) = msg.get("value").and_then(Value::as_str)
                    && (v == "language" || v == "directory")
                {
                    self.state.group = v.to_string();
                    self.save();
                    self.push();
                }
            }
            "setSort" => {
                if let Some(v) = msg.get("value").and_then(Value::as_str)
                    && matches!(v, "lines" | "code" | "files" | "name")
                {
                    self.state.sort = v.to_string();
                    self.save();
                    self.push();
                }
            }
            "setExclude" => {
                if let Some(v) = msg.get("value").and_then(Value::as_str) {
                    self.state.exclude = Some(v.to_string());
                    self.save();
                    self.scan();
                }
            }
            "resetExclude" => {
                self.state.exclude = None;
                self.save();
                self.scan();
            }
            _ => {}
        }
    }
}

fn friendly(e: Error) -> String {
    match e {
        Error::Denied(_) => {
            "缺少 project.read 权限，请在 Settings › Plugins › Statistic 代码统计 里开启授权"
                .to_string()
        }
        Error::NotFound(_) => "项目已关闭，或文件不可读".to_string(),
        other => format!("统计失败：{other}"),
    }
}

impl Plugin for Statistic {
    fn run_command(
        &mut self,
        id: &str,
        _ctx: &Invocation,
        _arg: Option<&str>,
    ) -> Result<Option<String>, Error> {
        self.ensure_loaded();
        match id {
            "refresh" => {
                self.scan();
                Ok(None)
            }
            "switch-group" => {
                self.state.group = if self.state.group == "language" {
                    "directory"
                } else {
                    "language"
                }
                .to_string();
                self.save();
                self.push();
                Ok(None)
            }
            "clear-cache" => {
                self.state.report = None;
                self.save();
                self.scan();
                Ok(None)
            }
            other => Err(Error::NotFound(format!("command {other}"))),
        }
    }
}

impl Views for Statistic {
    fn render(
        &mut self,
        _view: &str,
        _viewport: Option<Viewport>,
        _ctx: &Invocation,
    ) -> Result<View, Error> {
        // 声明了 web view，真正的界面由 web/index.html 画。
        Ok(View::message("正在加载统计结果…"))
    }

    fn handle(&mut self, _view: &str, event: ViewEvent, _ctx: &Invocation) -> Result<(), Error> {
        if let ViewEvent::Message(text) = event {
            self.on_message(&text);
        }
        Ok(())
    }
}

register_plugin!(Statistic, views);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_saved_values_are_normalized() {
        let s = State {
            group: "nonsense".into(),
            sort: "nonsense".into(),
            ..State::default()
        }
        .normalize();
        assert_eq!(s.group, "language");
        assert_eq!(s.sort, "lines");
    }

    #[test]
    fn snapshot_shape_is_stable() {
        let _host = openidea_plugin::mock::MockHost::install();
        let plugin = Statistic {
            state: State::default(),
            loaded: true,
            ..Statistic::default()
        };
        let v = plugin.snapshot();
        assert_eq!(v["type"], "snapshot");
        assert!(v["report"].is_null());
        assert_eq!(v["group"], "language");
        assert_eq!(v["sort"], "lines");
        assert_eq!(v["busy"], false);
        assert_eq!(v["maxFiles"], MAX_FILES);
    }

    #[test]
    fn switching_group_is_persisted() {
        let _host = openidea_plugin::mock::MockHost::install();
        let mut plugin = Statistic {
            state: State::default(),
            loaded: true,
            ..Statistic::default()
        };
        plugin.on_message(r#"{"type":"setGroup","value":"directory"}"#);
        assert_eq!(plugin.state.group, "directory");
        let saved = host::storage::get(KEY_STATE).unwrap().unwrap();
        let back: State = serde_json::from_str(&saved).unwrap();
        assert_eq!(back.group, "directory");
    }

    #[test]
    fn unknown_view_messages_are_ignored() {
        let _host = openidea_plugin::mock::MockHost::install();
        let mut plugin = Statistic {
            state: State::default(),
            loaded: true,
            ..Statistic::default()
        };
        plugin.on_message("not json");
        plugin.on_message(r#"{"type":"unknown"}"#);
        assert_eq!(plugin.state.group, "language");
    }
}
