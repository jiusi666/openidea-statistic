//! 统计逻辑：全部是不碰 host 的纯函数，方便单元测试。
//!
//! 行分类是启发式的：按语言的整行注释前缀和块注释符号判断，不解析字符串字面量。
//! 这对「看一眼项目规模」足够了，但不是编译器级别的词法分析。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 一个文件的行分类结果。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lines {
    pub total: u64,
    pub code: u64,
    pub comment: u64,
    pub blank: u64,
}

impl Lines {
    pub fn add_assign(&mut self, other: Lines) {
        self.total += other.total;
        self.code += other.code;
        self.comment += other.comment;
        self.blank += other.blank;
    }
}

/// 某种语言的注释规则。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lang {
    pub name: &'static str,
    pub line: &'static [&'static str],
    pub block: Option<(&'static str, &'static str)>,
}

const C_BLOCK: Option<(&str, &str)> = Some(("/*", "*/"));
const HTML_BLOCK: Option<(&str, &str)> = Some(("<!--", "-->"));
const HASH_LINE: &[&str] = &["#"];
const SLASH_LINE: &[&str] = &["//"];
const DASH_LINE: &[&str] = &["--"];
const NO_LINE: &[&str] = &[];

fn lang(
    name: &'static str,
    line: &'static [&'static str],
    block: Option<(&'static str, &'static str)>,
) -> Lang {
    Lang { name, line, block }
}

/// 按文件名 / 扩展名归到语言组。无法识别的扩展名归到「其他」，仍然计入统计。
pub fn language_of(path: &str) -> Lang {
    let normalized = path.replace('\\', "/");
    let file = normalized
        .rsplit('/')
        .next()
        .unwrap_or(&normalized)
        .to_ascii_lowercase();

    // 没有扩展名、但值得单独归类的文件。
    match file.as_str() {
        "dockerfile" | "containerfile" => return lang("Dockerfile", HASH_LINE, None),
        "makefile" | "gnumakefile" => return lang("Makefile", HASH_LINE, None),
        "cmakelists.txt" => return lang("CMake", HASH_LINE, None),
        "cargo.lock" | "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml" | "poetry.lock"
        | "composer.lock" | "go.sum" | "gemfile.lock" => return lang("Lockfile", NO_LINE, None),
        ".gitignore" | ".gitattributes" | ".editorconfig" | ".dockerignore" | ".npmignore"
        | ".env" | ".env.example" | "license" | "notice" => return lang("Config", HASH_LINE, None),
        _ => {}
    }

    let ext = match file.rsplit_once('.') {
        // 点开头的隐藏文件（如 .prettierrc）本身不是扩展名。
        Some((head, tail)) if !head.is_empty() && !file.starts_with('.') => tail,
        _ => "",
    };

    match ext {
        "rs" => lang("Rust", SLASH_LINE, C_BLOCK),
        "py" | "pyi" | "pyx" => lang("Python", HASH_LINE, None),
        "js" | "mjs" | "cjs" | "jsx" => lang("JavaScript", SLASH_LINE, C_BLOCK),
        "ts" | "mts" | "cts" | "tsx" => lang("TypeScript", SLASH_LINE, C_BLOCK),
        "java" => lang("Java", SLASH_LINE, C_BLOCK),
        "kt" | "kts" => lang("Kotlin", SLASH_LINE, C_BLOCK),
        "scala" | "sc" => lang("Scala", SLASH_LINE, C_BLOCK),
        "groovy" | "gradle" => lang("Groovy", SLASH_LINE, C_BLOCK),
        "go" => lang("Go", SLASH_LINE, C_BLOCK),
        "c" | "h" => lang("C", SLASH_LINE, C_BLOCK),
        "cc" | "cpp" | "cxx" | "hpp" | "hh" | "hxx" => lang("C++", SLASH_LINE, C_BLOCK),
        "cs" => lang("C#", SLASH_LINE, C_BLOCK),
        "swift" => lang("Swift", SLASH_LINE, C_BLOCK),
        "m" | "mm" => lang("Objective-C", SLASH_LINE, C_BLOCK),
        "php" | "phtml" => lang("PHP", &["//", "#"], C_BLOCK),
        "rb" | "rake" => lang("Ruby", HASH_LINE, None),
        "sh" | "bash" | "zsh" | "fish" | "ksh" => lang("Shell", HASH_LINE, None),
        "ps1" | "psm1" => lang("PowerShell", HASH_LINE, None),
        "sql" => lang("SQL", DASH_LINE, C_BLOCK),
        "lua" => lang("Lua", DASH_LINE, None),
        "hs" | "lhs" => lang("Haskell", DASH_LINE, C_BLOCK),
        "ex" | "exs" => lang("Elixir", HASH_LINE, None),
        "erl" | "hrl" => lang("Erlang", &["%"], None),
        "dart" => lang("Dart", SLASH_LINE, C_BLOCK),
        "vue" | "svelte" | "astro" => lang("Vue / Svelte", SLASH_LINE, HTML_BLOCK),
        "html" | "htm" | "xml" | "xsl" | "svg" => lang("HTML / XML", NO_LINE, HTML_BLOCK),
        "css" | "scss" | "sass" | "less" => lang("CSS", NO_LINE, C_BLOCK),
        "md" | "mdx" | "markdown" => lang("Markdown", NO_LINE, HTML_BLOCK),
        "json" | "jsonc" | "json5" => lang("JSON", SLASH_LINE, C_BLOCK),
        "yaml" | "yml" => lang("YAML", HASH_LINE, None),
        "toml" => lang("TOML", HASH_LINE, None),
        "ini" | "cfg" | "conf" | "properties" => lang("Config", &["#", ";"], None),
        "tf" | "tfvars" | "hcl" => lang("Terraform", &["//", "#"], C_BLOCK),
        "proto" => lang("Protobuf", SLASH_LINE, C_BLOCK),
        "graphql" | "gql" => lang("GraphQL", HASH_LINE, None),
        "bat" | "cmd" => lang("Batch", &["rem", "::"], None),
        "r" | "rmd" => lang("R", HASH_LINE, None),
        "jl" => lang("Julia", HASH_LINE, None),
        "clj" | "cljs" => lang("Clojure", &[";"], None),
        "lisp" | "el" | "scm" => lang("Lisp", &[";"], None),
        "tex" | "sty" => lang("LaTeX", &["%"], None),
        "txt" => lang("Text", NO_LINE, None),
        _ => lang("其他", NO_LINE, None),
    }
}

