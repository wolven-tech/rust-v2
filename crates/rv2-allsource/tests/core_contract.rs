//! What Core actually does, asserted against a live Core.
//!
//! ## Why this file exists
//!
//! Every AllSource defect found in this codebase so far was an assumption that
//! was *documented in a comment* and never asserted anywhere:
//!
//! - a query without `tenant_id` returns an empty set rather than an error, so
//!   session lookups silently found nothing and every request 401'd;
//! - `ProjectionWorker::start` does not hydrate, so `GET /posts` returned a
//!   partial list after a restart with no error;
//! - the SDK rewrites event types on ingest, which is harmless only because our
//!   grammar happens to be a fixed point of the rewrite.
//!
//! In each case the end-to-end test passed. It exercised a different code path,
//! or it asserted on an HTTP status that a wrong-but-plausible empty result
//! still produced. An end-to-end test that happens to be green tells you
//! nothing about *which* assumption holds.
//!
//! These tests therefore assert Core's behaviour **directly**, one behaviour per
//! test, named after the behaviour rather than the feature. When one fails it
//! names the broken assumption instead of pointing at a handler.
//!
//! ## Running
//!
//! `#[ignore]`d because they need a live Core:
//!
//! ```bash
//! ALLSOURCE_DATA_DIR=.allsource-data ALLSOURCE_DEV_MODE=true allsource-core &
//! cargo test -p rv2-allsource --test core_contract -- --ignored
//! ```
//!
//! Behaviour ids (B*) refer to `docs/ledger/allsource-integration-corpus.md`.

use serde_json::{Value, json};

fn core_url() -> String {
    std::env::var("ALLSOURCE_CORE_URL").unwrap_or_else(|_| "http://localhost:3900".to_string())
}

fn tenant() -> String {
    std::env::var("ALLSOURCE_TENANT_ID").unwrap_or_else(|_| "default".to_string())
}

/// A unique entity per test run, so tests never observe each other's events.
fn entity(kind: &str) -> String {
    format!("contract-{kind}:{}", uuid::Uuid::new_v4())
}

async fn append(
    client: &reqwest::Client,
    entity_id: &str,
    event_type: &str,
    payload: Value,
) -> Value {
    let response = client
        .post(format!("{}/api/v1/events", core_url()))
        .json(&json!({
            "entity_id": entity_id,
            "event_type": event_type,
            "payload": payload,
        }))
        .send()
        .await
        .expect("Core unreachable — is it running?");
    assert!(
        response.status().is_success(),
        "append failed: {:?}",
        response.status()
    );
    response.json().await.expect("append returned non-JSON")
}

async fn query(client: &reqwest::Client, params: &[(&str, &str)]) -> Value {
    client
        .get(format!("{}/api/v1/events/query", core_url()))
        .query(params)
        .send()
        .await
        .expect("Core unreachable")
        .json()
        .await
        .expect("query returned non-JSON")
}

fn events(response: &Value) -> &Vec<Value> {
    response["events"]
        .as_array()
        .expect("no `events` array in query response")
}

// ── B1 — append returns an id and a version ──────────────────────────────────

#[tokio::test]
#[ignore = "requires a live AllSource Core"]
async fn b1_append_returns_an_event_id_and_a_version() {
    let client = reqwest::Client::new();
    let id = entity("b1");

    let first = append(&client, &id, "contract.probe.created", json!({"n": 1})).await;
    assert!(first["event_id"].is_string(), "no event_id: {first}");
    assert_eq!(first["version"], 1, "first event should be version 1");

    // Version is per-entity and monotonic. A folder that reconstructs order from
    // it would silently mis-order if this ever became global or non-increasing.
    let second = append(&client, &id, "contract.probe.updated", json!({"n": 2})).await;
    assert_eq!(second["version"], 2, "version must increment per entity");
}

// ── B5 — the defect that started this file ───────────────────────────────────

/// **The single most important assertion here.**
///
/// A tenant-less query is not an error. It is an empty result set with HTTP 200,
/// indistinguishable from "this entity has no events". Every layer above it
/// faithfully propagated "nothing here" and produced a plausible 404/401.
///
/// If Core ever starts rejecting the tenant-less form, this test fails and the
/// workaround in `tenant_query` can be deleted. If it keeps silently returning
/// empty, this test documents exactly why that module exists.
#[tokio::test]
#[ignore = "requires a live AllSource Core"]
async fn b5_a_query_without_tenant_id_silently_returns_empty() {
    let client = reqwest::Client::new();
    let id = entity("b5");
    append(&client, &id, "contract.probe.created", json!({"n": 1})).await;

    let without = query(&client, &[("entity_id", id.as_str())]).await;
    assert_eq!(
        events(&without).len(),
        0,
        "Core answered a tenant-less query with data. If this is now scoped by \
         default, `rv2_allsource::tenant_query` may be removable — see B20."
    );
    assert_eq!(without["count"], 0);

    let with = query(
        &client,
        &[("entity_id", id.as_str()), ("tenant_id", &tenant())],
    )
    .await;
    assert_eq!(
        events(&with).len(),
        1,
        "the SAME query with tenant_id must return the event — otherwise the \
         event never landed and this test is measuring the wrong thing"
    );
}

