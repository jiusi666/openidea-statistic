# 更新 PR：jiusi.statistic → 0.2.0

> 提给 [l0ng-ai/openidea-plugins](https://github.com/l0ng-ai/openidea-plugins) 的更新 PR 正文。

## 标题

```
Update jiusi.statistic to 0.2.0
```

## 正文

```markdown
### jiusi.statistic → 0.2.0

Adds `0.2.0` to the version list. `0.1.1` stays published, so anyone whose `engines.openidea` cannot load the new build keeps getting it. This release keeps `engines.openidea = "^1.4"`; it uses no new capability and no 1.5+ interface.

Release: https://github.com/jiusi666/openidea-statistic/releases/tag/v0.2.0

#### What changed

Before, the plugin only counted lines. It now also tells you which file to refactor:

- **Per-file view** with total/code lines, an approximate cyclomatic complexity (`CC = 1 + if/for/while/case/catch/&&/||`), longest function length and start line, max nesting depth, TODO/FIXME count, and a weighted 0–100 risk score. The file list sorts by risk and can be filtered to high-risk files only.
- **Project tree badges** via `decorations.set_file`: high-risk files get a `!!` (≥70) or `!` (≥45) badge whose tooltip spells out lines, CC, longest function and TODO count. Cleared and re-applied on every scan, capped at 150 files, can be switched off in settings.
- **Click to jump**: clicking a file row calls `open-location` at the longest function's start line.
- Language/directory summaries also carry CC and TODO totals; the overview cards show file count, total/code lines, high-risk count, comment ratio and TODO count.
- New settings: `decorateTree`, `warnFileLines` (800), `warnFunctionLines` (60), `warnComplexity` (15).

The complexity numbers are deliberately described as text-level approximations — no AST, no parser; strings and trailing comments are stripped before counting. Documentation, config and JSON files are excluded from complexity and scoring.

#### Capabilities

Unchanged: `project.read` + `storage`, both Low. Tree badges are part of `decorations`, which requires only `project.read`. No `net:`, no `process:`, no `editor.write`, no `vcs.*`. Nothing leaves the machine.

#### Package

`jiusi.statistic-0.2.0.oiplugin`, 137 KB, built from the source in this repository for plugin API `^1.4`, signed by publisher `jiusi`, sha256 `e34cad90901341552f8ff1bb0c8b1670c5227a4118bdde0191d90bebedd33533`.
```
