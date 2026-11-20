//! Code block: code on a white card inside a light-gray tray. The tray carries the
//! file name or language and a copy button. Optional line numbers, highlighted
//! lines, wrapping, and a max height after which the code scrolls inside.
//!
//! The colorizer is a small built-in scanner for TypeScript/JavaScript, JSON, Bash
//! and Python. Other languages render as plain text.
//!
//! Port losses:
//! - The tokenizer is hand-written instead of a set of regular expressions. It
//!   follows the same rule order and boundaries. Malformed numbers and some
//!   Bash command words may color differently from the original.
//! - Whether the code scrolls is measured when the code, wrap, height or line
//!   numbers change. A window resize that makes it scroll does not re-measure.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/code-block.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Comment,
    Str,
    Keyword,
    Number,
    Literal,
    Function,
    Key,
    Variable,
}

impl TokenKind {
    /// The suffix of the `ui-code__t-*` class.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Comment => "com",
            Self::Str => "str",
            Self::Keyword => "kw",
            Self::Number => "num",
            Self::Literal => "lit",
            Self::Function => "fn",
            Self::Key => "key",
            Self::Variable => "var",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Js,
    Json,
    Bash,
    Python,
}

pub type Token = (Option<TokenKind>, String);

/// Language for a fence name such as `ts` or `Shell`. Unknown names give `None`.
pub fn language_of(name: &str) -> Option<Lang> {
    match name.to_ascii_lowercase().as_str() {
        "ts" | "tsx" | "typescript" | "js" | "jsx" | "javascript" | "mjs" | "cjs" => Some(Lang::Js),
        "json" | "jsonc" => Some(Lang::Json),
        "bash" | "sh" | "shell" | "zsh" | "console" => Some(Lang::Bash),
        "python" | "py" => Some(Lang::Python),
        _ => None,
    }
}

/// The name shown in the header when no filename is given. Unknown names are shown as given.
pub fn language_name(name: &str) -> String {
    let display = match name.to_ascii_lowercase().as_str() {
        "ts" | "typescript" => "TypeScript",
        "tsx" => "TSX",
        "js" | "javascript" | "mjs" | "cjs" => "JavaScript",
        "jsx" => "JSX",
        "json" | "jsonc" => "JSON",
        "bash" => "Bash",
        "sh" | "shell" => "Shell",
        "zsh" => "Zsh",
        "console" => "Terminal",
        "python" | "py" => "Python",
        _ => return name.to_string(),
    };
    display.to_string()
}

/// CRLF or CR to LF, and one trailing newline removed. A trailing newline is how
/// files end, so it does not show as an empty last line.
pub fn normalise_source(code: &str) -> String {
    let unified = code.replace("\r\n", "\n").replace('\r', "\n");
    unified.strip_suffix('\n').unwrap_or(&unified).to_string()
}

/// Digits needed to print the largest line number.
pub fn line_digits(line_count: usize) -> usize {
    line_count.max(1).to_string().len()
}

const JS_KEYWORDS: &[&str] = &[
    "abstract",
    "as",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "declare",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "finally",
    "for",
    "from",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "keyof",
    "let",
    "new",
    "of",
    "private",
    "protected",
    "public",
    "readonly",
    "return",
    "satisfies",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "try",
    "type",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];
const JS_LITERALS: &[&str] = &["true", "false", "null", "undefined", "NaN", "Infinity"];
const BASH_KEYWORDS: &[&str] = &[
    "if", "then", "else", "elif", "fi", "for", "in", "do", "done", "while", "until", "case",
    "esac", "function", "return", "export", "local", "readonly", "unset", "exit",
];
const PY_KEYWORDS: &[&str] = &[
    "and", "as", "assert", "async", "await", "break", "case", "class", "continue", "def", "del",
    "elif", "else", "except", "finally", "for", "from", "global", "if", "import", "in", "is",
    "lambda", "match", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while", "with",
    "yield",
];
const PY_LITERALS: &[&str] = &["True", "False", "None", "self", "cls"];

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn char_at(text: &str, i: usize) -> Option<char> {
    text.get(i..).and_then(|rest| rest.chars().next())
}

