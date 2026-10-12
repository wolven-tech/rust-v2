use sha2::{Digest, Sha256};
use std::{fs, path::Path};

#[test]
fn auth_patch_preserves_published_library_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/better-auth/src");
    for (file, expected) in [
        (
            "lib.rs",
            "4512043c2105d314af3bfc648c661601a8e39cdf7bdd67280ae2f6649ce92d54",
        ),
        (
            "core/mod.rs",
            "75b4f9d37ce3bbf95bb14a7784ed8546433d578467d732bfbd0ae229dd99f8d0",
        ),
        (
            "core/auth.rs",
            "1189027fc9b4e536c79b9e35da59d95c7332cfffb268fef2f238aa7058be44ed",
        ),
        (
            "handlers/mod.rs",
            "95b8d81b32fd7b4e3208e8783fbd40afe3b0bdfa689d0c4f39d51959680e7e15",
        ),
        (
            "handlers/axum.rs",
            "43739ed54df4bc6528cfec35f089aba8342fed356a3c0d62dc59e6028a40250a",
        ),
        (
            "tests/response_shape_tests.rs",
            "758984dbf149f8f40200657555794f9813c6642fd6997849b4195585517c669a",
        ),
    ] {
        let bytes = fs::read(root.join(file)).expect("vendored source exists");
        assert_eq!(format!("{:x}", Sha256::digest(bytes)), expected, "{file}");
    }
}

#[test]
fn root_lockfile_drops_unused_yaml_and_sql_adapter() {
    let lockfile = include_str!("../../../Cargo.lock");
    for package in ["serde_yaml", "unsafe-libyaml", "sqlx"] {
        assert!(
            !lockfile.contains(&format!("name = \"{package}\"")),
            "{package} must stay outside the resolved graph"
        );
    }
}
