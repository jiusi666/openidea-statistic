# PR：更新 jiusi.statistic → 0.2.1

> 提给 [l0ng-ai/openidea-plugins](https://github.com/l0ng-ai/openidea-plugins) 的更新 PR 正文。
> 这一版是对 PR #173 review（`CHANGES_REQUESTED`，P2）的回应。

## 标题

```
Update jiusi.statistic to 0.2.1
```

## 正文

```markdown
### jiusi.statistic → 0.2.1

Fixes the P2 from [review #5915752765](https://github.com/l0ng-ai/openidea-plugins/pull/173#pullrequestreview-5915752765): indentation-based functions still open at end of file were never finalized.

The registry entry now pins `0.2.1` instead of `0.2.0`. `0.2.0` was published as a GitHub release but never merged into the registry, so no user can have received it; its release notes now carry a warning pointing at `0.2.1`. `0.1.0` and `0.1.1` stay as they are, and `engines.openidea = "^1.4"` is unchanged — no new capability, no 1.5+ interface.

Release: https://github.com/jiusi666/openidea-statistic/releases/tag/v0.2.1

#### The fix

`measure()` ran a dedicated loop for indentation languages that recorded a function only when a later nonblank, noncomment line dedented to its starting indentation. The loop ended without finalizing the function still open, so a file whose last function runs to EOF reported `max_fn = 0` (single function) or an earlier function (several functions). After the loop, the pending function is now closed at the last line that carried code:

```rust
if let Some((_, start)) = open {
    let len = last_code.saturating_sub(start) + 1;
    ...
}
```

Trailing blank and comment lines do not extend the function, and the length is computed exactly the way the dedent branch computes it (`lineno - start` there, `last_code - start + 1` here, which are the same number once the dedent line exists).

The knock-on effects the review pointed out are gone: the score for "811-line function with 15 plain `if` branches" is now **51** at EOF, identical to the same function followed by an unrelated top-level statement (it was 26, under the 45 high-risk threshold, so the badge and the high-risk filter missed the file).

#### Tests

29 → 33. Four new cases in `src/scan.rs`:

- `finalizes_indent_function_at_eof` — single function running to EOF: 66 lines at line 1, with and without a trailing newline, and with trailing blank + comment lines.
- `finalizes_longest_indent_function_at_eof` — longest function last, preceded by another function: reports line 4 / 5 lines, not the earlier function.
- `eof_function_length_matches_dedent_case` — the same function with and without a trailing top-level statement yields equal `max_fn`, `fn_line` and `risk_score`.
- `eof_badge_threshold_is_reached_like_dedent_case` — the 811-line / 15-`if` scenario from the review, asserting exactly 51 on both sides.

I verified the first three fail against `v0.2.0` and pass after the change (`left: 0, right: 66`; `left: 1, right: 4`), and that the fourth reports 51. `cargo clippy --all-targets -- -D warnings` is clean.

#### Capabilities

Unchanged: `project.read` + `storage`, both Low. Tree badges are part of `decorations`, which requires only `project.read`. No `net:`, no `process:`, no `editor.write`, no `vcs.*`. Nothing leaves the machine.

#### Package

`jiusi.statistic-0.2.1.oiplugin`, 141014 bytes, built from source tag `v0.2.1` for plugin API `^1.4`, signed by publisher `jiusi`, sha256 `b79fe50bef0f70bcb1df887d37ba1bbd2862c2a0ea3d139c96d2c800b5969b79`, digest `21116abbbc98da92bdc72ce8a9a184ef5b3f5227ed320acdb4f6870a92255764`.
```

## 回复 review 的评论

```markdown
Thanks for the precise repro — confirmed and fixed in `0.2.1`.

`measure()`'s indentation loop now finalizes whatever function is still open after the loop, using the last line that carried code as the EOF boundary. Trailing blank/comment lines don't extend it, and the length uses the same arithmetic as the dedent branch, so the two paths agree by construction.

Your 811-line / 15-`if` case now scores **51** at EOF, same as when a top-level statement follows — it was 26 before.

I added four tests (`finalizes_indent_function_at_eof`, `finalizes_longest_indent_function_at_eof`, `eof_function_length_matches_dedent_case`, `eof_badge_threshold_is_reached_like_dedent_case`; 29 → 33). The first three fail on `v0.2.0` and pass now, with the fourth asserting exactly 51 on both sides.

The pin now points at `0.2.1` (`b79fe50bef0f70bcb1df887d37ba1bbd2862c2a0ea3d139c96d2c800b5969b79`). `0.2.0` was never merged so nothing has to be pulled; its release notes carry a warning pointing at `0.2.1`. `0.1.0` / `0.1.1` are untouched and `engines.openidea` stays `^1.4`.
```
