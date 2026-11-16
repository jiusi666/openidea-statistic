//! 设置读取：设置面板给默认值，面板里改过的排除目录覆盖它。

use openidea_plugin::host;

use crate::scan::Limits;

pub const DEFAULT_EXCLUDES: &str = ".git,.idea,.vscode,node_modules,target,dist,build,out,bin,obj,vendor,.venv,venv,__pycache__,coverage,.gradle,.next,.nuxt,.cache";

pub const DEFAULT_MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_BYTES_CEILING: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prefs {
    pub excludes: Vec<String>,
    pub max_file_bytes: usize,
    pub include_hidden: bool,
    pub show_status_bar: bool,
    /// 是否在项目树上给高风险文件加标记。
    pub decorate_tree: bool,
    /// 复杂度 / 超长阈值的分母，同时决定风险分。
    pub limits: Limits,
}

impl Prefs {
    pub fn load() -> Self {
        let raw =
            host::setting_string("excludeDirs").unwrap_or_else(|| DEFAULT_EXCLUDES.to_string());
        let max = host::setting_number("maxFileBytes")
            .map(|n| (n as usize).clamp(1024, MAX_BYTES_CEILING))
            .unwrap_or(DEFAULT_MAX_BYTES);
        let limits = Limits {
            file_lines: number("warnFileLines", 50.0, 200_000.0, 800.0) as u64,
            fn_lines: number("warnFunctionLines", 5.0, 5_000.0, 60.0) as u32,
            complexity: number("warnComplexity", 1.0, 500.0, 15.0) as u32,
        };
        Self {
            excludes: parse_excludes(&raw),
            max_file_bytes: max,
            include_hidden: host::setting_bool("includeHidden").unwrap_or(false),
            show_status_bar: host::setting_bool("showStatusBar").unwrap_or(true),
            decorate_tree: host::setting_bool("decorateTree").unwrap_or(true),
            limits,
        }
    }

    /// 面板里改过排除目录就用面板的值；`None` 表示回到设置面板的默认值。
    pub fn with_override(mut self, raw: Option<&str>) -> Self {
        if let Some(value) = raw {
            self.excludes = parse_excludes(value);
        }
        self
    }
}

/// 读一个带上下限的数字设置，越界就夹紧，读不到就用默认值。
fn number(key: &str, min: f64, max: f64, fallback: f64) -> f64 {
    host::setting_number(key)
        .map(|n| n.clamp(min, max))
        .unwrap_or(fallback)
}

pub fn parse_excludes(raw: &str) -> Vec<String> {
    raw.split([',', ';', '\n'])
        .map(|s| s.trim().trim_matches('/').to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_messy_exclude_lists() {
        assert_eq!(
            parse_excludes(" node_modules, /Target/ ; .git\n\nvendor"),
            vec!["node_modules", "target", ".git", "vendor"]
        );
        assert!(parse_excludes("  ").is_empty());
    }

    #[test]
    fn override_replaces_only_excludes() {
        let _host = openidea_plugin::mock::MockHost::install();
        let base = Prefs {
            excludes: vec!["a".into()],
            max_file_bytes: 1234,
            include_hidden: true,
            show_status_bar: false,
            decorate_tree: false,
            limits: Limits::default(),
        };
        let over = base.clone().with_override(Some("b, c"));
        assert_eq!(over.excludes, vec!["b", "c"]);
        assert_eq!(over.max_file_bytes, 1234);
        assert!(over.include_hidden);
        assert!(!over.show_status_bar);
        assert_eq!(base.clone().with_override(None).excludes, vec!["a"]);
    }
}
