# Statistic 代码统计

按语言和目录统计当前项目的代码行数，作为 OpenIDEA 右侧工具窗口运行。只读项目文件，全部在本机计算；结果和面板偏好只保存在本机。

| | |
| --- | --- |
| 插件 id | `jiusi.statistic` |
| 作者 | 九思 |
| 许可 | Apache-2.0 |
| 插件 API | `^1.4` |
| 分类 | tools |

## 它能做什么

| 能力 | 说明 |
| --- | --- |
| 两种分组 | **按语言**（Rust / Python / TypeScript / Java / Markdown… 40+ 种映射）或**按一级目录**，顶部六张概览卡给出文件数、总行、代码、注释、空行、字节数 |
| 行分类 | 总行拆成代码行、注释行、空行；按语言的整行注释前缀与块注释符号识别，跨行块注释可正确处理 |
| 排序 | 按总行数 / 代码行 / 文件数 / 名称，表格里带相对占比条 |
| 面板内设置 | 排除目录（逗号分隔，失焦即重扫）、恢复默认、分组切换、排序切换、刷新 |
| 设置面板 | 排除目录默认值、单文件最大字节数（默认 2 MiB）、是否包含隐藏文件、是否显示状态栏 |
| 状态栏 | 常驻显示总行数，点击即刷新；可关闭 |
| 命令面板 | `Statistic: 重新统计`（`Alt+Shift+S`，macOS `Meta+Shift+S`）、`切换分组`、`清除缓存并重扫` |
| 大项目 | 扫描跑在可取消的长任务里，每 200 个文件报告一次进度；上限 20,000 个文件，达到上限会在面板里明确提示结果不完整 |

**明确不做**：不联网、不执行任何程序、不写项目文件、不上传任何统计结果、不做代码质量评价、不提供 Git 历史统计。

## 安装

已上架 OpenIDEA 社区插件市场，当前 0.1.1：

- **市场安装**（推荐）：`Settings › Plugins › Available` 搜索「Statistic」或「代码统计」。
- **从包安装**：`Settings › Plugins › Install from Disk`，选 `jiusi.statistic-0.1.1.oiplugin`。

首次打开「代码统计」面板会自动扫一次；之后优先显示本机缓存的上一份结果，点「刷新」或状态栏才会重扫。这样打开面板不会每次都卡在扫描上。

## 隐私与权限

插件只申请两项**低风险**能力，安装时会逐项列出：

| 能力 | 风险 | 用途 |
| --- | --- | --- |
| `project.read` | Low | 通过 `Project::find_files("**/*")` 列出项目文件、`Project::read_file` 读取文本内容、`Project::root` 计算相对路径。**只读**，不调用 `write_file`，不读项目外的路径 |
| `storage` | Low | 保存上一份统计结果与面板偏好（分组、排序、排除目录）。数据留在本机，卸载即删除 |

没有 `net:`、没有 `process:`、没有 `editor.write`、没有 `vcs.*`、没有 `fs.read:`。面板跑在沙箱 iframe 里，本身没有网络权限，也不申请剪贴板能力。

**统计口径**：`find_files` 会跳过 IDE 忽略/排除的文件（含 gitignore）；插件另按设置排除常见构建目录；含 NUL 的二进制文件和超过大小上限的文件会被跳过并在页脚显示跳过数量。行分类是启发式的，不解析字符串字面量——`"// not a comment"` 这类内容可能被算成注释行，它适合看项目规模，不适合当审计数据。多项目同时打开时，统计第一个项目，面板标题会显示项目名。

## 使用

1. 安装后在右侧工具窗口打开「代码统计」，首次会自动扫描。
2. 点「刷新」重新统计；`按语言 / 按目录` 切换分组，下拉框切换排序。
3. 「排除的目录」输入框里改完失焦即重扫；「恢复默认」回到设置面板的默认值。
4. 底部页脚会显示跳过的文件数、是否达到 20,000 文件上限、以及权限错误原因。

表格用固定列宽自适应面板宽度，过长的分组名会省略（悬停可看全名）；占比列显示「百分比数字 + 进度条」，百分比是该分组总行占项目总行的比例；面板极窄时才会出现一条细滚动条。

设置面板（`Settings › Plugins › Statistic 代码统计`）给的是**默认值**；面板里改过的排除目录写入本机并覆盖默认值，卸载插件时一并删除。

## 开发

需要 Rust 与 `wasm32-wasip2`：

```sh
rustup target add wasm32-wasip2
cargo test                                    # 纯逻辑 + mock host 测试
cargo build --release --target wasm32-wasip2
mkdir -p dist && cp target/wasm32-wasip2/release/openidea_plugin_statistic.wasm dist/plugin.wasm
```

Windows 上如果 MSVC 链接器被 Git Bash 的 `link` 抢占，改用 GNU 工具链：`cargo +stable-x86_64-pc-windows-gnu build --release --target wasm32-wasip2`。

### 打包与签名

```sh
python scripts/oiplugin.py check .
python scripts/oiplugin.py pack . --key .keys/jiusi.key --publisher jiusi \
  --out dist/jiusi.statistic-0.1.1.oiplugin
python scripts/oiplugin.py verify dist/jiusi.statistic-0.1.1.oiplugin
```

`jiusi` 这个 publisher 前缀与 registry 里的 `publishers/jiusi.toml` 一致，签名用的是同一把私钥；**不要重新生成密钥**，否则用户会看到签名者变更。

### 结构

```
plugin.toml            清单：能力、命令、快捷键、视图、设置、状态栏
settings.schema.json   4 项设置默认值
src/lib.rs             插件入口：扫描编排、快照、命令、视图消息、状态栏
src/scan.rs            纯逻辑：语言映射、行分类、路径/排除判断、聚合、单元测试
src/prefs.rs           设置读取与面板覆盖
web/index.html         沙箱面板：无外部资源，靠 postMessage 与 WASM 通信
icons/                 插件图标（明暗）与视图图标
scripts/               打包、签名、校验、本地安装状态
vendor/                SDK v0.1.40 源码快照（保证离线可构建）
dist/plugin.wasm       构建产物
```

界面在沙箱 iframe 里运行，**没有网络权限**，所有数据都由 WASM 侧推过去；页面只负责渲染与交互。直接用浏览器打开 `web/index.html` 会显示一组假数据，方便调样式。

## 上架

已上架 [OpenIDEA 社区插件市场](https://github.com/l0ng-ai/openidea-plugins)，条目为 `plugins/jiusi.statistic.toml`；条目上的 `suggest` 会让插件出现在含 `pom.xml` / `package.json` / `Cargo.toml` 等构建文件的项目下「Suggested for This Project」。

完整流程见 `docs/PUBLISHING.md`：先发 GitHub Release 并上传 `.oiplugin`，再把 `publish/plugins/jiusi.statistic.toml` 加进 `l0ng-ai/openidea-plugins` 并提 PR。CI 会下载附件核对 sha256、验签并跑 `openidea-plugin check`。

## 路线图

- **0.2.0**：圈复杂度 / 嵌套深度 / 超长方法、文件热度、按目录钻取、排除规则模板、把结果导出成表格文本。仍是 `project.read` + `storage` 两个 Low。
- **0.3.0**：把每次统计留成本地快照，做「本周 vs 上周」的趋势对比。
**明确不做**：不联网、不上传统计结果、不做 Git 历史统计、不做代码质量评分。行分类是启发式的，这个口径会一直保持。

## 交流

Java 技术交流 Q 群：700072697

## 许可

Apache-2.0。
