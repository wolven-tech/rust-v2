//! `rv2-isa`: look up and enforce the `rv2-ui` instruction set from any
//! repository. `rv2-isa help` lists the commands.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use rv2_isa::index::{self, Index, Mode};
use rv2_isa::isa;

/// The index as of the commit this binary was built from, for repositories
/// with no copy of the kit's source.
const EMBEDDED: &str = include_str!("../../../docs/design-system/kit-index.json");

const USAGE: &str = "\
usage: rv2-isa <command>

  find <words…>        instructions for an intent, best first
  show <Name>          an instruction's props, example, tests and operands
  list                 every intent and its instructions
  check [--root DIR] [--src DIR]…
                       UI7: fail on a raw button, input, select, textarea,
                       h1–h4 or a in DIR/apps/*/src (and each --src)
  index --write|--check [--root DIR]
                       regenerate docs/design-system/kit-index.{md,json}
                       (inside an rv2 workspace only)

find, show and list read the kit's source when the current directory is
inside a workspace with crates/rv2-ui, so a fork's own copy is described;
elsewhere, or with --embedded, they use the index built into this binary.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("\nfailed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> index::Fallible<()> {
    let embedded = args.iter().any(|a| a == "--embedded");
    let args: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| *a != "--embedded")
        .collect();
    match args.as_slice() {
        ["find", words @ ..] if !words.is_empty() => find(&load(embedded)?, &words.join(" ")),
        ["show", name] => show(&load(embedded)?, name),
        ["list"] => {
            list(&load(embedded)?);
            Ok(())
        }
        ["check", rest @ ..] => check(rest),
        ["index", rest @ ..] => index_command(rest),
        ["help" | "--help" | "-h"] => {
            println!("{USAGE}");
            Ok(())
        }
        _ => Err(USAGE.into()),
    }
}

/// The nearest ancestor of the current directory holding the kit's source.
fn workspace_with_kit() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    cwd.ancestors()
        .find(|dir| dir.join(index::KIT_SRC).join("lib.rs").is_file())
        .map(Path::to_path_buf)
}

fn load(embedded: bool) -> index::Fallible<Index> {
    if !embedded && let Some(root) = workspace_with_kit() {
        eprintln!("index: built from {}", root.join(index::KIT_SRC).display());
        return index::build(&root, Mode::Lenient);
    }
    eprintln!("index: embedded in rv2-isa {}", env!("CARGO_PKG_VERSION"));
    Index::from_json(EMBEDDED)
}

fn find(index: &Index, query: &str) -> index::Fallible<()> {
    let hits = index.search(query);
    if hits.is_empty() {
        return Err(format!(
            "no instruction matches \"{query}\"\n         `rv2-isa list` shows every intent"
        )
        .into());
    }
    for component in hits.iter().take(8) {
        let summary = component.docs.split("\n\n").next().unwrap_or_default();
        let summary: String = summary.split_whitespace().collect::<Vec<_>>().join(" ");
        println!(
            "{:<20} {}\n{:<20} {}\n",
            component.name,
            component.intent.as_deref().unwrap_or("—"),
            "",
            summary
        );
    }
    println!("`rv2-isa show <Name>` for props and an example");
    Ok(())
}

fn show(index: &Index, name: &str) -> index::Fallible<()> {
    if let Some(component) = index.component(name) {
        print!("{}", index::component_markdown(component, 1));
        let used = index.types_used_by(component);
        if !used.is_empty() {
            println!("## Operands and parts it takes\n");
            for ty in used {
                println!("{}", index::type_markdown(ty, 3));
            }
        }
        return Ok(());
    }
    if let Some(ty) = index.type_doc(name) {
        print!("{}", index::type_markdown(ty, 1));
        return Ok(());
    }
    let near: Vec<&str> = index
        .search(name)
        .iter()
        .take(5)
        .map(|c| c.name.as_str())
        .collect();
    Err(format!(
        "rv2-ui has no `{name}`{}",
        if near.is_empty() {
            String::new()
        } else {
            format!("; nearest: {}", near.join(", "))
        }
    )
    .into())
}

fn list(index: &Index) {
    for row in &index.intents {
        println!("{:<34} {}", row.intent, row.instructions.join(", "));
    }
}

/// `--flag value` pairs, repeated flags kept in order.
fn flag_values<'a>(args: &[&'a str], flag: &str) -> index::Fallible<Vec<&'a str>> {
    let mut values = Vec::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if *arg == flag {
            values.push(*iter.next().ok_or(format!("{flag} needs a value"))?);
        }
    }
    Ok(values)
}

fn check(args: &[&str]) -> index::Fallible<()> {
    let root = PathBuf::from(flag_values(args, "--root")?.last().copied().unwrap_or("."));
    let mut dirs = isa::app_sources(&root)?;
    for src in flag_values(args, "--src")? {
        dirs.push(root.join(src));
    }
    if dirs.is_empty() {
        return Err(format!(
            "nothing to scan: {} has no apps/*/src; name directories with --src",
            root.display()
        )
        .into());
    }
    let (findings, files) = isa::scan_dirs(&root, &dirs)?;
    isa::report(&findings, files)
}

fn index_command(args: &[&str]) -> index::Fallible<()> {
    let root = match flag_values(args, "--root")?.last() {
        Some(root) => PathBuf::from(root),
        None => workspace_with_kit().ok_or("not inside a workspace with crates/rv2-ui")?,
    };
    if args.contains(&"--write") {
        let written = index::sync(&root, true)?;
        if written.is_empty() {
            println!("component index already current");
        } else {
            println!("wrote {}", written.join(", "));
        }
        Ok(())
    } else if args.contains(&"--check") {
        index::check(&root)
    } else {
        Err("index needs --write or --check".into())
    }
}
