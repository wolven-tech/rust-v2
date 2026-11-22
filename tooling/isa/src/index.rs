//! The component index: each `rv2-ui` instruction with the intent it serves,
//! its props, a working example from `/kit`, and the behaviour its tests pin.
//!
//! Generated, never written by hand, so it cannot disagree with the kit:
//! `cargo xtask ci` regenerates it and fails when the committed copy differs.
//! Its sources are the kit's own code, the instruction table in
//! `docs/architecture/003-ui-semantic-isa.md`, the `/kit` fixtures and the
//! kit's tests.

use std::collections::{BTreeSet, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use serde::{Deserialize, Serialize};

pub const MARKDOWN: &str = "docs/design-system/kit-index.md";
pub const JSON: &str = "docs/design-system/kit-index.json";
pub const KIT_SRC: &str = "crates/rv2-ui/src";
const KIT_TESTS: &str = "crates/rv2-ui/tests";
const ISA_DOC: &str = "docs/architecture/003-ui-semantic-isa.md";
/// Searched before any other app source, so an example comes from the page
/// that exists to mount every instruction.
const FIXTURES: &[&str] = &[
    "apps/web/src/kit_catalogue.rs",
    "apps/web/src/kit_records.rs",
];
/// Modules outside the instruction set (UI9).
const OUTSIDE_ISA: &[&str] = &["motion"];
/// An example longer than this is a page, not an example.
const EXAMPLE_LINES: usize = 30;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Index {
    pub intents: Vec<Intent>,
    pub components: Vec<Component>,
    pub types: Vec<TypeDoc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Intent {
    pub intent: String,
    pub instructions: Vec<String>,
    pub emits: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub name: String,
    pub module: String,
    pub in_isa: bool,
    pub intent: Option<String>,
    pub emits: Option<String>,
    pub docs: String,
    pub props: Vec<Prop>,
    pub example: Option<Example>,
    pub guarantees: Vec<Guarantee>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Prop {
    pub name: String,
    pub ty: String,
    pub required: bool,
    /// The `#[props(default = …)]` expression; `"Default"` for a bare
    /// `#[props(default)]`.
    pub default: Option<String>,
    pub docs: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Example {
    pub file: String,
    pub line: usize,
    pub code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Guarantee {
    pub test: String,
    pub file: String,
    pub docs: String,
}

/// An operand, a part passed to an instruction, or a helper function.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TypeDoc {
    pub name: String,
    pub module: String,
    /// `enum`, `struct` or `fn`.
    pub kind: String,
    pub docs: String,
    pub variants: Vec<String>,
    /// Public constructors and methods, or the function's own signature.
    pub signatures: Vec<String>,
}

pub type Fallible<T> = Result<T, Box<dyn std::error::Error>>;

/// How strictly to read the sources.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// This repository: every source must exist and every public instruction
    /// must have an intent row.
    Strict,
    /// A fork carrying its own copy of the kit: read what is there.
    Lenient,
}

/// Build the index from the sources under `root`.
pub fn build(root: &Path, mode: Mode) -> Fallible<Index> {
    let mut components = Vec::new();
    let mut types = Vec::new();
    for file in sorted_rust_files(&root.join(KIT_SRC))? {
        let module = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        if module == "lib" {
            continue;
        }
        let parsed = syn::parse_file(&std::fs::read_to_string(&file)?)
            .map_err(|e| format!("{}: {e}", file.display()))?;
        read_items(&module, &parsed.items, &mut components, &mut types);
    }

    let intents = match std::fs::read_to_string(root.join(ISA_DOC)) {
        Ok(doc) => intents(&doc),
        Err(_) if mode == Mode::Lenient => Vec::new(),
        Err(e) => return Err(format!("{ISA_DOC}: {e}").into()),
    };
    for component in &mut components {
        if let Some(row) = intents
            .iter()
            .find(|row| row.instructions.contains(&component.name))
        {
            component.intent = Some(row.intent.clone());
            component.emits = Some(row.emits.clone());
        }
    }

    let examples = example_sources(root)?;
    let scopes: Vec<KitScope> = examples
        .iter()
        .map(|(_, source)| KitScope::of(source))
        .collect();
    for component in &mut components {
        component.example = examples
            .iter()
            .zip(&scopes)
            .filter(|(_, scope)| scope.binds(&component.name))
            .find_map(|((file, source), _)| example(file, source, &component.name));
    }

    let names: HashSet<String> = components.iter().map(|c| c.name.clone()).collect();
    let tests_dir = root.join(KIT_TESTS);
    if tests_dir.is_dir() {
        for file in sorted_rust_files(&tests_dir)? {
            let shown = file
                .strip_prefix(root)
                .unwrap_or(&file)
                .display()
                .to_string();
            let parsed = syn::parse_file(&std::fs::read_to_string(&file)?)
                .map_err(|e| format!("{shown}: {e}"))?;
            for (test, docs, mentioned) in tests(&parsed, &names) {
                for name in mentioned {
                    if let Some(component) = components.iter_mut().find(|c| c.name == name) {
                        component.guarantees.push(Guarantee {
                            test: test.clone(),
                            file: shown.clone(),
                            docs: docs.clone(),
                        });
                    }
                }
            }
        }
    }

    order(&intents, &mut components);
    types.sort_by(|a, b| a.name.cmp(&b.name));
    let index = Index {
        intents,
        components,
        types,
    };

    if mode == Mode::Strict {
        let problems = index.problems();
        if !problems.is_empty() {
            return Err(format!(
                "the instruction table and the kit disagree:\n  {}\n         \
                 add an intent row to the Instructions table in {ISA_DOC}, or remove the name",
                problems.join("\n  ")
            )
            .into());
        }
    }
    Ok(index)
}

fn sorted_rust_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    crate::isa::rust_files(dir, &mut files)?;
    files.sort();
    Ok(files)
}

/// Intent-table order first, then module and name, so related instructions
/// sit together and the output is stable.
fn order(intents: &[Intent], components: &mut [Component]) {
    let rank = |c: &Component| {
        intents
            .iter()
            .enumerate()
            .find_map(|(i, row)| {
                row.instructions
                    .iter()
                    .position(|n| *n == c.name)
                    .map(|j| (i, j))
            })
            .unwrap_or((usize::MAX, 0))
    };
    components.sort_by(|a, b| (rank(a), &a.module, &a.name).cmp(&(rank(b), &b.module, &b.name)));
}

fn read_items(
    module: &str,
    items: &[syn::Item],
    components: &mut Vec<Component>,
    types: &mut Vec<TypeDoc>,
) {
    let public = |vis: &syn::Visibility| matches!(vis, syn::Visibility::Public(_));
    for item in items {
        match item {
            syn::Item::Fn(f) if public(&f.vis) && has_attr(&f.attrs, "component") => {
                components.push(component(module, f));
            }
            syn::Item::Fn(f) if public(&f.vis) => types.push(TypeDoc {
                name: f.sig.ident.to_string(),
                module: module.to_string(),
                kind: "fn".to_string(),
                docs: docs(&f.attrs),
                variants: Vec::new(),
                signatures: vec![tidy(&f.sig.to_token_stream())],
            }),
            syn::Item::Enum(e) if public(&e.vis) => types.push(TypeDoc {
                name: e.ident.to_string(),
                module: module.to_string(),
                kind: "enum".to_string(),
                docs: docs(&e.attrs),
                variants: e
                    .variants
                    .iter()
                    .map(|v| {
                        let line = first_line(&docs(&v.attrs));
                        if line.is_empty() {
                            v.ident.to_string()
                        } else {
                            format!("{} — {line}", v.ident)
                        }
                    })
                    .collect(),
                signatures: Vec::new(),
            }),
            syn::Item::Struct(s) if public(&s.vis) => types.push(TypeDoc {
                name: s.ident.to_string(),
                module: module.to_string(),
                kind: "struct".to_string(),
                docs: docs(&s.attrs),
                variants: Vec::new(),
                signatures: Vec::new(),
            }),
            _ => {}
        }
    }
    // Methods second: an `impl` may precede the type it implements.
    for item in items {
        let syn::Item::Impl(block) = item else {
            continue;
        };
        let syn::Type::Path(path) = &*block.self_ty else {
            continue;
        };
        let Some(owner) = path.path.segments.last().map(|s| s.ident.to_string()) else {
            continue;
        };
        let Some(ty) = types
            .iter_mut()
            .find(|t| t.name == owner && t.module == module)
        else {
            continue;
        };
        if let Some((_, trait_path, _)) = &block.trait_ {
            if trait_path
                .segments
                .last()
                .is_some_and(|s| s.ident == "From")
            {
                let generics = &block.generics;
                let bounds = generics
                    .where_clause
                    .as_ref()
                    .map(|w| format!(" {}", tidy(&w.to_token_stream())))
                    .unwrap_or_default();
                ty.signatures.push(format!(
                    "impl{} {} for {owner}{bounds}",
                    tidy(&generics.to_token_stream()),
                    tidy(&trait_path.to_token_stream())
                ));
            }
            continue;
        }
        for member in &block.items {
            if let syn::ImplItem::Fn(f) = member
                && matches!(f.vis, syn::Visibility::Public(_))
            {
                ty.signatures.push(tidy(&f.sig.to_token_stream()));
            }
        }
    }
}

fn component(module: &str, f: &syn::ItemFn) -> Component {
    let props = f
        .sig
        .inputs
        .iter()
        .filter_map(|input| {
            let syn::FnArg::Typed(arg) = input else {
                return None;
            };
            let syn::Pat::Ident(ident) = &*arg.pat else {
                return None;
            };
            let name = ident.ident.to_string();
            let name = name.strip_prefix("r#").unwrap_or(&name).to_string();
            let ty = tidy(&arg.ty.to_token_stream());
            let default = arg.attrs.iter().find_map(props_default);
            let required = default.is_none() && name != "children" && !ty.starts_with("Option<");
            Some(Prop {
                name,
                ty,
                required,
                default,
                docs: docs(&arg.attrs),
            })
        })
        .collect();
    Component {
        name: f.sig.ident.to_string(),
        module: module.to_string(),
        in_isa: !OUTSIDE_ISA.contains(&module),
        intent: None,
        emits: None,
        docs: docs(&f.attrs),
        props,
        example: None,
        guarantees: Vec::new(),
    }
}

/// `#[props(default)]` → `"Default"`, `#[props(default = x)]` → `"x"`.
fn props_default(attr: &syn::Attribute) -> Option<String> {
    if !attr.path().is_ident("props") {
        return None;
    }
    let syn::Meta::List(list) = &attr.meta else {
        return None;
    };
    let tokens: Vec<TokenTree> = list.tokens.clone().into_iter().collect();
    let at = tokens
        .iter()
        .position(|t| matches!(t, TokenTree::Ident(i) if i == "default"))?;
    match tokens.get(at + 1) {
        Some(TokenTree::Punct(p)) if p.as_char() == '=' => {
            let value: TokenStream = tokens[at + 2..]
                .iter()
                .take_while(|t| !matches!(t, TokenTree::Punct(p) if p.as_char() == ','))
                .cloned()
                .collect();
            Some(tidy(&value))
        }
        _ => Some("Default".to_string()),
    }
}

fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|a| a.path().is_ident(name))
}