fn char_before(text: &str, i: usize) -> Option<char> {
    text.get(..i).and_then(|head| head.chars().next_back())
}

/// `\b` before `i`: the character before is not a word character.
fn word_start(text: &str, i: usize) -> bool {
    !char_before(text, i).is_some_and(is_word)
}

/// `\b` after `i`: the character at `i` is not a word character.
fn word_end_at(text: &str, i: usize) -> bool {
    !char_at(text, i).is_some_and(is_word)
}

fn char_len_at(text: &str, i: usize) -> usize {
    char_at(text, i).map_or(1, char::len_utf8)
}

fn word_run_end(text: &str, p: usize) -> usize {
    let mut i = p;
    while let Some(c) = char_at(text, i).filter(|c| is_word(*c)) {
        i += c.len_utf8();
    }
    i
}

fn line_end(text: &str, p: usize) -> usize {
    text[p..].find('\n').map_or(text.len(), |offset| p + offset)
}

fn block_comment_end(text: &str, p: usize) -> usize {
    text[p + 2..]
        .find("*/")
        .map_or(text.len(), |offset| p + 2 + offset + 2)
}

/// A quoted run from `start`. Returns the end and whether the closing quote was
/// found. Without `multiline`, a newline ends the run before it. A backslash
/// escapes the next character; in the JS single and double quotes, an escaped
/// newline ends the run.
fn scan_quoted(text: &str, start: usize, quote: u8, multiline: bool) -> (usize, bool) {
    let bytes = text.as_bytes();
    let mut i = start + 1;
    while i < bytes.len() {
        let c = bytes[i];
        if c == quote {
            return (i + 1, true);
        }
        if c == b'\n' && !multiline {
            return (i, false);
        }
        if c == b'\\' {
            match bytes.get(i + 1) {
                Some(&next) if multiline || next != b'\n' => {
                    i += 1 + char_len_at(text, i + 1);
                    continue;
                }
                _ => return (i, false),
            }
        }
        i += 1;
    }
    (text.len(), false)
}

/// Digits, an optional fraction and exponent, an optional suffix, then a word
/// boundary. Hex, binary and octal forms take a `0x`, `0b` or `0o` prefix.
fn number_end(text: &str, p: usize, suffix: Option<u8>) -> Option<usize> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    if bytes[p] == b'0'
        && matches!(
            bytes.get(p + 1),
            Some(b'x' | b'X' | b'b' | b'B' | b'o' | b'O')
        )
    {
        let mut i = p + 2;
        while i < len && (bytes[i].is_ascii_hexdigit() || bytes[i] == b'_') {
            i += 1;
        }
        if i > p + 2 {
            if suffix.is_some_and(|s| bytes.get(i) == Some(&s)) {
                i += 1;
            }
            if word_end_at(text, i) {
                return Some(i);
            }
        }
    }
    let mut i = p;
    while i < len && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
        i += 1;
    }
    if i + 1 < len && bytes[i] == b'.' && bytes[i + 1].is_ascii_digit() {
        i += 1;
        while i < len && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    if i < len && matches!(bytes[i], b'e' | b'E') {
        let mut j = i + 1;
        if j < len && matches!(bytes[j], b'+' | b'-') {
            j += 1;
        }
        if j < len && bytes[j].is_ascii_digit() {
            while j < len && bytes[j].is_ascii_digit() {
                j += 1;
            }
            i = j;
        }
    }
    if suffix.is_some_and(|s| bytes.get(i) == Some(&s)) {
        i += 1;
    }
    word_end_at(text, i).then_some(i)
}

fn followed_by(text: &str, end: usize, wanted: char) -> bool {
    text[end..].trim_start().starts_with(wanted)
}

/// Whether a command word may start at `p`: the first word on its line, or the
/// first after a pipe, `;`, `&`, or `(`.
fn command_position(text: &str, p: usize) -> bool {
    let line_start = text[..p].rfind('\n').map_or(0, |i| i + 1);
    if text[line_start..p].bytes().all(|b| b == b' ' || b == b'\t') {
        return true;
    }
    text[..p]
        .trim_end_matches([' ', '\t'])
        .ends_with(['|', ';', '&', '('])
}

