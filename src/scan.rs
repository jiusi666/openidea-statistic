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

// ---------------------------------------------------------------------------
// 复杂度类指标
//
// 全部是文本级近似，不建语法树：分支按关键字计数、嵌套按括号深度或缩进、
// 函数长度靠「函数头 + 大括号配对」找。字符串与行尾注释会先剔掉，
// 但模板字符串插值、宏生成的代码这类结构仍可能误判，面板里必须标清是近似值。
// ---------------------------------------------------------------------------

/// 风险评分的分母，同时决定「超标」判断。来自设置面板。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    /// 单文件代码行超过这个数算「超长文件」。
    pub file_lines: u64,
    /// 单个函数体超过这个行数算「超长函数」。
    pub fn_lines: u32,
    /// 单文件圈复杂度超过这个数算「高复杂度」。
    pub complexity: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            file_lines: 800,
            fn_lines: 60,
            complexity: 15,
        }
    }
}

/// 单文件的复杂度类指标。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Metrics {
    /// 圈复杂度近似 = 1 + 分支关键字数（if / for / while / case / catch / && / ||）。
    pub cc: u32,
    /// 最大嵌套深度：括号语言按 `{` 深度，缩进语言按每 4 空格折算。
    pub depth: u32,
    /// 最长函数体行数（含首尾行）。
    pub max_fn: u32,
    /// 最长函数的起始行，1 基；0 表示没识别到。
    pub fn_line: u32,
    /// TODO / FIXME / XXX / HACK 标记数。
    pub todos: u32,
}

/// 语言的结构族：决定嵌套与函数怎么量。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    Brace,
    Indent,
    None,
}

fn family(spec: Lang) -> Family {
    match spec.name {
        "Rust" | "Java" | "Kotlin" | "Scala" | "Groovy" | "Go" | "C" | "C++" | "C#" | "Swift"
        | "Objective-C" | "PHP" | "JavaScript" | "TypeScript" | "Dart" | "Vue / Svelte" => {
            Family::Brace
        }
        "Python" | "Ruby" => Family::Indent,
        _ => Family::None,
    }
}

/// 这个语言值不值得算复杂度（文档、配置、JSON 不算）。
pub fn is_code(spec: Lang) -> bool {
    family(spec) != Family::None
}

fn branches(spec: Lang) -> &'static [&'static str] {
    match spec.name {
        "Rust" => &["if", "for", "while", "match", "&&", "||"],
        "Go" => &["if", "for", "case", "&&", "||"],
        "Python" | "Ruby" => &["if", "elif", "for", "while", "except", "and", "or"],
        "Kotlin" | "Scala" => &["if", "for", "while", "when", "case", "catch", "&&", "||"],
        _ => &["if", "for", "while", "case", "catch", "&&", "||"],
    }
}

/// 出现在行首但不是函数头的关键字。
const CONTROL_WORDS: &[&str] = &[
    "if", "for", "while", "switch", "catch", "else", "try", "do", "return", "new", "throw",
    "case", "when", "synchronized", "using", "lock", "assert", "super", "this", "await", "yield",
    "typeof", "delete", "match", "select", "with", "print",
];

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// 按标识符边界计数；`&&`、`||` 这类符号按普通子串计数。
fn count_token(hay: &str, token: &str) -> u32 {
    if token.is_empty() {
        return 0;
    }
    if !token
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return hay.matches(token).count() as u32;
    }
    let bytes = hay.as_bytes();
    let mut n = 0;
    let mut from = 0;
    while let Some(pos) = hay[from..].find(token) {
        let start = from + pos;
        let end = start + token.len();
        let before_ok = start == 0 || !is_ident_byte(bytes[start - 1]);
        let after_ok = end >= bytes.len() || !is_ident_byte(bytes[end]);
        if before_ok && after_ok {
            n += 1;
        }
        from = end;
    }
    n
}

/// 剔掉字符串字面量与行尾注释，避免把 `"if"` 或注释里的括号算进指标。
fn strip_noise(line: &str, quote_apostrophe: bool, hash_comment: bool) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if let Some(q) = quote {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match c {
            '"' | '`' => {
                quote = Some(c);
                out.push(' ');
            }
            '\'' if quote_apostrophe => {
                quote = Some(c);
                out.push(' ');
            }
            '/' if chars.get(i + 1) == Some(&'/') => break,
            '#' if hash_comment => break,
            _ => out.push(c),
        }
        i += 1;
    }
    out
}

