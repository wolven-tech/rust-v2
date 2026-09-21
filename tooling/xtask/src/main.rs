//! Development commands.
//!
//! ## Why this exists
//!
//! Before it, "will CI pass?" took six commands — `cargo fmt`, `cargo clippy`,
//! `cargo test`, a wasm cross-compile, a `cargo tree` assertion and two greps —
//! and the honest answer for most people was "run it and find out". A
//! contributor could get `meta test` green and still be four failures away from
//! a green pipeline.
//!
//! CI now runs `cargo xtask ci`. So does a developer. They cannot drift, because
//! there is nothing to keep in sync: it is one code path with one caller
//! signature, not a shell script duplicated into YAML.
//!
//! ## Commands
//!
//! - `cargo xtask ci` — everything CI's `check` and `wasm32 boundary` jobs run.
//! - `cargo xtask styles` — compile each app's Tailwind stylesheet.
//! - `cargo xtask live` — run the tests that need a live Core.
//! - `cargo xtask core` — run AllSource Core with the dev settings.
//! - `cargo xtask stage-web` — validate and stage an SSG release bundle.
//! - `cargo xtask stage-app` — validate and stage the CSR application bundle.
//! - `cargo xtask indexnow` — notify IndexNow after a successful deploy.
//!
//! Not covered here: `cargo deny` (its own job, needs a separate tool install)
//! and the release bundle (slow, and `meta build` owns it).

mod discovery;
mod styles;

use std::process::{Command, Stdio};

fn main() -> std::process::ExitCode {
    let task = std::env::args().nth(1);
    let result = match task.as_deref() {
        Some("ci") => ci(),
        Some("styles") => styles::build().map(|sizes| {
            for (app, bytes) in sizes {
                println!("{app}: {bytes} bytes");
            }
        }),
        Some("live") => live(),
        Some("core") => core(),
        Some("stage-web") => discovery::stage(),
        Some("stage-app") => discovery::stage_app(),
        Some("indexnow") => discovery::notify_indexnow(),
        other => {
            if let Some(name) = other {
                eprintln!("unknown task: {name}\n");
            }
            eprintln!("usage: cargo xtask <ci|styles|live|core|stage-web|stage-app|indexnow>");
            return std::process::ExitCode::FAILURE;
        }
    };

    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("\n\x1b[31mfailed:\x1b[0m {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

type Fallible = Result<(), Box<dyn std::error::Error>>;

/// Announce a step so a failure inside `cargo xtask ci` names itself. Running
/// the whole gate as one CI step trades GitHub's per-step UI for the guarantee
/// that local and CI are the same code; these headers buy the readability back.
fn step(name: &str) {
    println!("\n\x1b[1;36m── {name} ─────────────────────────────────────\x1b[0m");
}

fn run(program: &str, args: &[&str]) -> Fallible {
    let status = Command::new(program)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        return Err(format!("`{program} {}` failed", args.join(" ")).into());
    }
    Ok(())
}

/// Everything CI's `check` and `wasm32 boundary` jobs run, in the same order:
/// cheapest first, so a formatting slip does not cost a full compile to find.
fn ci() -> Fallible {
    step("rustfmt");
    run("cargo", &["fmt", "--all", "--", "--check"])?;

    step("clippy");
    run(
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
    )?;

    step("build");
    run("cargo", &["build", "--workspace"])?;

    step("test");
    run("cargo", &["test", "--workspace"])?;

    step("stylesheets are current");
    styles_are_current()?;

    step("no predecessor database");
    no_predecessor()?;

    step("no hardcoded hues");
    no_hardcoded_hues()?;

    step("wasm32 boundary");
    wasm_boundary()?;

    println!("\n\x1b[1;32mall checks passed\x1b[0m");
    Ok(())
}

/// Every colour names its job, in every app and in the shared kit.
///
/// `apps/hiring` is dark and the other two are light, and all three define the
/// same token names against their own palettes. A class naming a hue is
/// therefore correct in at most one of them: `bg-white` on a component in
/// `rv2-ui` drew a white card on the dark page for as long as nothing checked.
///
/// This lives here rather than as a test in one app because it is a scan over
/// source text. As a test it needed that app's test binary to compile before it
/// could read a string, and it could only reach the files that app owned —
/// which is how the other two apps and seven of the kit's files stayed
/// unscanned. A grep needs neither.
///
/// Bare `white` and `black` are included. They carry no shade, so a pattern
/// built from `{palette}-{shade}` cannot see them, and `bg-white` was the
/// spelling that actually shipped.
fn no_hardcoded_hues() -> Fallible {
    const PALETTES: &str = "slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|\
                            emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose";
    const SHADES: &str = "50|100|200|300|400|500|600|700|800|900|950";
    const PREFIXES: &str = "bg|text|border|ring|fill|stroke|from|to|via|divide|outline|shadow|\
                            decoration|placeholder|caret|accent";

    let pattern = format!("({PREFIXES})-(({PALETTES})-({SHADES})|white|black)");

    let output = Command::new("grep")
        .args([
            "-rInE",
            &pattern,
            "--include=*.rs",
            "--exclude-dir=target",
            "--exclude-dir=xtask",
            "apps",
            "crates",
        ])
        .output()?;

    let offenders: Vec<_> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.contains(HUE_OPT_OUT))
        .map(str::to_string)
        .collect();

    if !offenders.is_empty() {
        eprintln!("{}", offenders.join("\n"));
        return Err(format!(
            "a colour is named by its hue rather than its job\n         \
             the token names are in each app's assets/input.css\n         \
             if a line genuinely needs the word, mark it `{HUE_OPT_OUT}`"
        )
        .into());
    }
    println!("clean");
    Ok(())
}