fn docs(attrs: &[syn::Attribute]) -> String {
    let lines: Vec<String> = attrs
        .iter()
        .filter_map(|a| match &a.meta {
            syn::Meta::NameValue(nv) if nv.path.is_ident("doc") => match &nv.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) => Some(s.value()),
                _ => None,
            },
            _ => None,
        })
        .map(|line| line.strip_prefix(' ').unwrap_or(&line).to_string())
        .collect();
    lines.join("\n").trim().to_string()
}

fn first_line(text: &str) -> String {
    text.split("\n\n")
        .next()
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Token text with the spaces `quote` puts around punctuation removed.
fn tidy(tokens: &TokenStream) -> String {
    let mut text = tokens.to_string();
    for (from, to) in [
        (" < ", "<"),
        ("< ", "<"),
        (" <", "<"),
        (" >", ">"),
        (" ::", "::"),
        (":: ", "::"),
        (" ,", ","),
        ("& ", "&"),
        ("( ", "("),
        (" (", "("),
        (" )", ")"),
        (" :", ":"),
        ("# [", "#["),
        (" ;", ";"),
        ("[ ", "["),
        (" ]", "]"),
        (" . ", "."),
        (" !", "!"),
        ("' static", "'static"),
    ] {
        while text.contains(from) {
            text = text.replace(from, to);
        }
    }
    text.replace("->", " -> ").replace("  ", " ")
}

/// The rows of the Instructions table in the ISA doc.
pub fn intents(doc: &str) -> Vec<Intent> {
    let Some((_, section)) = doc.split_once("\n## Instructions\n") else {
        return Vec::new();
    };
    section
        .lines()
        .skip_while(|l| !l.starts_with('|'))
        .take_while(|l| l.starts_with('|'))
        .skip(2)
        .filter_map(|row| {
            let cells: Vec<&str> = row.trim_matches('|').split('|').map(str::trim).collect();
            let [intent, instructions, emits, ..] = cells.as_slice() else {
                return None;
            };
            Some(Intent {
                intent: (*intent).to_string(),
                instructions: code_spans(instructions)
                    .into_iter()
                    .filter(|n| {
                        n.starts_with(|c: char| c.is_ascii_uppercase()) || n.starts_with("use_")
                    })
                    .collect(),
                emits: (*emits).to_string(),
            })
        })
        .collect()
}

fn code_spans(text: &str) -> Vec<String> {
    text.split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

fn example_sources(root: &Path) -> Fallible<Vec<(String, String)>> {
    let mut files: Vec<PathBuf> = FIXTURES.iter().map(|f| root.join(f)).collect();
    let apps = root.join("apps");
    if apps.is_dir() {
        let mut rest = Vec::new();
        for app in std::fs::read_dir(apps)? {
            let app = app?.path();
            for sub in ["src", "examples"] {
                if app.join(sub).is_dir() {
                    crate::isa::rust_files(&app.join(sub), &mut rest)?;
                }
            }
        }
        rest.sort();
        rest.retain(|f| !files.contains(f));
        files.extend(rest);
    }
    // Last resort for an instruction no app mounts: the kit's tests render it.
    let tests = root.join(KIT_TESTS);
    if tests.is_dir() {
        files.extend(sorted_rust_files(&tests)?);
    }
    let mut sources = Vec::new();
    for file in files {
        if let Ok(source) = std::fs::read_to_string(&file) {
            let shown = file
                .strip_prefix(root)
                .unwrap_or(&file)
                .display()
                .to_string();
            sources.push((shown, source));
        }
    }
    Ok(sources)
}

/// The names a source file's `use` items bring in from `rv2_ui`, and those
/// they bring in from anywhere else. `rv2-affect` exports components under the
/// same names as `rv2-ui` (`TextArea`, `Select`, `Popover`), so a match on the
/// name alone shows another kit's props as this one's.
#[derive(Debug, Default)]
pub struct KitScope {
    kit: HashSet<String>,
    kit_glob: bool,
    other: HashSet<String>,
}

impl KitScope {
    /// A file that does not parse binds nothing.
    pub fn of(source: &str) -> Self {
        let mut scope = Self::default();
        if let Ok(file) = syn::parse_file(source) {
            scope.items(&file.items);
        }
        scope
    }

    pub fn binds(&self, name: &str) -> bool {
        !self.other.contains(name) && (self.kit.contains(name) || self.kit_glob)
    }

    fn items(&mut self, items: &[syn::Item]) {
        for item in items {
            match item {
                syn::Item::Use(item) => self.tree(&item.tree, None),
                syn::Item::Mod(module) => {
                    if let Some((_, items)) = &module.content {
                        self.items(items);
                    }
                }
                _ => {}
            }
        }
    }

    fn tree(&mut self, tree: &syn::UseTree, root: Option<&str>) {
        let from_kit = root == Some("rv2_ui");
        match tree {
            syn::UseTree::Path(path) => {
                let segment = path.ident.to_string();
                self.tree(&path.tree, Some(root.unwrap_or(&segment)));
            }
            syn::UseTree::Name(name) => {
                let name = name.ident.to_string();
                if from_kit {
                    self.kit.insert(name);
                } else {
                    self.other.insert(name);
                }
            }
            syn::UseTree::Rename(rename) => {
                self.other.insert(rename.rename.to_string());
            }
            syn::UseTree::Glob(_) => self.kit_glob |= from_kit,
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.tree(tree, root);
                }
            }
        }
    }
}

