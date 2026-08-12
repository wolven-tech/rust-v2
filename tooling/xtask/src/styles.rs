//! Compiles each app's Tailwind stylesheet.
//!
//! ## Why this exists
//!
//! `dx` 0.7 is documented as auto-detecting Tailwind, but `dx build` and
//! `dx bundle` only **copy** `assets/tailwind.css` into the bundle. Shipping the
//! uncompiled source — a single `@import "tailwindcss";` line the browser
//! cannot resolve — produced a 23-byte stylesheet and a completely unstyled
//! site, while every command still reported success. Nothing in the build
//! catches that, which is why this step is explicit and why it asserts on the
//! output size at the end.
//!
//! ## Why the standalone binary
//!
//! Tailwind's npm CLI needs `tailwindcss` resolvable from `node_modules`, which
//! would put a `package.json` and a JS package manager back into a workspace
//! whose whole point is not having one. The standalone release is a single
//! self-contained executable with the engine inside it: no `package.json`, no
//! lockfile, nothing added to the repo. It is a build tool fetched on demand,
//! the same way `dx` is — not a dependency of the produced artifact.
//!
//! ## Why `@source` matters
//!
//! Tailwind cannot discover Rust files by itself. Each app's `input.css` names
//! the directories holding class strings, and that list **must** include
//! `crates/rv2-ui` — otherwise every class used only by the component kit is
//! tree-shaken out and the components render unstyled while the app's own
//! classes work, which is a genuinely confusing failure to debug.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Pinned rather than `latest`: an unpinned build tool means the stylesheet can
/// change without a single line of this repo changing.
const TAILWIND_VERSION: &str = "v4.3.3";

/// Below this, the output is the uncompiled `@import` line or an empty file
/// rather than a real stylesheet. The failure this guards against is silent, so
/// the guard has to be explicit.
const MIN_PLAUSIBLE_CSS_BYTES: u64 = 2_048;

struct App {
    name: &'static str,
    dir: &'static str,
}

const APPS: &[App] = &[
    App {
        name: "web",
        dir: "apps/web",
    },
    App {
        name: "app",
        dir: "apps/app",
    },
];

/// Compile every app's stylesheet. Returns the byte size per app.
pub fn build() -> Result<Vec<(&'static str, u64)>, Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let binary = ensure_tailwind(&root)?;
    let mut sizes = Vec::new();

    for app in APPS {
        let dir = root.join(app.dir);
        let input = dir.join("assets/input.css");
        let output = dir.join("assets/tailwind.css");

        if !input.exists() {
            return Err(format!(
                "{} has no assets/input.css — Tailwind needs a source file naming its \
                 @source globs, including ../../../crates/rv2-ui/src/**/*.rs",
                app.name
            )
            .into());
        }

        let status = Command::new(&binary)
            .current_dir(&root)
            .args([
                "-i",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
                "--minify",
            ])
            .status()?;

        if !status.success() {
            return Err(format!("tailwindcss failed for {}", app.name).into());
        }

        let bytes = std::fs::metadata(&output)?.len();
        if bytes < MIN_PLAUSIBLE_CSS_BYTES {
            return Err(format!(
                "{} produced only {bytes} bytes of CSS. That is the uncompiled source, not a \
                 stylesheet — check the @source globs in {}",
                app.name,
                input.display()
            )
            .into());
        }

        sizes.push((app.name, bytes));
    }

    Ok(sizes)
}

fn workspace_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // CARGO_MANIFEST_DIR is tooling/tailwind; the root is two levels up.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest
        .parent()
        .and_then(Path::parent)
        .ok_or("cannot locate workspace root")?
        .to_path_buf())
}

/// Fetches the standalone CLI into `target/` on first use.
///
/// `target/` rather than the repo: it is already gitignored, so a platform
/// binary can never be committed by accident.
fn ensure_tailwind(root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let binary = root.join("target/tailwindcss");
    if binary.exists() {
        return Ok(binary);
    }

    let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "macos-arm64",
        ("macos", "x86_64") => "macos-x64",
        ("linux", "aarch64") => "linux-arm64",
        ("linux", "x86_64") => "linux-x64",
        (os, arch) => return Err(format!("no standalone Tailwind build for {os}/{arch}").into()),
    };

    let url = format!(
        "https://github.com/tailwindlabs/tailwindcss/releases/download/{TAILWIND_VERSION}/tailwindcss-{platform}"
    );
    println!("fetching {url}");

    std::fs::create_dir_all(root.join("target"))?;
    let status = Command::new("curl")
        .args(["-sL", "--fail", "-o", binary.to_str().unwrap(), &url])
        .status()?;
    if !status.success() {
        return Err(format!("could not download {url}").into());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755))?;
    }

    Ok(binary)
}
