# 上架流程（jiusi.statistic）

当前版本 `0.1.0` 已在本机完成构建、打包、签名与校验。

## 已验证的本机产物

| 项 | 值 |
| --- | --- |
| 包 | `dist/jiusi.statistic-0.1.0.oiplugin`（115 KB） |
| WASM | `dist/plugin.wasm`（280 KB，`wasm32-wasip2` 组件） |
| BLAKE3 摘要 | `2a29c7f44e22f188791c0a39b22759483e96829f91e60e07662bca8d00cf4ce6` |
| 包 sha256 | `3d5a1a01a8a839d8e1e9b495e82d80d315e468e296f5202440064929e0cb9c56` |

面板排版调整后重新打过包，上面是**当前**包的摘要；如果之前已经上传过 Release 附件，必须替换成这个新包，再更新 registry 条目里的 sha256。
| 签名者 | `jiusi`，公钥与官方 registry 的 `publishers/jiusi.toml` 一致 |

**私钥没有复制进本仓库**，打包时直接引用了 `openidea-stock-watch/.keys/jiusi.key`。
如果要单独维护这个仓库，把同一把私钥放到本项目的 `.keys/jiusi.key`（已被
`.gitignore` 忽略），不要重新生成——换钥会让已安装的用户看到签名者变更。

## 本机构建命令（Windows / Git Bash）

默认的 MSVC 工具链在这台机器上抢到了 Git Bash 的 `link`，且没有装 VS C++
链接器，所以用 GNU 工具链：

```sh
P="F:/code/openidea-plugins/openidea-statistic"
cargo +stable-x86_64-pc-windows-gnu test --manifest-path "$P/Cargo.toml"
cargo +stable-x86_64-pc-windows-gnu build --release --target wasm32-wasip2 --manifest-path "$P/Cargo.toml"
cp "$P/target/wasm32-wasip2/release/openidea_plugin_statistic.wasm" "$P/dist/plugin.wasm"
python "$P/scripts/oiplugin.py" check "$P"
python "$P/scripts/oiplugin.py" pack "$P" --key "$P/.keys/jiusi.key" --publisher jiusi \
  --out "$P/dist/jiusi.statistic-0.1.0.oiplugin"
python "$P/scripts/oiplugin.py" verify "$P/dist/jiusi.statistic-0.1.0.oiplugin"
python "$P/scripts/local_install.py" status --project "$P"
```

改过任何进包的文件都要**重新打包**：摘要认字节不认内容，`plugin.toml` 的
`change-notes`、图标、HTML、wasm 都算在内。

## 本地验收

`Settings › Plugins › Install from Disk` 选 `dist/jiusi.statistic-0.1.0.oiplugin`，
确认三件事：版本号是 `0.1.0`、右侧「代码统计」面板能出数据、安装弹窗只问
`project.read` 和 `storage` 两项低风险权限。

开发期想边改边看，用 `python scripts/local_install.py link --apply --project .`；
**出包前务必 `clean --apply` 摘掉链接**，否则链接式开发插件会和安装包抢同一个 id。

## 上架步骤

1. `git push` 源码到 `jiusi666/openidea-statistic`。
2. `git tag v0.1.0 && git push origin v0.1.0`。
3. 网页发 Release：tag 选 `v0.1.0`，上传
   `dist/jiusi.statistic-0.1.0.oiplugin`（**文件名不能改**），必须点 Publish，
   不能存草稿。
4. `python scripts/check_release.py` 下载附件、核 sha256、验签、比对本地字节。
5. 把 `publish/plugins/jiusi.statistic.toml` 复制进 fork 的 `plugins/`，正文用
   `publish/PR.md`；`publish/publishers/jiusi.toml` **不要提交**，官方 registry 已有。
6. `python scripts/open_pr.py --dry-run` 看计划，再 `python scripts/open_pr.py` 提 PR。
   CI 会下载附件、核 sha256、验签并跑 `openidea-plugin check`。

之后发新版（0.1.1、0.2.0…）改用 `openidea-plugin-update` skill：推进版本号、
在条目里追加 `[[versions]]`（保留旧版）、只改条目的更新 PR。