/// The first use of `name` as an RSX element in `source` that fits in
/// [`EXAMPLE_LINES`], dedented.
pub fn example(file: &str, source: &str, name: &str) -> Option<Example> {
    let bytes = source.as_bytes();
    let mut from = 0;
    while let Some(offset) = source[from..].find(name) {
        let start = from + offset;
        from = start + name.len();

        let ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        if start > 0 && (ident(bytes[start - 1]) || bytes[start - 1] == b':') {
            continue;
        }
        let after = &source[from..];
        if after.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
            || !after.trim_start_matches([' ', '\t']).starts_with('{')
        {
            continue;
        }
        let line_start = source[..start].rfind('\n').map_or(0, |i| i + 1);
        let lead = source[line_start..start].trim();
        if lead.starts_with("//")
            || ["struct", "enum", "impl", "fn", "for", "mod", "use"]
                .iter()
                .any(|kw| lead.split_whitespace().next_back() == Some(kw))
        {
            continue;
        }
        let open = from + (after.len() - after.trim_start_matches([' ', '\t']).len());
        let Some(close) = matching_brace(source, open) else {
            continue;
        };
        let indent = start - line_start;
        let code: Vec<&str> = source[start..=close].lines().collect();
        if code.len() > EXAMPLE_LINES {
            continue;
        }
        let code = code
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if i == 0 {
                    (*line).to_string()
                } else {
                    let cut = line.len() - line.trim_start().len();
                    line[cut.min(indent)..].to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        return Some(Example {
            file: file.to_string(),
            line: source[..start].matches('\n').count() + 1,
            code,
        });
    }
    None
}

/// The `}` closing the `{` at `open`, skipping braces inside string literals.
fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                let raw_hashes = raw_string_hashes(bytes, i);
                i += 1;
                loop {
                    if i >= bytes.len() {
                        return None;
                    }
                    match (raw_hashes, bytes[i]) {
                        (None, b'\\') => i += 1,
                        (None, b'"') => break,
                        (Some(n), b'"') if bytes[i + 1..].iter().take(n).all(|&b| b == b'#') => {
                            i += n;
                            break;
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// `Some(n)` when the quote at `quote` opens a raw string `r#…"` with `n`
/// hashes.
fn raw_string_hashes(bytes: &[u8], quote: usize) -> Option<usize> {
    let mut j = quote;
    let mut hashes = 0;
    while j > 0 && bytes[j - 1] == b'#' {
        hashes += 1;
        j -= 1;
    }
    (j > 0 && bytes[j - 1] == b'r').then_some(hashes)
}

/// Each `#[test]` in a file, with its doc comment and the instructions its
/// body names.
fn tests(file: &syn::File, names: &HashSet<String>) -> Vec<(String, String, BTreeSet<String>)> {
    file.items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Fn(f) if has_attr(&f.attrs, "test") => {
                let mut mentioned = BTreeSet::new();
                idents(f.block.to_token_stream(), names, &mut mentioned);
                Some((f.sig.ident.to_string(), docs(&f.attrs), mentioned))
            }
            _ => None,
        })
        .collect()
}

fn idents(tokens: TokenStream, names: &HashSet<String>, into: &mut BTreeSet<String>) {
    for token in tokens {
        match token {
            TokenTree::Ident(ident) => {
                let ident = ident.to_string();
                if names.contains(&ident) {
                    into.insert(ident);
                }
            }
            TokenTree::Group(group) => idents(group.stream(), names, into),
            _ => {}
        }
    }
}

impl Index {
    /// Disagreements between the instruction table and the kit.
    pub fn problems(&self) -> Vec<String> {
        let known: HashSet<&str> = self
            .components
            .iter()
            .map(|c| c.name.as_str())
            .chain(self.types.iter().map(|t| t.name.as_str()))
            .collect();
        let mut problems: Vec<String> = self
            .components
            .iter()
            .filter(|c| c.in_isa && c.intent.is_none())
            .map(|c| format!("`{}` ({}) has no intent row", c.name, c.module))
            .collect();
        for row in &self.intents {
            for name in &row.instructions {
                if !known.contains(name.as_str()) {
                    problems.push(format!(
                        "intent \"{}\" names `{name}`, which rv2-ui does not export",
                        row.intent
                    ));
                }
            }
        }
        problems
    }

    pub fn component(&self, name: &str) -> Option<&Component> {
        self.components
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
    }

    pub fn type_doc(&self, name: &str) -> Option<&TypeDoc> {
        self.types
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(name))
    }

    /// Instructions ranked against `query`, best first. An intent or name
    /// match outweighs a match in the prose, so "delete" finds
    /// `ArmedDeleteButton` before a component that mentions deleting.
    pub fn search(&self, query: &str) -> Vec<&Component> {
        let words: Vec<String> = query
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 1)
            .map(stem)
            .collect();
        if words.is_empty() {
            return Vec::new();
        }
        let hits = |text: &str| -> usize {
            let text: Vec<String> = text
                .split(|c: char| !c.is_alphanumeric())
                .flat_map(split_camel)
                .map(|w| stem(&w))
                .collect();
            words.iter().filter(|w| text.contains(w)).count()
        };
        let mut scored: Vec<(usize, &Component)> = self
            .components
            .iter()
            .filter(|c| c.in_isa)
            .map(|c| {
                let guarantees: String = c
                    .guarantees
                    .iter()
                    .map(|g| format!("{} ", g.test))
                    .collect();
                let score = 6 * hits(&c.name)
                    + 5 * hits(c.intent.as_deref().unwrap_or_default())
                    + 3 * hits(&first_line(&c.docs))
                    + 2 * hits(c.emits.as_deref().unwrap_or_default())
                    + hits(&c.docs)
                    + hits(&guarantees.replace('_', " "));
                (score, c)
            })
            .filter(|(score, _)| *score > 0)
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
        scored.into_iter().map(|(_, c)| c).collect()
    }

    /// The operands and parts a component's props name, and those their
    /// constructors take, so `show` prints how to build everything it needs.
    pub fn types_used_by(&self, component: &Component) -> Vec<&TypeDoc> {
        let names = |text: &str, t: &TypeDoc| {
            text.split(|c: char| !c.is_alphanumeric() && c != '_')
                .any(|w| w == t.name)
        };
        let types = || self.types.iter().filter(|t| t.kind != "fn");
        let direct: Vec<&TypeDoc> = types()
            .filter(|t| component.props.iter().any(|p| names(&p.ty, t)))
            .collect();
        let mut used = direct.clone();
        for t in types() {
            let reached = direct
                .iter()
                .any(|d| d.signatures.iter().any(|s| names(s, t)));
            if reached && !used.iter().any(|u| u.name == t.name) {
                used.push(t);
            }
        }
        used
    }

    pub fn to_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self).expect("the index serializes");
        json.push('\n');
        json
    }

    pub fn from_json(json: &str) -> Fallible<Self> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str(HEADER);

        out.push_str("\n## By intent\n\n| Intent | Instructions | Emits |\n|---|---|---|\n");
        for row in &self.intents {
            let names: Vec<String> = row
                .instructions
                .iter()
                .map(|n| {
                    if self.component(n).is_some() {
                        format!("[`{n}`](#{})", n.to_lowercase())
                    } else {
                        format!("`{n}`")
                    }
                })
                .collect();
            let _ = writeln!(
                out,
                "| {} | {} | {} |",
                row.intent,
                names.join(", "),
                row.emits
            );
        }

        out.push_str("\n## Instructions\n");
        for component in self.components.iter().filter(|c| c.in_isa) {
            out.push('\n');
            out.push_str(&component_markdown(component, 3));
        }

        out.push_str(
            "\n## Outside the instruction set\n\n`rv2_ui::motion` is a showcase (UI9). \
             No app may depend on it to carry meaning.\n\n",
        );
        for component in self.components.iter().filter(|c| !c.in_isa) {
            let _ = writeln!(
                out,
                "- `{}`: {}",
                component.name,
                first_line(&component.docs)
            );
        }

        out.push_str("\n## Operands, parts and helpers\n");
        for ty in &self.types {
            out.push('\n');
            out.push_str(&type_markdown(ty, 3));
        }
        out
    }
}