/// 缩进宽度，Tab 按 4 列算。
fn indent_width(line: &str) -> usize {
    let mut width = 0;
    for c in line.chars() {
        match c {
            ' ' => width += 1,
            '\t' => width += 4,
            _ => break,
        }
    }
    width
}

/// 是否是函数头：形如 `public void run() {`、`fun f(): Int {`、`fn f() -> i32 {`、
/// `func (r *T) F() error {`。判断比编译器宽松，宁可漏判不误判。
fn looks_like_fn_head(code: &str) -> bool {
    let trimmed = code.trim();
    if trimmed.is_empty() || !trimmed.contains('(') {
        return false;
    }
    let head = trimmed.trim_start_matches(|c: char| !c.is_alphanumeric() && c != '_');
    if head.is_empty() {
        return false;
    }
    let word: String = head
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if CONTROL_WORDS.contains(&word.to_ascii_lowercase().as_str()) {
        return false;
    }
    let close = match trimmed.rfind(')') {
        Some(pos) => pos,
        None => return false,
    };
    let tail = trimmed[close + 1..].trim();
    // 大括号必须和签名同一行，否则 `foo(bar)` 这种调用也会被当成函数头。
    tail.ends_with('{')
        || tail.starts_with(':')
        || tail.starts_with("throws")
        || tail.starts_with("->")
        || tail.starts_with("where")
}

fn is_def_line(body: &str) -> bool {
    let body = body.trim_start_matches('@');
    let body = body.trim_start();
    body.starts_with("def ") || body.starts_with("async def ") || body.starts_with("class ")
}

fn count_todos(text: &str) -> u32 {
    let lowered = text.to_lowercase();
    ["todo", "fixme", "xxx", "hack"]
        .iter()
        .map(|t| count_token(&lowered, t))
        .sum()
}

/// 判断这一行是否属于块注释（`/* … */`），顺便维护跨行状态。
/// 行首就是注释符的行整行跳过——`/** if 为空则… */` 里的关键字不该算分支。
fn skip_as_block_comment(raw: &str, spec: Lang, in_block: &mut bool) -> bool {
    let Some((start, end)) = spec.block else {
        return false;
    };
    let trimmed = raw.trim();
    if *in_block {
        if trimmed.contains(end) {
            *in_block = false;
        }
        return true;
    }
    if let Some(rest) = trimmed.strip_prefix(start) {
        if !rest.contains(end) {
            *in_block = true;
        }
        return true;
    }
    false
}