fn js_at(text: &str, p: usize) -> Option<(TokenKind, usize)> {
    let bytes = text.as_bytes();
    let c = bytes[p];
    if c == b'/' && bytes.get(p + 1) == Some(&b'/') {
        return Some((TokenKind::Comment, line_end(text, p)));
    }
    if c == b'/' && bytes.get(p + 1) == Some(&b'*') {
        return Some((TokenKind::Comment, block_comment_end(text, p)));
    }
    if c == b'"' || c == b'\'' {
        return Some((TokenKind::Str, scan_quoted(text, p, c, false).0));
    }
    if c == b'`' {
        return Some((TokenKind::Str, scan_quoted(text, p, c, true).0));
    }
    if c.is_ascii_digit()
        && word_start(text, p)
        && let Some(end) = number_end(text, p, Some(b'n'))
    {
        return Some((TokenKind::Number, end));
    }
    if is_word(c as char) && word_start(text, p) {
        let end = word_run_end(text, p);
        let word = &text[p..end];
        if JS_LITERALS.contains(&word) {
            return Some((TokenKind::Literal, end));
        }
        if JS_KEYWORDS.contains(&word) {
            return Some((TokenKind::Keyword, end));
        }
    }
    if (c.is_ascii_alphabetic() || c == b'_' || c == b'$')
        && (p == 0 || !matches!(char_before(text, p), Some(ch) if is_word(ch) || ch == '$'))
    {
        let end = text[p..]
            .char_indices()
            .find(|(_, ch)| !(is_word(*ch) || *ch == '$'))
            .map_or(text.len(), |(offset, _)| p + offset);
        if followed_by(text, end, '(') {
            return Some((TokenKind::Function, end));
        }
    }
    None
}

fn json_at(text: &str, p: usize) -> Option<(TokenKind, usize)> {
    let bytes = text.as_bytes();
    let c = bytes[p];
    if c == b'/' && bytes.get(p + 1) == Some(&b'/') {
        return Some((TokenKind::Comment, line_end(text, p)));
    }
    if c == b'/' && bytes.get(p + 1) == Some(&b'*') {
        return Some((TokenKind::Comment, block_comment_end(text, p)));
    }
    if c == b'"' {
        let (end, closed) = scan_quoted(text, p, b'"', false);
        let kind = if closed && followed_by(text, end, ':') {
            TokenKind::Key
        } else {
            TokenKind::Str
        };
        return Some((kind, end));
    }
    if c == b'-' || (c.is_ascii_digit() && word_start(text, p)) {
        let digits_at = if c == b'-' { p + 1 } else { p };
        if bytes.get(digits_at).is_some_and(|d| d.is_ascii_digit())
            && let Some(end) = number_end(text, digits_at, None)
        {
            return Some((TokenKind::Number, end));
        }
    }
    if is_word(c as char) && word_start(text, p) {
        let end = word_run_end(text, p);
        if matches!(&text[p..end], "true" | "false" | "null") {
            return Some((TokenKind::Literal, end));
        }
    }
    None
}