fn starts_with_any(line: &str, prefixes: &[&str]) -> bool {
    prefixes
        .iter()
        .any(|p| !p.is_empty() && line.starts_with(p))
}

/// 统计一份文本的总行 / 代码行 / 注释行 / 空行。
///
/// 规则：空行单独算 blank；块注释内的行算 comment；整行以行注释前缀开头算 comment；
/// 块注释与代码混在一行时算 code。字符串里的注释符号会被误判，属于已知的启发式误差。
pub fn count_lines(text: &str, spec: Lang) -> Lines {
    let mut out = Lines::default();
    let mut in_block = false;

    for raw in text.lines() {
        let line = raw.trim();
        out.total += 1;

        if line.is_empty() {
            if in_block {
                out.comment += 1;
            } else {
                out.blank += 1;
            }
            continue;
        }

        if in_block {
            out.comment += 1;
            if let Some((_, end)) = spec.block
                && line.contains(end)
            {
                in_block = false;
            }
            continue;
        }

        if starts_with_any(line, spec.line) {
            out.comment += 1;
            continue;
        }

        if let Some((start, end)) = spec.block
            && let Some(pos) = line.find(start)
        {
            let has_code_before = !line[..pos].trim().is_empty();
            let after_open = &line[pos + start.len()..];
            match after_open.find(end) {
                Some(end_pos) => {
                    let tail = after_open[end_pos + end.len()..].trim();
                    if has_code_before || !tail.is_empty() {
                        out.code += 1;
                    } else {
                        out.comment += 1;
                    }
                }
                None => {
                    if has_code_before {
                        out.code += 1;
                    } else {
                        out.comment += 1;
                    }
                    in_block = true;
                }
            }
            continue;
        }

        out.code += 1;
    }

    out
}