// ── B6 / B8 — entity scoping ─────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires a live AllSource Core"]
async fn b8_entity_id_filters_to_exactly_that_entity() {
    let client = reqwest::Client::new();
    let (mine, theirs) = (entity("b8-mine"), entity("b8-theirs"));

    append(
        &client,
        &mine,
        "contract.probe.created",
        json!({"who": "mine"}),
    )
    .await;
    append(
        &client,
        &theirs,
        "contract.probe.created",
        json!({"who": "theirs"}),
    )
    .await;

    let response = query(
        &client,
        &[("entity_id", mine.as_str()), ("tenant_id", &tenant())],
    )
    .await;
    let found = events(&response);
    assert_eq!(found.len(), 1, "entity_id leaked another entity's events");
    assert_eq!(found[0]["payload"]["who"], "mine");
}

// ── B7 — ordering ────────────────────────────────────────────────────────────

/// `latest_live_payload` takes `.last()`, and every folder replays in the order
/// Core returns. If Core ever flipped to newest-first, folds would silently
/// reconstruct the *oldest* state — a regression with no error attached.
#[tokio::test]
#[ignore = "requires a live AllSource Core"]
async fn b7_query_results_are_oldest_first() {
    let client = reqwest::Client::new();
    let id = entity("b7");

    for n in 1..=3 {
        append(&client, &id, "contract.probe.updated", json!({"n": n})).await;
    }

    let response = query(
        &client,
        &[("entity_id", id.as_str()), ("tenant_id", &tenant())],
    )
    .await;
    let order: Vec<i64> = events(&response)
        .iter()
        .map(|e| e["payload"]["n"].as_i64().expect("payload.n"))
        .collect();

    assert_eq!(
        order,
        vec![1, 2, 3],
        "Core returned events newest-first; every fold is now backwards"
    );
}

// ── B11 — read-after-write ───────────────────────────────────────────────────

/// `posts::create` appends and then reads back through the fold to prove the
/// write landed. That is only sound if an append is immediately queryable. If
/// Core ever became eventually-consistent here, `POST /posts` would start
/// returning 404 intermittently — which is exactly how the tenant bug presented,
/// so it is worth being able to tell the two apart.
#[tokio::test]
#[ignore = "requires a live AllSource Core"]
async fn b11_an_appended_event_is_immediately_queryable() {
    let client = reqwest::Client::new();
    let id = entity("b11");

    append(&client, &id, "contract.probe.created", json!({"n": 1})).await;

    // Deliberately no sleep, no retry.
    let response = query(
        &client,
        &[("entity_id", id.as_str()), ("tenant_id", &tenant())],
    )
    .await;
    assert_eq!(
        events(&response).len(),
        1,
        "append is not read-your-writes; posts::create's read-back needs a retry"
    );
}

// ── B9 — event type prefix ───────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires a live AllSource Core"]
async fn b9_event_type_prefix_filters_to_that_family() {
    let client = reqwest::Client::new();
    let id = entity("b9");
    let family = format!("contractb9{}", uuid::Uuid::new_v4().simple());

    append(
        &client,
        &id,
        &format!("{family}.thing.created"),
        json!({"keep": true}),
    )
    .await;
    append(
        &client,
        &id,
        &format!("{family}.thing.updated"),
        json!({"keep": true}),
    )
    .await;
    append(
        &client,
        &id,
        "contract.other.created",
        json!({"keep": false}),
    )
    .await;

    let response = query(
        &client,
        &[
            ("event_type_prefix", format!("{family}.").as_str()),
            ("tenant_id", &tenant()),
        ],
    )
    .await;

    let found = events(&response);
    assert_eq!(
        found.len(),
        2,
        "prefix filter returned the wrong set: {found:?}"
    );
    assert!(
        found.iter().all(|e| e["payload"]["keep"] == true),
        "prefix filter matched an unrelated family"
    );
}

// ── B10 — payload filter ─────────────────────────────────────────────────────