fn bash_at(text: &str, p: usize) -> Option<(TokenKind, usize)> {
    let bytes = text.as_bytes();
    let c = bytes[p];
    if c == b'#'
        && char_before(text, p)
            .is_none_or(|ch| ch.is_whitespace() || matches!(ch, ';' | '|' | '&' | '('))
    {
        return Some((TokenKind::Comment, line_end(text, p)));
    }
    if c == b'"' {
        return Some((TokenKind::Str, scan_quoted(text, p, b'"', true).0));
    }
    if c == b'\'' {
        let end = text[p + 1..]
            .find('\'')
            .map_or(text.len(), |offset| p + 1 + offset + 1);
        return Some((TokenKind::Str, end));
    }
    if c == b'$' {
        let next = bytes.get(p + 1).copied();
        match next {
            Some(b'{') => {
                let end = text[p..].find(['}', '\n']).map_or(text.len(), |offset| {
                    if text.as_bytes()[p + offset] == b'}' {
                        p + offset + 1
                    } else {
                        p + offset
                    }
                });
                return Some((TokenKind::Variable, end));
            }
            Some(n) if n.is_ascii_alphabetic() || n == b'_' => {
                let mut end = p + 1;
                while end < bytes.len() && is_word(bytes[end] as char) {
                    end += 1;
                }
                return Some((TokenKind::Variable, end));
            }
            Some(n) if b"0123456789@#?*$!-".contains(&n) => {
                return Some((TokenKind::Variable, p + 2));
            }
            _ => {}
        }
    }
    if is_word(c as char) && word_start(text, p) {
        let end = word_run_end(text, p);
        if BASH_KEYWORDS.contains(&&text[p..end]) {
            return Some((TokenKind::Keyword, end));
        }
    }
    if (c.is_ascii_alphabetic() || matches!(c, b'_' | b'.' | b'/' | b'~'))
        && command_position(text, p)
    {
        let end = text[p..]
            .char_indices()
            .skip(1)
            .find(|(_, ch)| !(is_word(*ch) || matches!(ch, '.' | '/' | '~' | '+' | '-')))
            .map_or(text.len(), |(offset, _)| p + offset);
        if !char_at(text, end).is_some_and(|ch| is_word(ch) || ch == '=') {
            return Some((TokenKind::Function, end));
        }
    }
    None
}

/// The string prefix (`r`, `b`, `u`, `f` and pairs) before a quote at `p`, as a
/// length in bytes. Zero when no prefix is there.
fn python_prefix_len(text: &str, p: usize) -> usize {
    if !word_start(text, p) {
        return 0;
    }
    let letters = |ch: char| matches!(ch, 'r' | 'R' | 'b' | 'B' | 'u' | 'U' | 'f' | 'F');
    for len in [2usize, 1] {
        let prefix: Vec<char> = text[p..].chars().take(len).collect();
        if prefix.len() == len
            && prefix.iter().all(|ch| letters(*ch))
            && matches!(char_at(text, p + len), Some('"' | '\''))
        {
            return len;
        }
    }
    0
}

fn python_at(text: &str, p: usize) -> Option<(TokenKind, usize)> {
    let bytes = text.as_bytes();
    let c = bytes[p];
    if c == b'#' {
        return Some((TokenKind::Comment, line_end(text, p)));
    }
    let prefix = python_prefix_len(text, p);
    let quote_at = p + prefix;
    if matches!(bytes.get(quote_at), Some(b'"' | b'\'')) {
        let quote = bytes[quote_at];
        let closing = if quote == b'"' { "\"\"\"" } else { "'''" };
        if text[quote_at..].starts_with(closing) {
            let body = quote_at + 3;
            let end = text[body..]
                .find(closing)
                .map_or(text.len(), |offset| body + offset + 3);
            return Some((TokenKind::Str, end));
        }
        return Some((TokenKind::Str, scan_quoted(text, quote_at, quote, false).0));
    }
    if c == b'@'
        && bytes
            .get(p + 1)
            .is_some_and(|n| n.is_ascii_alphabetic() || *n == b'_')
    {
        let end = text[p + 1..]
            .char_indices()
            .find(|(_, ch)| !(is_word(*ch) || *ch == '.'))
            .map_or(text.len(), |(offset, _)| p + 1 + offset);
        return Some((TokenKind::Function, end));
    }
    if c.is_ascii_digit()
        && word_start(text, p)
        && let Some(end) = number_end(text, p, Some(b'j'))
    {
        return Some((TokenKind::Number, end));
    }
    if is_word(c as char) && word_start(text, p) {
        let end = word_run_end(text, p);
        let word = &text[p..end];
        if PY_LITERALS.contains(&word) {
            return Some((TokenKind::Literal, end));
        }
        if PY_KEYWORDS.contains(&word) {
            return Some((TokenKind::Keyword, end));
        }
        if followed_by(text, end, '(') {
            return Some((TokenKind::Function, end));
        }
    }
    None
}

fn match_at(lang: Lang, text: &str, p: usize) -> Option<(TokenKind, usize)> {
    match lang {
        Lang::Js => js_at(text, p),
        Lang::Json => json_at(text, p),
        Lang::Bash => bash_at(text, p),
        Lang::Python => python_at(text, p),
    }
}