/// 粗判二进制：解码后的文本里出现 NUL 就跳过行数统计。
pub fn is_binary(text: &str) -> bool {
    text.as_bytes().contains(&0)
}

/// 把绝对路径换成项目根相对路径，统一成正斜杠。
pub fn relative_path(root: &str, path: &str) -> String {
    let r = root.replace('\\', "/");
    let p = path.replace('\\', "/");
    let trimmed = r.trim_end_matches('/');
    if trimmed.len() <= p.len() && p.starts_with(trimmed) {
        let rest = &p[trimmed.len()..];
        return rest.trim_start_matches('/').to_string();
    }
    p
}

/// 是否应跳过：命中的排除目录、或（默认）隐藏目录 / 隐藏文件。
pub fn should_exclude(rel: &str, excludes: &[String], include_hidden: bool) -> bool {
    let normalized = rel.replace('\\', "/");
    let mut parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return true;
    }
    let file = parts.pop().unwrap_or("");
    for part in parts.iter().chain(std::iter::once(&file)) {
        if !include_hidden && part.starts_with('.') {
            return true;
        }
    }
    let lowered_dirs: Vec<String> = parts.iter().map(|s| s.to_ascii_lowercase()).collect();
    excludes
        .iter()
        .any(|x| lowered_dirs.iter().any(|d| d == x.as_str()))
}

/// 一级目录名；根目录里的文件归到「(根目录)」。
pub fn top_dir(rel: &str) -> String {
    let normalized = rel.replace('\\', "/");
    let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() <= 1 {
        "(根目录)".to_string()
    } else {
        parts[0].to_string()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub key: String,
    pub files: u64,
    pub lines: Lines,
    pub bytes: u64,
}

/// 一次扫描的汇总，可序列化进 storage，也可直接推给面板。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub groups: Vec<Group>,
    pub dirs: Vec<Group>,
    pub files: u64,
    pub lines: Lines,
    pub bytes: u64,
    pub skipped: u64,
    pub truncated: bool,
    pub project: String,
}

#[derive(Default)]
pub struct Acc {
    pub by_lang: BTreeMap<String, Group>,
    pub by_dir: BTreeMap<String, Group>,
    pub files: u64,
    pub lines: Lines,
    pub bytes: u64,
    pub skipped: u64,
}

impl Acc {
    pub fn skip(&mut self) {
        self.skipped += 1;
    }

    pub fn add_file(&mut self, rel: &str, name: &str, bytes: usize, lines: Lines) {
        self.files += 1;
        self.bytes += bytes as u64;
        self.lines.add_assign(lines);
        add_group(&mut self.by_lang, name, bytes, lines);
        add_group(&mut self.by_dir, &top_dir(rel), bytes, lines);
    }

    pub fn finish(self, project: &str, truncated: bool) -> Report {
        Report {
            groups: self.by_lang.into_values().collect(),
            dirs: self.by_dir.into_values().collect(),
            files: self.files,
            lines: self.lines,
            bytes: self.bytes,
            skipped: self.skipped,
            truncated,
            project: project.to_string(),
        }
    }
}

fn add_group(map: &mut BTreeMap<String, Group>, key: &str, bytes: usize, lines: Lines) {
    let entry = map.entry(key.to_string()).or_insert_with(|| Group {
        key: key.to_string(),
        ..Group::default()
    });
    entry.files += 1;
    entry.bytes += bytes as u64;
    entry.lines.add_assign(lines);
}