/// The compiled stylesheets are committed, so a fresh clone renders correctly
/// without a build step. The cost of that is they can go stale: edit a
/// component's classes, forget `cargo xtask styles`, and the committed CSS no
/// longer matches the source with nothing to notice.
///
/// Same shape as `cargo fmt --check` — regenerate, compare, fail on a
/// difference.
///
/// It compiles into `target/style-check/` rather than over the committed files.
/// The first version wrote in place and restored the originals when they
/// differed, which made the one command whose entire job is "verify nothing
/// changed" also the command most likely to leave a dirty tree: a panic, a
/// failing second app, or a Ctrl-C between the write and the restore all left
/// generated CSS staged over the committed CSS.
fn styles_are_current() -> Fallible {
    let root = styles::workspace_root()?;
    let scratch = root.join("target/style-check");

    let mut stale = Vec::new();
    for sheet in styles::compile(Some(&scratch))? {
        let committed = styles::committed(&root, sheet.app);
        if std::fs::read(&committed).unwrap_or_default() != std::fs::read(&sheet.path)? {
            stale.push(committed.display().to_string());
        }
    }

    if !stale.is_empty() {
        return Err(format!(
            "stale stylesheet(s): {}\n         run `cargo xtask styles` and commit the result",
            stale.join(", ")
        )
        .into());
    }
    println!("stylesheets match their sources");
    Ok(())
}

/// The marker a line may carry to opt out of [`no_predecessor`].
///
/// The check is a blunt grep for three words, and a blunt grep cannot tell a
/// dependency from a sentence *about* a dependency. Without an escape hatch,
/// writing "we do not use Postgres" in a design doc fails CI with no way to say
/// "I meant that" — and the usual outcome of an unarguable check is that
/// somebody deletes the check.
///
/// It is deliberately ugly and deliberately per-line: an opt-out that is easy
/// to apply broadly stops being an exception.
const PREDECESSOR_OPT_OUT: &str = "predecessor-mention-ok";

/// The marker a line may carry to opt out of [`no_hardcoded_hues`].
///
/// Same reasoning as [`PREDECESSOR_OPT_OUT`], and the scan's own test fixtures
/// are the case that forced it: a test asserting that `bg-white` is reported
/// has to contain the string `bg-white`.
const HUE_OPT_OUT: &str = "hue-scan-ok";

/// No trace of the predecessor stack, anywhere, documentation included.
///
/// This needed allow-listed exceptions while a migration binary existed to read
/// the old database. It does not any more, so it is the strict form: zero
/// matches, no exclusions beyond build output, this tool's own source (which
/// necessarily contains the words it searches for), and any line explicitly
/// marked with [`PREDECESSOR_OPT_OUT`].
fn no_predecessor() -> Fallible {
    let output = Command::new("grep")
        .args([
            // `-I` skips binary files, and the reason is the opt-out rather
            // than taste: the only remedy this check offers is a
            // `predecessor-mention-ok` marker on the offending LINE, and a
            // binary file has no line anyone can mark. A binary match is
            // therefore unactionable by this check's own instructions, so
            // failing on one leaves no way forward except deleting the file or
            // deleting the check. It has already happened once, against a
            // task-tracker's compressed event store that had recorded a bead
            // describing this very grep.
            "-rInE",
            "supabase|postgres|pg2events",
            "--exclude-dir=.git",
            "--exclude-dir=target",
            "--exclude-dir=.github",
            "--exclude-dir=xtask",
            ".",
        ])
        .output()?;

    let offenders: Vec<_> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.contains(PREDECESSOR_OPT_OUT))
        .map(str::to_string)
        .collect();

    if !offenders.is_empty() {
        eprintln!("{}", offenders.join("\n"));
        return Err(format!(
            "a reference to the predecessor stack is present\n         \
             if a line genuinely needs the word, mark it `{PREDECESSOR_OPT_OUT}`"
        )
        .into());
    }
    println!("clean");
    Ok(())
}

