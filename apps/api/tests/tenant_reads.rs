//! Every API read goes through the tenant-scoped reader.
//!
//! Core answers an SDK query with no `tenant_id` with an empty result and HTTP
//! 200, so a handler that reads through `QueryClient` returns 404 or `[]` and
//! no test that skips Core can see it. This scan runs in plain `cargo test`.

use std::{fs, path::Path};

const FORBIDDEN: [&str; 3] = [".query_and_fold", ".query_events(", ".query.query"];

fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).expect("readable source dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn api_source_never_reads_through_the_sdk_query_client() {
    let mut files = Vec::new();
    rust_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(
        files.iter().any(|f| f.ends_with("handlers/posts.rs")),
        "scan found no handlers: {files:?}"
    );

    let offenders: Vec<String> = files
        .iter()
        .flat_map(|file| {
            let text = fs::read_to_string(file).expect("readable source file");
            text.lines()
                .enumerate()
                .filter(|(_, line)| !line.trim_start().starts_with("//"))
                .filter(|(_, line)| FORBIDDEN.iter().any(|call| line.contains(call)))
                .map(|(n, line)| format!("{}:{}: {}", file.display(), n + 1, line.trim()))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(
        offenders.is_empty(),
        "use `AllSource::fold_entity` / `events_by_type_prefix`:\n{}",
        offenders.join("\n")
    );
}
