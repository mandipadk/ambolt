//! What somebody came to say: the door, the three kinds, and the
//! promise that nothing here closes for being old.

use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;

/// Open the door on ada/demo and make it public, which is the pair of
/// switches that lets a stranger say anything at all.
async fn open_the_door(forge: &Forge) {
    let app = &forge.app;
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/visibility",
        "ada",
        Some(json!({ "visibility": "public" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/policy",
        "ada",
        Some(
            json!({ "require_executed_check": true, "independence": "human_or_two_models",
                     "require_runner_verification": false, "runner_quorum": 1,
                     "required_domains": [], "require_concerns_resolved": true,
                     "agents_act_in_sessions": false, "proposals": false,
                     "community": true }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_closed_repository_says_how_its_door_opens() {
    let forge = boot().await;
    let app = &forge.app;
    let (status, body) = api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "nadia", "kind": "human", "display": "Nadia" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports",
        "nadia",
        Some(json!({ "kind": "bug", "title": "It broke", "body": "It broke." })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    // A refusal that names the switch, not one that names the rule.
    assert!(
        body["error"].as_str().unwrap().contains("community"),
        "{body}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_stranger_reports_a_bug_and_the_owner_hears_about_it() {
    let forge = boot().await;
    let app = &forge.app;
    open_the_door(&forge).await;
    let (status, body) = api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "nadia", "kind": "human", "display": "Nadia" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, filed) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports",
        "nadia",
        Some(json!({
            "kind": "bug",
            "title": "clone fails on git 2.39",
            "body": "Cloning over http hangs and then fails.",
            "version": "0.1.0-alpha.3",
            "command": "git clone https://ambolt.sh/git/ada/demo",
            "observed": "fatal: the remote end hung up unexpectedly",
            "expected": "a clone"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{filed}");
    assert_eq!(filed["number"], 1, "{filed}");

    let (status, report) = api(app, "GET", "/api/repos/ada/demo/reports/1", "nadia", None).await;
    assert_eq!(status, StatusCode::OK, "{report}");
    assert_eq!(report["kind"], "bug");
    assert_eq!(report["state"], "open");
    assert_eq!(report["by"], "nadia");
    assert_eq!(
        report["repro"]["command"], "git clone https://ambolt.sh/git/ada/demo",
        "the field that makes it checkable is kept: {report}"
    );

    // The owner hears. Somebody who watches the repository hears too.
    let (_, inbox) = api(app, "GET", "/api/inbox", "ada", None).await;
    let heard: Vec<&str> = inbox["notices"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "reported")
        .map(|n| n["what"].as_str().unwrap())
        .collect();
    assert_eq!(heard.len(), 1, "{inbox}");
    assert!(heard[0].contains("clone fails on git 2.39"), "{inbox}");
}

/// The bar is on the report, never on the reporter. Prose files fine —
/// it just cannot claim to be checkable — and anyone may sharpen it
/// afterwards, which is the whole difference between a ladder and a
/// gate.
#[tokio::test(flavor = "multi_thread")]
async fn prose_is_enough_to_file_and_anybody_may_sharpen_it() {
    let forge = boot().await;
    let app = &forge.app;
    open_the_door(&forge).await;
    let (status, body) = api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "nadia", "kind": "human", "display": "Nadia" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, filed) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports",
        "nadia",
        Some(json!({ "kind": "bug", "title": "it does not work",
                     "body": "I pressed the thing and nothing happened." })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{filed}");

    let (status, report) = api(app, "GET", "/api/repos/ada/demo/reports/1", "nadia", None).await;
    assert_eq!(status, StatusCode::OK, "{report}");
    assert!(report["repro"]["command"].is_null(), "{report}");

    // Somebody else adds what was missing.
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports/1/reply",
        "ada",
        Some(json!({ "body": "Reproduced: `ambolt serve --db x.db` then curl /healthz." })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, report) = api(app, "GET", "/api/repos/ada/demo/reports/1", "nadia", None).await;
    assert_eq!(report["replies"].as_array().unwrap().len(), 1, "{report}");

    // And whoever filed it hears that somebody answered, which is the
    // failure this whole surface exists to avoid.
    let (_, inbox) = api(app, "GET", "/api/inbox", "nadia", None).await;
    assert!(
        inbox["notices"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["kind"] == "report-reply"),
        "{inbox}"
    );
}

/// A request and a question carry no reproduction, because a field
/// nobody reads pretending to be evidence is worse than no field.
#[tokio::test(flavor = "multi_thread")]
async fn only_a_bug_carries_a_reproduction() {
    let forge = boot().await;
    let app = &forge.app;
    open_the_door(&forge).await;
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports",
        "ada",
        Some(
            json!({ "kind": "request", "title": "let me pin a repository",
                     "body": "I keep three open and hunt for them.",
                     "command": "not a command" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, report) = api(app, "GET", "/api/repos/ada/demo/reports/1", "ada", None).await;
    assert!(report["repro"]["command"].is_null(), "{report}");
}

/// Every settlement says which kind it was. There is no settlement that
/// means "we stopped looking", and no way to reach one by waiting.
#[tokio::test(flavor = "multi_thread")]
async fn settling_says_which_kind_of_closure_it_was() {
    let forge = boot().await;
    let app = &forge.app;
    open_the_door(&forge).await;
    for (number, kind, title) in [
        (1, "question", "how do I pin"),
        (2, "question", "how do I pin, again"),
    ] {
        let (status, body) = api(
            app,
            "POST",
            "/api/repos/ada/demo/reports",
            "ada",
            Some(json!({ "kind": kind, "title": title, "body": "asking" })),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body["number"], number, "{body}");
    }

    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports/1/settle",
        "ada",
        Some(json!({ "how": "answered", "note": "Bookmarks, on the repository page." })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, report) = api(app, "GET", "/api/repos/ada/demo/reports/1", "ada", None).await;
    assert_eq!(report["state"], "settled", "{report}");
    assert_eq!(report["settled"]["how"], "answered", "{report}");
    assert!(
        report["settled"]["note"]
            .as_str()
            .unwrap()
            .contains("Bookmarks"),
        "the reason is the artifact, not a tombstone: {report}"
    );

    // A duplicate names the one it repeats, and cannot name itself.
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports/2/settle",
        "ada",
        Some(json!({ "how": "duplicate", "note": "Asked before.", "duplicate_of": 2 })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports/2/settle",
        "ada",
        Some(json!({ "how": "duplicate", "note": "Asked before.", "duplicate_of": 1 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, report) = api(app, "GET", "/api/repos/ada/demo/reports/2", "ada", None).await;
    assert_eq!(report["settled"]["duplicate_of"], 1, "{report}");
}

/// Reporting is open; answering for the repository is not.
#[tokio::test(flavor = "multi_thread")]
async fn a_stranger_may_withdraw_their_own_and_settle_nothing_else() {
    let forge = boot().await;
    let app = &forge.app;
    open_the_door(&forge).await;
    let (status, body) = api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "nadia", "kind": "human", "display": "Nadia" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    for title in ["mine", "also mine"] {
        let (status, body) = api(
            app,
            "POST",
            "/api/repos/ada/demo/reports",
            "nadia",
            Some(json!({ "kind": "question", "title": title, "body": "asking" })),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }

    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports/1/settle",
        "nadia",
        Some(json!({ "how": "answered", "note": "I will answer for the project." })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert!(body["error"].as_str().unwrap().contains("task"), "{body}");

    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports/1/settle",
        "nadia",
        Some(json!({ "how": "withdrawn", "note": "Worked it out." })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // Somebody else's is not theirs to withdraw either.
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports/2/settle",
        "arbiter",
        Some(json!({ "how": "withdrawn", "note": "Not mine." })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
}

/// Discarding is the exception: the number and the reason stay, the
/// stranger's text does not, and its replies go with it.
#[tokio::test(flavor = "multi_thread")]
async fn discarding_keeps_the_number_and_the_reason_and_nothing_else() {
    let forge = boot().await;
    let app = &forge.app;
    open_the_door(&forge).await;
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports",
        "ada",
        Some(json!({ "kind": "bug", "title": "buy cheap watches", "body": "spam spam" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports/1/reply",
        "ada",
        Some(json!({ "body": "more spam" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports/1/discard",
        "ada",
        Some(json!({ "reason": "advertising" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (_, report) = api(app, "GET", "/api/repos/ada/demo/reports/1", "ada", None).await;
    assert_eq!(report["state"], "discarded", "{report}");
    assert!(
        !report["body"].as_str().unwrap().contains("spam"),
        "{report}"
    );
    assert!(report["replies"].as_array().unwrap().is_empty(), "{report}");
    assert_eq!(report["number"], 1, "the number stays: {report}");

    // A discarded report takes no more replies.
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports/1/reply",
        "ada",
        Some(json!({ "body": "hello?" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
}

/// The guide answers an agent that holds nothing but the address, and
/// says the one thing an agent most needs to be told: not here.
#[tokio::test(flavor = "multi_thread")]
async fn the_guide_answers_a_stranger_and_names_the_verbs() {
    let forge = boot().await;
    let app = &forge.app;
    open_the_door(&forge).await;

    let (status, guide) = api_anonymous(app, "GET", "/api/repos/ada/demo/guide", None).await;
    assert_eq!(status, StatusCode::OK, "{guide}");
    assert_eq!(guide["reports"]["open"], true, "{guide}");
    assert!(
        guide["reports"]["how"]
            .as_str()
            .unwrap()
            .contains("/api/repos/ada/demo/reports"),
        "{guide}"
    );
    assert!(
        guide["reports"]["cli"]
            .as_str()
            .unwrap()
            .contains("ambolt report"),
        "{guide}"
    );
    // The block a repository carries for every agent that never heard
    // of this forge.
    let block = guide["agents_md"].as_str().unwrap();
    assert!(block.contains("ambolt report bug"), "{block}");
    assert!(
        block.contains("mirror of it are not read"),
        "an agent's default is to file on the mirror: {block}"
    );
    assert!(block.contains("refs/for/main"), "{block}");
}

/// A private repository's guide is a private repository's business.
#[tokio::test(flavor = "multi_thread")]
async fn the_guide_of_a_private_repository_is_not_a_stranger_s_business() {
    let forge = boot().await;
    let app = &forge.app;
    let (status, body) = api_anonymous(app, "GET", "/api/repos/ada/demo/guide", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
}

/// The forge corrects the caller at the moment it reached for the wrong
/// verb, which is the only moment the correction is certain to be read.
#[tokio::test(flavor = "multi_thread")]
async fn a_refusal_to_write_intent_names_the_way_to_say_it_instead() {
    let forge = boot().await;
    let app = &forge.app;
    open_the_door(&forge).await;
    let (status, body) = api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "nadia", "kind": "human", "display": "Nadia" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, body) = api(
        app,
        "POST",
        "/api/tasks",
        "nadia",
        Some(json!({ "repo": "ada/demo", "title": "Add pinning", "spec": "Pin a repository." })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    let said = body["error"].as_str().unwrap();
    assert!(said.contains("/api/repos/ada/demo/reports"), "{said}");
    assert!(said.contains("request"), "{said}");
}

/// An open door still has a sill.
#[tokio::test(flavor = "multi_thread")]
async fn a_reporter_s_room_is_their_own_and_bounded() {
    let forge = boot().await;
    let app = &forge.app;
    open_the_door(&forge).await;
    let (status, body) = api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "nadia", "kind": "human", "display": "Nadia" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "POST",
        "/api/principals/nadia/quota",
        "ada",
        Some(json!({ "open_reports": 2 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    for number in 1..=2 {
        let (status, body) = api(
            app,
            "POST",
            "/api/repos/ada/demo/reports",
            "nadia",
            Some(json!({ "kind": "question", "title": format!("q{number}"), "body": "asking" })),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports",
        "nadia",
        Some(json!({ "kind": "question", "title": "one too many", "body": "asking" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");

    // Settling one gives the room back.
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports/1/settle",
        "nadia",
        Some(json!({ "how": "withdrawn", "note": "Worked it out." })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports",
        "nadia",
        Some(json!({ "kind": "question", "title": "room again", "body": "asking" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

/// A mention in a report reaches the person named, as one in a thread
/// does — and points at the report rather than at a change.
#[tokio::test(flavor = "multi_thread")]
async fn a_mention_in_a_report_reaches_the_person_named() {
    let forge = boot().await;
    let app = &forge.app;
    open_the_door(&forge).await;
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/reports",
        "ada",
        Some(json!({ "kind": "question", "title": "who knows this",
                     "body": "@arbiter do you know why the clone hangs?" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (_, inbox) = api(app, "GET", "/api/inbox", "arbiter", None).await;
    let mentioned: Vec<&serde_json::Value> = inbox["notices"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "mentioned")
        .collect();
    assert_eq!(mentioned.len(), 1, "{inbox}");
    assert!(
        mentioned[0]["what"].as_str().unwrap().contains("report #1"),
        "{inbox}"
    );
    assert_eq!(mentioned[0]["number"], 1, "{inbox}");
    assert!(mentioned[0]["change"].is_null(), "{inbox}");
}