/// 量一份源码的复杂度类指标。
pub fn measure(text: &str, spec: Lang) -> Metrics {
    let fam = family(spec);
    let mut m = Metrics {
        todos: count_todos(text),
        ..Metrics::default()
    };
    if fam == Family::None || text.is_empty() {
        return m;
    }
    let tokens = branches(spec);
    // Rust 的 `'a` 生命周期会把单引号当字符串起点，反而吞掉后面的代码。
    let quote_apostrophe = spec.name != "Rust";

    let mut cc = 1u32;
    let mut depth = 0i32;
    let mut max_depth = 0i32;
    let mut fn_base: Option<i32> = None;
    let mut fn_start = 0u32;
    let mut pending: Option<(u32, i32, u32)> = None;
    let mut in_block = false;

    for (i, raw) in text.lines().enumerate() {
        let lineno = i as u32 + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || starts_with_any(trimmed, spec.line) {
            continue;
        }
        if skip_as_block_comment(raw, spec, &mut in_block) {
            continue;
        }
        let code = strip_noise(raw, quote_apostrophe, fam == Family::Indent);
        if code.trim().is_empty() {
            continue;
        }

        if fam == Family::Indent {
            for t in tokens {
                cc += count_token(&code, t);
            }
            let d = (indent_width(raw) / 4) as i32;
            if d > max_depth {
                max_depth = d;
            }
            continue;
        }

        for t in tokens {
            cc += count_token(&code, t);
        }
        let opens = code.matches('{').count() as i32;
        let closes = code.matches('}').count() as i32;

        if fn_base.is_none() && pending.is_none() && looks_like_fn_head(&code) {
            pending = Some((lineno, depth, 0));
        }
        depth += opens - closes;
        if depth > max_depth {
            max_depth = depth;
        }
        if let Some((line, base, age)) = pending {
            if depth > base {
                fn_base = Some(base);
                fn_start = line;
                pending = None;
            } else if age >= 3 {
                pending = None;
            } else {
                pending = Some((line, base, age + 1));
            }
        }
        if let Some(base) = fn_base
            && depth <= base
        {
            let len = lineno.saturating_sub(fn_start) + 1;
            if len > m.max_fn {
                m.max_fn = len;
                m.fn_line = fn_start;
            }
            fn_base = None;
        }
    }

    // 缩进语言单独走一遍：这里复用上面的循环结构太绕，干脆拆开。
    if fam == Family::Indent {
        let mut open: Option<(usize, u32)> = None;
        let mut in_block = false;
        for (i, raw) in text.lines().enumerate() {
            let lineno = i as u32 + 1;
            let trimmed = raw.trim();
            if trimmed.is_empty() || starts_with_any(trimmed, spec.line) {
                continue;
            }
            if skip_as_block_comment(raw, spec, &mut in_block) {
                continue;
            }
            let code = strip_noise(raw, true, true);
            let body = code.trim();
            if body.is_empty() {
                continue;
            }
            let indent = indent_width(raw);
            if let Some((base, start)) = open
                && indent <= base
            {
                let len = lineno.saturating_sub(start);
                if len > m.max_fn {
                    m.max_fn = len;
                    m.fn_line = start;
                }
                open = None;
            }
            if open.is_none() && is_def_line(body) {
                open = Some((indent, lineno));
            }
        }
    }

    m.cc = cc;
    m.depth = max_depth.max(0) as u32;
    m
}

/// 风险分 0–100：复杂度、超长函数、超长文件、深嵌套、TODO 密度加权。
/// 只对代码文件有意义，调用方应先判 `is_code`。
pub fn risk_score(m: &Metrics, lines: Lines, lim: &Limits) -> u32 {
    let mut score = 0u32;
    let cc = m.cc.max(1) as u64;
    let cc_limit = lim.complexity.max(1) as u64;
    score += if cc >= cc_limit * 3 {
        30
    } else if cc >= cc_limit * 2 {
        22
    } else if cc >= cc_limit {
        14
    } else if cc * 10 >= cc_limit * 6 {
        7
    } else {
        0
    };
    let fn_limit = lim.fn_lines.max(1) as u64;
    let longest = m.max_fn as u64;
    score += if longest >= fn_limit * 3 {
        25
    } else if longest >= fn_limit * 2 {
        18
    } else if longest >= fn_limit {
        10
    } else {
        0
    };
    let file_limit = lim.file_lines.max(1);
    let code = lines.code.max(1);
    score += if code >= file_limit * 2 {
        20
    } else if code >= file_limit {
        12
    } else {
        0
    };
    score += match m.depth {
        d if d >= 7 => 10,
        d if d >= 5 => 7,
        d if d >= 4 => 4,
        _ => 0,
    };
    score += match m.todos {
        t if t >= 10 => 5,
        t if t >= 3 => 2,
        _ => 0,
    };
    score.min(100)
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
    /// 组内圈复杂度合计（只统计代码文件）。
    pub cc: u32,
    /// 组内 TODO / FIXME 标记数。
    pub todos: u32,
}

/// 文件级结果，供「按文件」页签展示与点击跳转。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRow {
    pub path: String,
    pub lang: String,
    pub total: u64,
    pub code: u64,
    pub comment: u64,
    pub bytes: u64,
    pub cc: u32,
    pub depth: u32,
    pub max_fn: u32,
    pub fn_line: u32,
    pub todos: u32,
    pub score: u32,
}

/// 文件级结果最多保留这么多行：按风险分和代码行从高到低截断，够看「该重构谁」。
pub const MAX_FILE_ROWS: usize = 5000;
/// 风险分达到这个值才算「高风险文件」。
pub const HIGH_RISK: u32 = 45;