/// Colored spans in source order. Plain text comes back as `None` spans, with
/// adjacent plain text merged.
pub fn tokenize(text: &str, lang: Option<Lang>) -> Vec<Token> {
    let Some(lang) = lang.filter(|_| !text.is_empty()) else {
        return vec![(None, text.to_string())];
    };
    let mut out: Vec<Token> = Vec::new();
    let mut plain = String::new();
    let mut p = 0;
    while p < text.len() {
        match match_at(lang, text, p) {
            Some((kind, end)) => {
                if !plain.is_empty() {
                    out.push((None, std::mem::take(&mut plain)));
                }
                out.push((Some(kind), text[p..end].to_string()));
                p = end;
            }
            None => {
                let ch = char_at(text, p).unwrap_or_default();
                plain.push(ch);
                p += ch.len_utf8();
            }
        }
    }
    if !plain.is_empty() {
        out.push((None, plain));
    }
    out
}

/// Splits tokens into lines. A token that runs over several lines keeps its color on each.
pub fn split_lines(tokens: &[Token]) -> Vec<Vec<Token>> {
    let mut lines: Vec<Vec<Token>> = vec![Vec::new()];
    for (kind, text) in tokens {
        for (i, part) in text.split('\n').enumerate() {
            if i > 0 {
                lines.push(Vec::new());
            }
            if let (false, Some(line)) = (part.is_empty(), lines.last_mut()) {
                line.push((*kind, part.to_string()));
            }
        }
    }
    lines
}

fn copy_script(text: &str) -> String {
    format!(
        "const text = {text:?};
         try {{ await navigator.clipboard.writeText(text); return true; }} catch (e) {{}}
         const area = document.createElement('textarea');
         area.value = text;
         area.setAttribute('readonly', '');
         area.style.cssText = 'position:fixed;opacity:0;pointer-events:none';
         document.body.appendChild(area);
         area.select();
         let ok = false;
         try {{ ok = document.execCommand('copy'); }} catch (e) {{ ok = false; }}
         area.remove();
         return ok;"
    )
}