const HEADER: &str = "# rv2-ui component index

Generated by `rv2-isa index --write`; do not edit. `cargo xtask ci` fails when
this file differs from what the generator writes. Its sources are
`crates/rv2-ui/src`, the Instructions table in
[`../architecture/003-ui-semantic-isa.md`](../architecture/003-ui-semantic-isa.md),
the `/kit` fixtures in `apps/web/src` and the tests in `crates/rv2-ui/tests`.

Each instruction lists its props, the first example of it in the apps (from
`/kit` where one exists), and the tests that pin its behaviour. The same data
is in [`kit-index.json`](kit-index.json), and from any repository:

```sh
cargo install --git https://github.com/wolven-tech/rust-v2 rv2-isa
rv2-isa find \"delete a row\"   # instructions for an intent
rv2-isa show ConfirmDialog      # props, example, guarantees, operands
rv2-isa check                   # UI7 over apps/*/src
```
";

fn cell(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "\\|")
}

/// Doc-comment headings demoted below `level`, so a component's own `##`
/// cannot outrank the section it sits in.
fn demote(docs: &str, level: usize) -> String {
    docs.lines()
        .map(|line| {
            if line.starts_with('#') {
                format!("{}{line}", "#".repeat(level))
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn component_markdown(c: &Component, level: usize) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{} {}\n", "#".repeat(level), c.name);
    let mut facts = vec![
        format!("`rv2_ui::{}`", c.name),
        format!("module `{}`", c.module),
    ];
    if let Some(intent) = &c.intent {
        facts.push(format!("intent: {intent}"));
    }
    if let Some(emits) = &c.emits {
        facts.push(format!("emits {emits}"));
    }
    let _ = writeln!(out, "{}\n", facts.join(" · "));
    if !c.docs.is_empty() {
        let _ = writeln!(out, "{}\n", demote(&c.docs, level));
    }
    if !c.props.is_empty() {
        out.push_str("| Prop | Type | Default | Notes |\n|---|---|---|---|\n");
        for p in &c.props {
            let default = match (&p.default, p.required) {
                (Some(d), _) => format!("`{}`", cell(d)),
                (None, true) => "required".to_string(),
                (None, false) => "—".to_string(),
            };
            let _ = writeln!(
                out,
                "| `{}` | `{}` | {} | {} |",
                p.name,
                cell(&p.ty),
                default,
                cell(&p.docs)
            );
        }
        out.push('\n');
    }
    if let Some(example) = &c.example {
        let _ = writeln!(
            out,
            "Example, `{}:{}`:\n\n```rust\n{}\n```\n",
            example.file, example.line, example.code
        );
    }
    if !c.guarantees.is_empty() {
        out.push_str("Pinned by tests:\n\n");
        for g in &c.guarantees {
            let _ = writeln!(out, "- {} (`{}`)", g.test.replace('_', " "), g.file);
        }
        out.push('\n');
    }
    out
}

pub fn type_markdown(t: &TypeDoc, level: usize) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}\n\n{} in `{}`\n",
        "#".repeat(level),
        t.name,
        t.kind,
        t.module
    );
    if !t.docs.is_empty() {
        let _ = writeln!(out, "{}\n", demote(&t.docs, level));
    }
    for variant in &t.variants {
        let _ = match variant.split_once(" — ") {
            Some((name, doc)) => writeln!(out, "- `{name}` — {doc}"),
            None => writeln!(out, "- `{variant}`"),
        };
    }
    if !t.variants.is_empty() {
        out.push('\n');
    }
    if !t.signatures.is_empty() {
        out.push_str("```rust\n");
        for s in &t.signatures {
            let _ = writeln!(out, "{s}");
        }
        out.push_str("```\n");
    }
    out
}