/// 一次扫描的汇总，可序列化进 storage，也可直接推给面板。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Report {
    pub groups: Vec<Group>,
    pub dirs: Vec<Group>,
    pub files: u64,
    pub lines: Lines,
    pub bytes: u64,
    pub skipped: u64,
    pub truncated: bool,
    pub project: String,
    pub file_rows: Vec<FileRow>,
    pub file_rows_truncated: bool,
    pub high_risk: u64,
    pub cc: u32,
    pub todos: u32,
}

#[derive(Default)]
pub struct Acc {
    pub by_lang: BTreeMap<String, Group>,
    pub by_dir: BTreeMap<String, Group>,
    pub files: u64,
    pub lines: Lines,
    pub bytes: u64,
    pub skipped: u64,
    pub rows: Vec<FileRow>,
    pub high_risk: u64,
    pub cc: u32,
    pub todos: u32,
}

impl Acc {
    pub fn skip(&mut self) {
        self.skipped += 1;
    }

    pub fn add_file(&mut self, rel: &str, name: &str, bytes: usize, lines: Lines, m: &Metrics, score: u32) {
        self.files += 1;
        self.bytes += bytes as u64;
        self.lines.add_assign(lines);
        self.cc += m.cc;
        self.todos += m.todos;
        if score >= HIGH_RISK {
            self.high_risk += 1;
        }
        add_group(&mut self.by_lang, name, bytes, lines, m);
        add_group(&mut self.by_dir, &top_dir(rel), bytes, lines, m);
        self.rows.push(FileRow {
            path: rel.replace('\\', "/"),
            lang: name.to_string(),
            total: lines.total,
            code: lines.code,
            comment: lines.comment,
            bytes: bytes as u64,
            cc: m.cc,
            depth: m.depth,
            max_fn: m.max_fn,
            fn_line: m.fn_line,
            todos: m.todos,
            score,
        });
    }

    pub fn finish(mut self, project: &str, truncated: bool) -> Report {
        // 文件多的时候保住最该看的那些：先按风险分，再按代码行。
        self.rows
            .sort_by(|a, b| b.score.cmp(&a.score).then(b.code.cmp(&a.code)));
        let rows_truncated = self.rows.len() > MAX_FILE_ROWS;
        self.rows.truncate(MAX_FILE_ROWS);
        Report {
            groups: self.by_lang.into_values().collect(),
            dirs: self.by_dir.into_values().collect(),
            files: self.files,
            lines: self.lines,
            bytes: self.bytes,
            skipped: self.skipped,
            truncated,
            project: project.to_string(),
            file_rows: self.rows,
            file_rows_truncated: rows_truncated,
            high_risk: self.high_risk,
            cc: self.cc,
            todos: self.todos,
        }
    }
}