#[component]
pub fn CodeBlock(
    /// The code, copied exactly as given.
    code: String,
    /// ts, tsx, js, jsx, json, bash, sh, python. Colors the code when known, otherwise plain text.
    #[props(default, into)]
    language: Option<String>,
    /// Shown in the header, e.g. `agent.config.ts`. Without it the header names the language.
    #[props(default, into)]
    filename: Option<String>,
    /// Numbers in front of every line. They stay in place while the code scrolls sideways.
    #[props(default)]
    show_line_numbers: bool,
    /// Lines to mark, counted from 1.
    #[props(default)]
    highlight_lines: Vec<usize>,
    /// Long lines wrap instead of scrolling sideways.
    #[props(default)]
    wrap: bool,
    /// CSS length after which the code scrolls inside, e.g. `320px` or `40vh`.
    #[props(default, into)]
    max_height: Option<String>,
    #[props(default = true)] copyable: bool,
    /// Called after the code was copied, e.g. for a toast.
    #[props(default)]
    on_copy: Option<EventHandler<String>>,
    #[props(default)] class: String,
    #[props(default)] style: String,
) -> Element {
    let id = use_hook(|| format!("ui-code-{}", next_id()));
    let scroll_id = format!("{id}-scroll");
    let shown = normalise_source(&code);
    let lang = language.as_deref().and_then(language_of);
    let tokens = tokenize(&shown, lang);
    let lines = split_lines(&tokens);
    let label = match (filename.clone(), language.clone()) {
        (Some(name), _) if !name.is_empty() => name,
        (_, Some(name)) => language_name(&name),
        _ => String::new(),
    };
    let header = !label.is_empty() || copyable;
    let digits = line_digits(lines.len());
    let style = if style.is_empty() {
        format!("--ui-code-digits: {digits}")
    } else {
        format!("{style}; --ui-code-digits: {digits}")
    };

    let mut copied = use_signal(|| false);
    let mut copy_ticket = use_signal(|| 0u32);
    let mut scrolls = use_signal(|| false);

    let measure_id = scroll_id.clone();
    use_effect(use_reactive(
        (&shown, &wrap, &max_height, &show_line_numbers),
        move |(_shown, _wrap, _max, _numbers)| {
            let script = format!(
                "const el = document.getElementById({measure_id:?});
                 if (!el) return false;
                 return el.scrollWidth > el.clientWidth + 1 || el.scrollHeight > el.clientHeight + 1;"
            );
            spawn(async move {
                let measured = document::eval(&script)
                    .await
                    .ok()
                    .and_then(|value| value.as_bool());
                scrolls.set(measured.unwrap_or(false));
            });
        },
    ));

    let copy_source = code.clone();
    let on_copy_click = move |_| {
        let source = copy_source.clone();
        let script = copy_script(&source);
        spawn(async move {
            let ok = document::eval(&script)
                .await
                .ok()
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            if !ok {
                return;
            }
            copied.set(true);
            let mine = *copy_ticket.peek() + 1;
            copy_ticket.set(mine);
            if let Some(handler) = on_copy {
                handler.call(source);
            }
            sleep(1400).await;
            if *copy_ticket.peek() == mine {
                copied.set(false);
            }
        });
    };

    let copied_now = copied();
    let copy_label = if copied_now { "Copied" } else { "Copy code" };
    let sr_text = if copied_now { "Copied" } else { "" };
    let max_style = max_height.as_ref().map(|h| format!("max-height: {h}"));
    let scroll_label = format!(
        "{}, code",
        if label.is_empty() {
            "Code"
        } else {
            label.as_str()
        }
    );
    let figure_class = class_list("ui-code", &class);
    let label_title = (!label.is_empty()).then(|| label.clone());
    let scrolling = scrolls();

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        figure {
            class: figure_class,
            "data-wrap": wrap.then_some("true"),
            "data-numbers": show_line_numbers.then_some("true"),
            style,
            if header {
                figcaption { class: "ui-code__head",
                    span { class: "ui-code__label", dir: "auto", title: label_title, "{label}" }
                    if copyable {
                        button {
                            r#type: "button",
                            class: "ui-code__copy",
                            "data-copied": copied_now.then_some("true"),
                            aria_label: "{copy_label}",
                            title: "{copy_label}",
                            onclick: on_copy_click,
                            svg { class: "ui-code__copy-icon", view_box: "0 0 16 16", "aria-hidden": "true",
                                rect { x: "5.5", y: "5.5", width: "8.5", height: "8.5", rx: "2" }
                                path { d: "M10.5 3.75v-.25A1.5 1.5 0 0 0 9 2H3.5A1.5 1.5 0 0 0 2 3.5V9a1.5 1.5 0 0 0 1.5 1.5h.25" }
                            }
                            svg { class: "ui-code__copy-done", view_box: "0 0 16 16", "aria-hidden": "true",
                                path { d: "M3.5 8.4 6.6 11.3 12.5 4.9" }
                            }
                        }
                    }
                }
            }
            if copyable {
                span { class: "ui-code__sr", role: "status", "{sr_text}" }
            }
            div {
                id: "{scroll_id}",
                class: "ui-code__scroll",
                style: max_style,
                tabindex: scrolling.then_some("0"),
                role: scrolling.then_some("region"),
                aria_label: scrolling.then_some(scroll_label),
                pre { class: "ui-code__pre", dir: "ltr",
                    code { class: "ui-code__code",
                        for (i, line) in lines.iter().enumerate() {
                            span {
                                key: "{i}",
                                class: "ui-code__line",
                                "data-marked": highlight_lines.contains(&(i + 1)).then_some("true"),
                                if show_line_numbers {
                                    span { class: "ui-code__ln", "aria-hidden": "true", "{i + 1}" }
                                }
                                span { class: "ui-code__text",
                                    for (j, (kind, text)) in line.iter().enumerate() {
                                        if let Some(kind) = kind {
                                            span { key: "{j}", class: format!("ui-code__t-{}", kind.as_str()), "{text}" }
                                        } else {
                                            "{text}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