/// Words split at lower-to-upper boundaries, so `ArmedDeleteButton` matches
/// "delete".
fn split_camel(word: &str) -> Vec<String> {
    let mut parts = vec![word.to_lowercase()];
    let mut current = String::new();
    for c in word.chars() {
        if c.is_uppercase() && !current.is_empty() {
            parts.push(current.to_lowercase());
            current.clear();
        }
        current.push(c);
    }
    if !current.is_empty() && current.len() != word.len() {
        parts.push(current.to_lowercase());
    }
    parts
}

fn stem(word: &str) -> String {
    let word = word.to_lowercase();
    for suffix in ["ing", "es", "s", "ed"] {
        if let Some(root) = word.strip_suffix(suffix)
            && root.len() >= 3
        {
            return root.to_string();
        }
    }
    word
}

/// Regenerate both files under `root` and report which differ from what is
/// committed. Writes when `write` is set; otherwise only compares.
pub fn sync(root: &Path, write: bool) -> Fallible<Vec<String>> {
    let index = build(root, Mode::Strict)?;
    let mut stale = Vec::new();
    for (path, content) in [(MARKDOWN, index.to_markdown()), (JSON, index.to_json())] {
        let target = root.join(path);
        if std::fs::read_to_string(&target).ok().as_deref() != Some(content.as_str()) {
            stale.push(path.to_string());
            if write {
                std::fs::write(&target, content)?;
            }
        }
    }
    Ok(stale)
}

/// The gate: fail when the committed index is not what the sources produce.
pub fn check(root: &Path) -> Fallible<()> {
    let stale = sync(root, false)?;
    if !stale.is_empty() {
        return Err(format!(
            "stale component index: {}\n         run `cargo run -p rv2-isa -- index --write` and commit the result",
            stale.join(", ")
        )
        .into());
    }
    println!("component index matches the kit");
    Ok(())
}
