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
//!
//! Not covered here: `cargo deny` (its own job, needs a separate tool install)
//! and the release bundle (slow, and `meta build` owns it).

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
        other => {
            if let Some(name) = other {
                eprintln!("unknown task: {name}\n");
            }
            eprintln!("usage: cargo xtask <ci|styles|live|core>");
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

    step("wasm32 boundary");
    wasm_boundary()?;

    println!("\n\x1b[1;32mall checks passed\x1b[0m");
    Ok(())
}

/// The compiled stylesheets are committed, so a fresh clone renders correctly
/// without a build step. The cost of that is they can go stale: edit a
/// component's classes, forget `cargo xtask styles`, and the committed CSS no
/// longer matches the source with nothing to notice.
///
/// Same shape as `cargo fmt --check` — regenerate, compare, fail on a
/// difference. This is the only check here that *writes* before it reads, which
/// is why it restores the originals on mismatch rather than leaving a dirty tree.
fn styles_are_current() -> Fallible {
    let root = std::env::current_dir()?;
    let paths: Vec<_> = ["apps/web", "apps/app"]
        .iter()
        .map(|dir| root.join(dir).join("assets/tailwind.css"))
        .collect();

    let before: Vec<_> = paths
        .iter()
        .map(|p| std::fs::read(p).unwrap_or_default())
        .collect();

    styles::build()?;

    let mut stale = Vec::new();
    for (path, original) in paths.iter().zip(&before) {
        let now = std::fs::read(path)?;
        if &now != original {
            std::fs::write(path, original)?;
            stale.push(path.display().to_string());
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

/// No trace of the predecessor stack, anywhere, documentation included.
///
/// This needed allow-listed exceptions while a migration binary existed to read
/// the old database. It does not any more, so it is the strict form: zero
/// matches, no exclusions beyond build output and this tool's own source (which
/// necessarily contains the words it searches for).
fn no_predecessor() -> Fallible {
    let output = Command::new("grep")
        .args([
            "-rinE",
            "supabase|postgres|pg2events",
            "--exclude-dir=.git",
            "--exclude-dir=target",
            "--exclude-dir=.github",
            "--exclude-dir=xtask",
            ".",
        ])
        .output()?;

    if output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stdout));
        return Err("a reference to the predecessor stack is present".into());
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
            "rv2-api-types",
            "-p",
            "rv2-ui",
            "-p",
            "rv2-client",
            "-p",
            "app",
            "-p",
            "web",
        ],
    )?;

    for server_crate in ["rv2-allsource", "rv2-shared", "better-auth-allsource"] {
        for wasm_app in ["app", "web"] {
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
            let reachable = String::from_utf8_lossy(&tree.stdout)
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
