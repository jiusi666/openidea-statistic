# PR：新增插件 jiusi.statistic

> 提给 [l0ng-ai/openidea-plugins](https://github.com/l0ng-ai/openidea-plugins) 的 PR 正文草稿。
> publisher `jiusi` 已在 registry 中，本次 PR 只需提交 `plugins/jiusi.statistic.toml`。
> 顺序：先把源码推到 `jiusi666/openidea-statistic`，发 `v0.1.0` Release 并上传
> `jiusi.statistic-0.1.0.oiplugin`，再跑 `python scripts/open_pr.py`。

## 标题

```
Add jiusi.statistic — project line counts by language and directory
```

## 正文

```markdown
### What it is

`jiusi.statistic` is a side panel that counts a project's code: file count,
total / code / comment / blank lines and bytes, grouped by language or by
top-level directory. It sorts each group, shows a relative bar, and keeps the
last result locally so opening the panel doesn't rescan the whole tree.

Scans run in a cancellable task, skip binary and oversized files, and stop at
20,000 files with a visible "incomplete" warning. Line classification is
heuristic (comment prefixes and block-comment markers, no string-literal
parsing) — it is a project-size overview, not an audit tool.

Source: https://github.com/jiusi666/openidea-statistic (Apache-2.0).

### Capabilities

`project.read` (Low, required) — the only way the plugin sees the project:
`Project::find_files("**/*", 20000)`, `Project::read_file`, `Project::root`.
Read-only; `write_file` is never called and no path outside the project is read.

`storage` (Low, required) — the last scan result and the panel's group / sort /
exclude-directory overrides, on this machine only, removed on uninstall.

No `net:`, no `process:`, no `editor.write`, no `vcs.*`, no `fs.*`, no
`clipboard`. The panel runs in a sandboxed web view with no network of its own;
all data is pushed from the WASM side. Nothing leaves the machine.

### Verification

- `cargo +stable-x86_64-pc-windows-gnu test` — 19 unit tests: line
  classification (blank / line comments / spanning and inline block comments),
  language mapping, binary detection, path normalisation and exclusion,
  language/directory aggregation, saved-state normalisation, snapshot shape,
  message handling and storage round-trips.
- `cargo +stable-x86_64-pc-windows-gnu build --release --target wasm32-wasip2`
  — the shipped `dist/plugin.wasm` is a WebAssembly **component**.
- `python scripts/oiplugin.py check .` — manifest, contributions, icons and
  component magic all pass.
- `python scripts/oiplugin.py verify dist/jiusi.statistic-0.1.0.oiplugin` —
  digest recomputed and Ed25519 signature verified against publisher `jiusi`.
- `node --check` on the panel's inline script — syntax clean.
- `python scripts/local_install.py status --project .` — no conflicting
  Installed / Linked / Dev copy of this id.

### Notes

- Built by hand from this source with `wasm32-wasip2` + component model
  (`engines.openidea = "^1.4"`), no minified or generated-only code; the UI is a
  single HTML file with no external resources.
- Package: `jiusi.statistic-0.1.0.oiplugin`, sha256
  `3d5a1a01a8a839d8e1e9b495e82d80d315e468e296f5202440064929e0cb9c56`.
- The signing key is the one already listed in `publishers/jiusi.toml`; no new
  publisher entry is needed.
```