/// `better-auth-allsource::find_by_field` looks users up by email this way. If
/// `payload_filter` silently stopped matching, sign-in would report "no such
/// user" for users that exist — a silent-empty failure of the same family as B5.
#[tokio::test]
#[ignore = "requires a live AllSource Core"]
async fn b10_payload_filter_matches_top_level_fields() {
    let client = reqwest::Client::new();
    let family = format!("contractb10{}", uuid::Uuid::new_v4().simple());
    let needle = uuid::Uuid::new_v4().to_string();

    append(
        &client,
        &entity("b10-a"),
        &format!("{family}.thing.created"),
        json!({"email": needle}),
    )
    .await;
    append(
        &client,
        &entity("b10-b"),
        &format!("{family}.thing.created"),
        json!({"email": "other"}),
    )
    .await;

    let response = query(
        &client,
        &[
            ("event_type_prefix", format!("{family}.").as_str()),
            (
                "payload_filter",
                format!(r#"{{"email":"{needle}"}}"#).as_str(),
            ),
            ("tenant_id", &tenant()),
        ],
    )
    .await;

    let found = events(&response);
    assert_eq!(
        found.len(),
        1,
        "payload_filter matched {} events, expected 1",
        found.len()
    );
    assert_eq!(found[0]["payload"]["email"], needle);
}

// ── B12 — delete tombstones ──────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires a live AllSource Core"]
async fn b12_a_delete_tombstone_is_the_latest_event() {
    let client = reqwest::Client::new();
    let id = entity("b12");

    append(
        &client,
        &id,
        "contract.probe.created",
        json!({"alive": true}),
    )
    .await;
    append(
        &client,
        &id,
        "contract.probe.deleted",
        json!({"_deleted": true}),
    )
    .await;

    let response = query(
        &client,
        &[("entity_id", id.as_str()), ("tenant_id", &tenant())],
    )
    .await;
    let found = events(&response);

    // `latest_live_payload` reads `.last()` and checks `_deleted`. Both halves
    // of that depend on this ordering, not just the flag.
    assert_eq!(found.len(), 2);
    assert_eq!(
        found.last().expect("events")["payload"]["_deleted"],
        true,
        "the tombstone is not last; deleted entities would read as alive"
    );
}

// ── B15 / B16 — the projection KV round-trip ─────────────────────────────────

/// The worker keeps its read model in **process memory**. `ProjectionWorker`
/// streams from the server-side cursor into a freshly `Default`-constructed
/// state and does not hydrate (B14), so after a restart everything folded before
/// the last ack is simply gone — `GET /posts` returns a partial list with no
/// error anywhere.
///
/// `start_posts_worker` closes that gap with Core's projection KV: flush each
/// entity's state as it folds, read it back at boot. That fix is only sound if
/// the round-trip actually works, and nothing asserted it — the same shape of
/// omission that produced the tenant defect.
///
/// This test drives the two SDK calls the fix depends on, against a live Core.
#[tokio::test]
#[ignore = "requires a live AllSource Core"]
async fn b15_b16_projection_state_survives_a_write_and_read_back() {
    use allsource::CoreClient;

    let core = CoreClient::new(&core_url(), "dev").expect("client");
    // Unique worker name: the summary read is per-worker, so a shared name would
    // make this test observe state from other runs.
    let worker = format!("contract_posts_{}", uuid::Uuid::new_v4().simple());
    let entity_id = entity("b15");

    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct Snapshot {
        title: String,
        version: u32,
    }

    let written = Snapshot {
        title: "On the Analytical Engine".to_string(),
        version: 7,
    };

    core.put_projection_state(&worker, &entity_id, &written)
        .await
        .expect("put_projection_state failed — the hydrate fix has no storage");

    let summary: Vec<(String, Snapshot)> = core
        .get_projection_state_summary(&worker)
        .await
        .expect("get_projection_state_summary failed — boot hydration is dead");

    let found = summary
        .iter()
        .find(|(id, _)| id == &entity_id)
        .map(|(_, state)| state);

    assert_eq!(
        found,
        Some(&written),
        "projection state did not survive the round-trip; a worker restart \
         would silently serve a partial read model (B14)"
    );
}

// ── B17 — a fresh consumer id replays from zero ──────────────────────────────

/// D15 makes read-model rebuild a rename: point the worker at a new durable
/// consumer id and it replays the whole stream, rather than resuming from an
/// existing cursor. If a *new* id ever inherited a position, a rebuild would
/// silently produce a partial read model — and "rebuild the projection" is the
/// documented answer to a bad fold, so failing quietly there is expensive.
#[tokio::test]
#[ignore = "requires a live AllSource Core"]
async fn b17_an_unseen_consumer_id_has_no_checkpoint() {
    use allsource::CoreClient;

    let core = CoreClient::new(&core_url(), "dev").expect("client");
    let fresh = format!("contract_rebuild_{}", uuid::Uuid::new_v4().simple());

    let checkpoint = core
        .load_checkpoint(&fresh)
        .await
        .expect("load_checkpoint failed");

    assert_eq!(
        checkpoint, None,
        "a never-registered consumer already has a checkpoint; renaming the \
         consumer would NOT replay from zero and D15's rebuild strategy is unsound"
    );
}

// ── B2 — metadata survives the append ────────────────────────────────────────

/// `tooling/pg2events` stamps every migrated event with
/// `{"source": "supabase-migration", …}`. If metadata were silently dropped,
/// migrated and native events would become indistinguishable after the fact —
/// and the migration is a one-shot, so the provenance is unrecoverable.
#[tokio::test]
#[ignore = "requires a live AllSource Core"]
async fn b2_metadata_survives_the_append() {
    let client = reqwest::Client::new();
    let id = entity("b2");

    let response = client
        .post(format!("{}/api/v1/events", core_url()))
        .json(&json!({
            "entity_id": id,
            "event_type": "contract.probe.created",
            "payload": {"n": 1},
            "metadata": {"source": "core-contract-test"},
        }))
        .send()
        .await
        .expect("Core unreachable");
    assert!(response.status().is_success());

    let found = query(
        &client,
        &[("entity_id", id.as_str()), ("tenant_id", &tenant())],
    )
    .await;
    let event = &events(&found)[0];
    assert_eq!(
        event["metadata"]["source"], "core-contract-test",
        "metadata did not round-trip; migration provenance would be lost: {event}"
    );
}

// ── B3 / B4 — the event-type normalizer ──────────────────────────────────────

/// `CoreClient::ingest_event` unconditionally rewrites the event type through
/// `normalize_event_type`. D11 rejects that normalizer as a schema because it is
/// lossy and non-injective — `user_created`, `userCreated`, `UserCreated` and
/// `user-created` all collapse to `user.created`.
///
/// We cannot switch it off, so the whole design rests on our grammar being a
/// **fixed point**: `<domain>.<entity>.<action>`, already lowercase and dotted,
/// comes back unchanged. `EventWriter::append` fails loudly if that stops
/// holding, but nothing asserted that it holds *against a live Core* — only
/// against a reading of the SDK source.
///
/// If this fails, events are being stored under names no folder matches, and
/// every read model goes quietly empty.
#[tokio::test]
#[ignore = "requires a live AllSource Core"]
async fn b4_our_wire_grammar_is_a_fixed_point_of_the_normalizer() {
    let client = reqwest::Client::new();

    // The real wire types this codebase emits, not invented samples.
    for wire_type in [
        "identity.user.registered",
        "content.post.created",
        "content.post.updated",
        "content.post.deleted",
        "auth.session.created",
    ] {
        let id = entity("b4");
        append(&client, &id, wire_type, json!({"probe": true})).await;

        let found = query(
            &client,
            &[("entity_id", id.as_str()), ("tenant_id", &tenant())],
        )
        .await;
        let stored = events(&found)[0]["event_type"]
            .as_str()
            .expect("event_type")
            .to_string();

        assert_eq!(
            stored, wire_type,
            "the normalizer rewrote `{wire_type}` to `{stored}` — our grammar is \
             no longer a fixed point, and every folder matching the old name is dead"
        );
    }
}

/// The counter-test: prove the normalizer is genuinely active, so
/// [`b4_our_wire_grammar_is_a_fixed_point_of_the_normalizer`] is meaningful.
/// Without it, B4 would still pass if the normalizer were removed entirely, and
/// would then be asserting nothing at all.
///
/// This one must go through `CoreClient::ingest_event`, not raw HTTP. The
/// normalization is **client-side** — `ingest_event` rewrites `input.event_type`
/// before posting. Core itself *rejects* a non-conforming type outright:
/// posting `ContractProbeCreated` over raw HTTP returns **400**, it is not
/// silently normalized server-side.
///
/// That distinction matters. It means an event written by any non-SDK client —
/// `curl`, another language's SDK, `tooling/pg2events` if it ever bypassed the
/// writer — fails loudly rather than landing under a mangled name. The
/// normalizer is a client-side convenience, not a server-side schema.
#[tokio::test]
#[ignore = "requires a live AllSource Core"]
async fn b3_the_sdk_normalizes_event_types_client_side() {
    use allsource::{CoreClient, IngestEventInput};

    let core = CoreClient::new(&core_url(), "dev").expect("client");
    let id = entity("b3");

    core.ingest_event(IngestEventInput {
        entity_id: id.clone(),
        event_type: "ContractProbeCreated".to_string(),
        payload: json!({"probe": true}),
        metadata: None,
    })
    .await
    .expect(
        "ingest_event failed — if this is a 400, the SDK stopped normalizing \
             and every PascalCase caller now breaks at the wire instead",
    );

    let client = reqwest::Client::new();
    let found = query(
        &client,
        &[("entity_id", id.as_str()), ("tenant_id", &tenant())],
    )
    .await;
    let stored = events(&found)[0]["event_type"]
        .as_str()
        .expect("event_type");

    assert_eq!(
        stored, "contract.probe.created",
        "the SDK did not normalize as documented; B4's fixed-point argument \
         needs revisiting (D11)"
    );
}