fn add_group(
    map: &mut BTreeMap<String, Group>,
    key: &str,
    bytes: usize,
    lines: Lines,
    m: &Metrics,
) {
    let entry = map.entry(key.to_string()).or_insert_with(|| Group {
        key: key.to_string(),
        ..Group::default()
    });
    entry.files += 1;
    entry.bytes += bytes as u64;
    entry.lines.add_assign(lines);
    entry.cc += m.cc;
    entry.todos += m.todos;
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
        acc.add_file(
            "src/a.rs",
            "Rust",
            100,
            one,
            &Metrics {
                cc: 4,
                depth: 2,
                max_fn: 10,
                fn_line: 1,
                todos: 1,
            },
            50,
        );
        acc.add_file("src/b.rs", "Rust", 200, one, &Metrics::default(), 0);
        acc.add_file("docs/c.md", "Markdown", 50, one, &Metrics::default(), 0);
        acc.skip();
        let r = acc.finish("demo", false);
        assert_eq!(r.files, 3);
        assert_eq!(r.lines.total, 30);
        assert_eq!(r.bytes, 350);
        assert_eq!(r.skipped, 1);
        assert_eq!(r.groups.len(), 2);
        assert_eq!(r.groups[0].key, "Markdown");
        assert_eq!(r.groups[1].files, 2);
        assert_eq!(r.groups[1].cc, 4);
        assert_eq!(r.dirs.len(), 2);
        assert_eq!(r.project, "demo");
        // 文件行按风险分排在最前，路径统一成正斜杠。
        assert_eq!(r.file_rows.len(), 3);
        assert_eq!(r.file_rows[0].path, "src/a.rs");
        assert_eq!(r.file_rows[0].score, 50);
        assert_eq!(r.high_risk, 1);
        assert_eq!(r.cc, 4);
        assert!(!r.file_rows_truncated);
    }

    #[test]
    fn numbers_are_grouped_by_thousands() {
        assert_eq!(group_number(0), "0");
        assert_eq!(group_number(999), "999");
        assert_eq!(group_number(1_234), "1,234");
        assert_eq!(group_number(12_345_678), "12,345,678");
    }

    #[test]
    fn measures_branches_depth_and_longest_function() {
        let text = "fn a() {\n    if x && y {\n        for i in 0..3 { println!(\"if\"); }\n    }\n}\n";
        let m = measure(text, language_of("a.rs"));
        // if + && + for 三个分支，字符串里的 if 不算。
        assert_eq!(m.cc, 4);
        assert_eq!(m.depth, 2);
        assert_eq!(m.max_fn, 5);
        assert_eq!(m.fn_line, 1);
    }

    #[test]
    fn comments_and_strings_do_not_inflate_complexity() {
        let text = "// if while for\nfn a() {\n    let s = \"if while for catch\";\n}\n";
        let m = measure(text, language_of("a.rs"));
        assert_eq!(m.cc, 1);
        assert_eq!(m.todos, 0);
    }

    #[test]
    fn control_statements_are_not_mistaken_for_functions() {
        let text = "void f() {\n    if (a) {\n        g(b);\n    }\n    while (c) {\n        h();\n    }\n}\n";
        let m = measure(text, language_of("A.java"));
        assert_eq!(m.max_fn, 8);
        assert_eq!(m.fn_line, 1);
        assert_eq!(m.cc, 3);
    }

    #[test]
    fn javadoc_keywords_do_not_count_as_branches() {
        let text = "/**\n * 如果为空就返回 null if for while\n */\npublic void f() {\n    if (a) { }\n}\n";
        let m = measure(text, language_of("A.java"));
        // 只有真正那句 if 算分支，注释里的 if / for / while 不算。
        assert_eq!(m.cc, 2);
        assert_eq!(m.max_fn, 3);
        assert_eq!(m.fn_line, 4);
    }

    #[test]
    fn measures_python_by_indentation() {
        let text = "def f(x):\n    if x:\n        for i in x:\n            pass\n\ndef g():\n    pass\n";
        let m = measure(text, language_of("a.py"));
        assert_eq!(m.cc, 3);
        assert_eq!(m.depth, 3);
        assert_eq!(m.max_fn, 5);
        assert_eq!(m.fn_line, 1);
    }

    #[test]
    fn counts_todo_markers_and_ignores_docs() {
        let text = "// TODO: 拆开\nfn a() {}\n// FIXME 忘了处理\n";
        assert_eq!(measure(text, language_of("a.rs")).todos, 2);
        // Markdown 不算代码文件，但标记数照样可以统计。
        let md = measure("TODO: 写文档\n", language_of("a.md"));
        assert_eq!(md.todos, 1);
        assert_eq!(md.cc, 0);
        assert!(!is_code(language_of("a.md")));
        assert!(is_code(language_of("a.rs")));
    }

    #[test]
    fn risk_score_weights_by_settings_thresholds() {
        let lim = Limits::default();
        let m = Metrics {
            cc: 45,
            depth: 5,
            max_fn: 130,
            fn_line: 3,
            todos: 3,
        };
        let lines = Lines {
            total: 1000,
            code: 900,
            comment: 60,
            blank: 40,
        };
        // 30（CC）+ 18（函数）+ 12（文件）+ 7（嵌套）+ 2（TODO）
        assert_eq!(risk_score(&m, lines, &lim), 69);

        let quiet = Metrics {
            cc: 3,
            ..Metrics::default()
        };
        assert_eq!(risk_score(&quiet, Lines { code: 40, ..Lines::default() }, &lim), 0);
        // 各项权重上限之和是 90（30+25+20+10+5），再多也不会超过 100。
        let worst = Metrics {
            cc: 400,
            depth: 20,
            max_fn: 900,
            fn_line: 1,
            todos: 50,
        };
        assert_eq!(
            risk_score(&worst, Lines { code: 90_000, ..Lines::default() }, &lim),
            90
        );
    }
}