/// §2.2, "the single most dangerous line in this workspace", in both
/// directions: the WASM-safe crates must cross-compile, and the server-only
/// crates must not be reachable from the apps.
fn wasm_boundary() -> Fallible {
    run("rustup", &["target", "add", "wasm32-unknown-unknown"])?;
    run(
        "cargo",
        &[
            "check",
            "--target",
            "wasm32-unknown-unknown",
            "-p",
            "rv2-events",
            "-p",
            "rv2-domain",
            "-p",
            "rv2-ai",
            "-p",
            "rv2-api-types",
            "-p",
            "rv2-ui",
            "-p",
            "rv2-client",
            "-p",
            "app",
            "-p",
            "web",
            "-p",
            "rv2-hiring",
            "-p",
            "hiring",
        ],
    )?;

    const SERVER_ONLY: &[&str] = &[
        "rv2-allsource",
        "rv2-shared",
        "better-auth-allsource",
        "rv2-analytics",
        "rv2-email",
        "rv2-jobs",
    ];

    // One `cargo tree` per app, not one per (app, crate) pair. The tree depends
    // only on the app, so the nested loop this replaced ran the same two
    // commands five times each — ten resolutions for two distinct answers, on
    // every CI run and every local gate.
    for wasm_app in ["app", "web", "hiring"] {
        let tree = Command::new("cargo")
            .args([
                "tree",
                "--target",
                "wasm32-unknown-unknown",
                "-p",
                wasm_app,
                "--prefix",
                "none",
            ])
            .output()?;
        let tree = String::from_utf8_lossy(&tree.stdout);

        for server_crate in SERVER_ONLY {
            let reachable = tree
                .lines()
                .any(|line| line.starts_with(&format!("{server_crate} v")));
            if reachable {
                return Err(
                    format!("{wasm_app} depends on the server-only crate {server_crate}").into(),
                );
            }
        }
    }
    println!("boundary holds in both directions");
    Ok(())
}

/// The tests that need a live Core.
///
/// They are `#[ignore]`d, so `cargo test` skips them silently — which is
/// indistinguishable from passing. This is the command that actually runs them,
/// and it sets the four environment variables the README previously asked
/// people to export by hand.
///
/// Contract suite first: it asserts *which* Core behaviours everything else
/// relies on, so a broken assumption names itself instead of surfacing as a
/// confusing 404 in the slice.
fn live() -> Fallible {
    let core =
        std::env::var("ALLSOURCE_CORE_URL").unwrap_or_else(|_| "http://localhost:3900".to_string());

    step("Core reachable");
    let health = Command::new("curl")
        .args(["-sf", "-m", "3", &format!("{core}/health")])
        .output()?;
    if !health.status.success() {
        return Err(format!(
            "no Core at {core}\n         start one with:\n           \
             ALLSOURCE_DATA_DIR=.allsource-data ALLSOURCE_DEV_MODE=true allsource-core\n         \
             or `meta dev`, which runs it for you"
        )
        .into());
    }
    println!("{}", String::from_utf8_lossy(&health.stdout));

    // Safe: single-threaded, before any test process is spawned.
    unsafe {
        std::env::set_var("ALLSOURCE_CORE_URL", &core);
        std::env::set_var("ALLSOURCE_QUERY_URL", &core);
        std::env::set_var("ALLSOURCE_API_KEY", "dev");
        std::env::set_var(
            "JWT_SECRET",
            "dev-secret-key-that-is-at-least-32-characters-long",
        );
    }

    step("Core contract");
    run(
        "cargo",
        &[
            "test",
            "-p",
            "rv2-allsource",
            "--test",
            "core_contract",
            "--",
            "--ignored",
            "--test-threads=1",
        ],
    )?;

    step("vertical slice");
    run(
        "cargo",
        &[
            "test",
            "-p",
            "api",
            "--features",
            "allsource-auth",
            "--test",
            "vertical_slice",
            "--",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ],
    )?;

    println!("\n\x1b[1;32mlive checks passed\x1b[0m");
    Ok(())
}

/// Run AllSource Core with the local dev settings.
///
/// `meta.toml` drives `meta dev` through this rather than invoking
/// `allsource-core` directly, because `meta doctor` probes every declared tool
/// with `--version` — and `allsource-core` has no such flag, so the probe
/// *starts a server* and doctor hangs forever. Routing through cargo, which
/// does answer `--version`, keeps doctor honest.
///
/// `ALLSOURCE_DEV_MODE` bypasses API-key auth. Local development only.
fn core() -> Fallible {
    let data_dir =
        std::env::var("ALLSOURCE_DATA_DIR").unwrap_or_else(|_| ".allsource-data".to_string());
    println!("Core on :3900, data in {data_dir}/ (dev mode: API-key auth bypassed)");

    let status = Command::new("allsource-core")
        .env("ALLSOURCE_HOST", "127.0.0.1")
        .env("ALLSOURCE_PORT", "3900")
        .env("ALLSOURCE_DATA_DIR", &data_dir)
        .env("ALLSOURCE_DEV_MODE", "true")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| {
            format!("could not run allsource-core ({e}). Install it with:\n           cargo install allsource-core --version 0.23.0 --locked")
        })?;

    if !status.success() {
        return Err("allsource-core exited non-zero".into());
    }
    Ok(())
}