/// 状态栏用的千分位格式，纯函数。
pub fn group_number(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c() -> Lang {
        language_of("src/main.rs")
    }

    #[test]
    fn splits_blank_code_and_line_comments() {
        let text = "fn main() {\n\n// hello\n    let x = 1;\n}\n";
        assert_eq!(
            count_lines(text, c()),
            Lines {
                total: 5,
                code: 3,
                comment: 1,
                blank: 1
            }
        );
    }

    #[test]
    fn counts_a_spanning_block_comment() {
        let text = "/* start\n middle\n end */\nlet x = 1;\n";
        assert_eq!(
            count_lines(text, c()),
            Lines {
                total: 4,
                code: 1,
                comment: 3,
                blank: 0
            }
        );
    }

    #[test]
    fn inline_block_comment_on_one_line_is_a_comment() {
        let text = "/* only */\nint x = 1; /* tail */\n";
        assert_eq!(
            count_lines(text, c()),
            Lines {
                total: 2,
                code: 1,
                comment: 1,
                blank: 0
            }
        );
    }

    #[test]
    fn empty_text_is_zero_lines() {
        assert_eq!(count_lines("", c()), Lines::default());
    }

    #[test]
    fn single_line_without_newline_counts_as_one() {
        assert_eq!(
            count_lines("let x = 1;", c()),
            Lines {
                total: 1,
                code: 1,
                comment: 0,
                blank: 0
            }
        );
    }

    #[test]
    fn hash_and_html_comment_rules() {
        let py = count_lines("# a\nprint(1)\n\n", language_of("a.py"));
        assert_eq!(
            py,
            Lines {
                total: 3,
                code: 1,
                comment: 1,
                blank: 1
            }
        );
        let html = count_lines("<!-- a\n b -->\n<p>x</p>\n", language_of("a.html"));
        assert_eq!(
            html,
            Lines {
                total: 3,
                code: 1,
                comment: 2,
                blank: 0
            }
        );
    }

    #[test]
    fn language_mapping_covers_common_files() {
        assert_eq!(language_of("src/lib.rs").name, "Rust");
        assert_eq!(language_of("app\\main.py").name, "Python");
        assert_eq!(language_of("index.tsx").name, "TypeScript");
        assert_eq!(language_of("README.md").name, "Markdown");
        assert_eq!(language_of("Dockerfile").name, "Dockerfile");
        assert_eq!(language_of("weird.xyz").name, "其他");
    }

    #[test]
    fn binary_detection() {
        assert!(is_binary("abc\0def"));
        assert!(!is_binary("abc\n def"));
    }

    #[test]
    fn relative_path_handles_both_separators() {
        assert_eq!(relative_path("/home/p", "/home/p/src/a.rs"), "src/a.rs");
        assert_eq!(
            relative_path("C:\\work\\p", "C:\\work\\p\\src\\a.rs"),
            "src/a.rs"
        );
        assert_eq!(relative_path("/other", "/home/p/a.rs"), "/home/p/a.rs");
    }

    #[test]
    fn excludes_match_directory_segments_only() {
        let ex = vec!["node_modules".to_string(), "target".to_string()];
        assert!(should_exclude("web/node_modules/a/index.js", &ex, false));
        assert!(should_exclude("web/.cache/a.js", &ex, false));
        assert!(!should_exclude("web/src/my_node_modules.js", &ex, false));
        assert!(!should_exclude("web/.cache/a.js", &ex, true));
    }

    #[test]
    fn top_level_dir_and_root() {
        assert_eq!(top_dir("src/a/b.rs"), "src");
        assert_eq!(top_dir("Cargo.toml"), "(根目录)");
    }

    #[test]
    fn accumulator_groups_by_language_and_directory() {
        let mut acc = Acc::default();
        let one = Lines {
            total: 10,
            code: 7,
            comment: 2,
            blank: 1,
        };
        acc.add_file("src/a.rs", "Rust", 100, one);
        acc.add_file("src/b.rs", "Rust", 200, one);
        acc.add_file("docs/c.md", "Markdown", 50, one);
        acc.skip();
        let r = acc.finish("demo", false);
        assert_eq!(r.files, 3);
        assert_eq!(r.lines.total, 30);
        assert_eq!(r.bytes, 350);
        assert_eq!(r.skipped, 1);
        assert_eq!(r.groups.len(), 2);
        assert_eq!(r.groups[0].key, "Markdown");
        assert_eq!(r.groups[1].files, 2);
        assert_eq!(r.dirs.len(), 2);
        assert_eq!(r.project, "demo");
    }

    #[test]
    fn numbers_are_grouped_by_thousands() {
        assert_eq!(group_number(0), "0");
        assert_eq!(group_number(999), "999");
        assert_eq!(group_number(1_234), "1,234");
        assert_eq!(group_number(12_345_678), "12,345,678");
    }
}
