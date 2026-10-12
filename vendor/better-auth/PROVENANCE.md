# better-auth 0.10.0 library patch

Source: [published better-auth 0.10.0](https://crates.io/crates/better-auth/0.10.0),
upstream commit `cfee70e34e864af18f83b6a8577f91a8437aa02b`.
Registry archive SHA-256:
`c13f256cc4f6d4bfadbddc7509e60775127ada519c3cfdfec3e7ca43dc211dd6`.
The six files under `src/`, README and MIT license are copied unchanged.

The manifest preserves upstream library features and production dependencies
except `serde_yaml`. The library has no YAML calls. Upstream's two OpenAPI
maintenance binaries and `tests/compatibility_tests.rs` use YAML; these tools,
examples and external development tests are not part of this library patch.
Their target declarations and development-only dependencies are omitted.

Removing the unused dependency removes deprecated `serde_yaml` and unmaintained
`unsafe-libyaml` from the root lockfile. No auth implementation, wire format or
parser changes. The existing advisory policy remains enforced.

The predecessor source scan permits exactly five unchanged upstream lines
describing an optional adapter. The dependency policy bans that adapter from
the resolved graph. Source-checksum regression tests cover all six copied files;
root-lockfile tests reject the removed dependencies.

The root `[patch.crates-io]` selects this library while retaining published
better-auth-core and better-auth-api 0.10.0. This directory is excluded from
workspace membership and has no separate lockfile. Workspace auth tests and
live Core contracts exercise the consumer; upstream development tests are not
claimed as executed.

Remove this patch when a compatible upstream release removes or feature-gates
the unused YAML dependency. Do not update library source independently of its
published version and provenance.
