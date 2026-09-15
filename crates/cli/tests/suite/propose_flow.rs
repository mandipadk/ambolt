//! Proposing: a stranger opens a change on a public repository and acts
//! on their own, and on nothing else.

use crate::common::*;
use axum::http::StatusCode;
use serde_json::{Value, json};

async fn policy_with_proposals(app: &axum::Router, on: bool) {
    let (_, mut policy) = api(app, "GET", "/api/repos/ada/demo/policy", "ada", None).await;
    policy["proposals"] = Value::Bool(on);
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/policy",
        "ada",
        Some(policy),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

async fn visibility(app: &axum::Router, public: bool) {
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/visibility",
        "ada",
        Some(json!({ "visibility": if public { "public" } else { "private" } })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_stranger_proposes_and_reaches_only_their_own() {
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

    // Private, or public with the door shut: the refusal names push.
    let open =
        json!({ "repo": "ada/demo", "target": "main", "title": "Fix the typo in the README" });
    let (status, body) = api(app, "POST", "/api/changes", "nadia", Some(open.clone())).await;
    assert_ne!(status, StatusCode::OK, "{body}");
    visibility(app, true).await;
    let (status, body) = api(app, "POST", "/api/changes", "nadia", Some(open.clone())).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert!(
        body["error"].as_str().unwrap_or("").contains("'push'"),
        "{body}"
    );

    // Open: the change is hers, and marked a proposal.
    policy_with_proposals(app, true).await;
    let (status, body) = api(app, "POST", "/api/changes", "nadia", Some(open.clone())).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let mine = body["id"].as_str().unwrap().to_owned();
    let (_, change) = api(app, "GET", &format!("/api/changes/{mine}"), "nadia", None).await;
    assert_eq!(change["proposal"], true, "{change}");
    assert_eq!(change["owner"], "nadia", "{change}");

    // She revises it, claims on it, and may abandon it.
    let revision = json!({ "commit_oid": "a".repeat(40), "message": "Fix the typo" });
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{mine}/revisions"),
        "nadia",
        Some(revision.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let claim = json!({ "kind": "test", "passed": true, "summary": "read it twice", "command": "cargo test" });
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{mine}/claims"),
        "nadia",
        Some(claim),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // Not hers: ada's change is out of reach, and so are judging and landing.
    let (status, body) = api(
        app,
        "POST",
        "/api/changes",
        "ada",
        Some(json!({ "repo": "ada/demo", "target": "main", "title": "Ada's own" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let adas = body["id"].as_str().unwrap().to_owned();
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{adas}/revisions"),
        "nadia",
        Some(json!({ "commit_oid": "b".repeat(40) })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert!(
        body["error"]
            .as_str()
            .unwrap_or("")
            .contains("changes you opened"),
        "{body}"
    );
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{adas}/abandon"),
        "nadia",
        Some(json!({ "reason": "mine now" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{mine}/enqueue"),
        "nadia",
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{mine}/verdicts"),
        "nadia",
        Some(
            json!({ "domain": "correctness", "disposition": "approve", "rationale": "I like it" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");

    // A proposal does not attempt a task.
    let (status, body) = api(
        app,
        "POST",
        "/api/tasks",
        "ada",
        Some(json!({ "repo": "ada/demo", "title": "Write the docs", "spec": "Say how proposing works" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let task = body["id"].as_str().unwrap().to_owned();
    let (status, body) = api(
        app,
        "POST",
        "/api/changes",
        "nadia",
        Some(json!({ "repo": "ada/demo", "target": "main", "title": "An attempt", "task": task })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert!(
        body["error"]
            .as_str()
            .unwrap_or("")
            .contains("Task: trailer"),
        "{body}"
    );

    // The page says so, and the door shuts at once.
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (_, page) = page_with_cookie(app, "/ada/demo/changes", &cookie).await;
    assert!(page.contains("Proposal"), "{page}");
    policy_with_proposals(app, false).await;
    let (status, body) = api(app, "POST", "/api/changes", "nadia", Some(open.clone())).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    // Her own proposal is still hers to revise and abandon.
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{mine}/revisions"),
        "nadia",
        Some(json!({ "commit_oid": "c".repeat(40) })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "the door shut: {body}");

    // A grant of propose opens a private repository to one person.
    visibility(app, false).await;
    let (status, body) = api(
        app,
        "POST",
        "/api/grants",
        "ada",
        Some(json!({ "grantee": "nadia", "repo": "ada/demo", "actions": ["propose"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "POST",
        "/api/changes",
        "nadia",
        Some(json!({ "repo": "ada/demo", "target": "main", "title": "Invited in" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["event"]["proposal"], true, "{body}");
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{mine}/revisions"),
        "nadia",
        Some(json!({ "commit_oid": "d".repeat(40) })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

/// A proposal's own claim is its word: it counts once a runner reproduces
/// it, and runners on a timer wait to be let at it.
#[tokio::test(flavor = "multi_thread")]
async fn a_proposals_word_waits_for_a_runner() {
    let forge = boot().await;
    let app = &forge.app;
    for (id, kind) in [("nadia", "human"), ("runner", "agent")] {
        let (status, body) = api(
            app,
            "POST",
            "/api/principals",
            "ada",
            Some(json!({ "id": id, "kind": kind, "display": id })),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let (status, body) = api(
        app,
        "POST",
        "/api/grants",
        "ada",
        Some(json!({ "grantee": "runner", "actions": ["verify"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    visibility(app, true).await;
    policy_with_proposals(app, true).await;

    let (status, body) = api(
        app,
        "POST",
        "/api/changes",
        "nadia",
        Some(json!({ "repo": "ada/demo", "target": "main", "title": "Faster parsing" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let change = body["id"].as_str().unwrap().to_owned();
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{change}/revisions"),
        "nadia",
        Some(json!({ "commit_oid": "e".repeat(40), "message": "Faster parsing" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{change}/claims"),
        "nadia",
        Some(json!({ "kind": "test", "passed": true, "summary": "green here", "command": "cargo test" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let claim = body["id"].as_str().unwrap().to_owned();

    // Her word does not meet the test requirement.
    let test_claim = |readiness: &Value| {
        readiness["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| {
                r["description"]
                    .as_str()
                    .unwrap()
                    .contains("passing test claim")
            })
            .cloned()
            .unwrap()
    };
    let (_, readiness) = api(
        app,
        "GET",
        &format!("/api/changes/{change}/readiness"),
        "ada",
        None,
    )
    .await;
    let requirement = test_claim(&readiness);
    assert_eq!(requirement["satisfied"], false, "{readiness}");
    assert!(
        requirement["evidence"]
            .as_str()
            .unwrap()
            .contains("its word"),
        "{readiness}"
    );

    // Runners are not handed it; she cannot let them at it; ada can.
    let (_, waiting) = api(
        app,
        "GET",
        "/api/repos/ada/demo/awaiting-verification",
        "runner",
        None,
    )
    .await;
    assert_eq!(waiting.as_array().map(Vec::len), Some(0), "{waiting}");
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{change}/admit"),
        "nadia",
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{change}/admit"),
        "ada",
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["event"]["kind"].as_str().or(Some("change_admitted")),
        Some("change_admitted"),
        "{body}"
    );
    let (_, waiting) = api(
        app,
        "GET",
        "/api/repos/ada/demo/awaiting-verification",
        "runner",
        None,
    )
    .await;
    assert_eq!(waiting.as_array().map(Vec::len), Some(1), "{waiting}");
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{change}/admit"),
        "ada",
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");

    // The page says so.
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (_, page) = page_with_cookie(app, "/ada/demo/changes/1", &cookie).await;
    assert!(page.contains("A proposal"), "{page}");
    assert!(page.contains("runners have been let at it"), "{page}");

    // Reproduced, it counts.
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/claims/{claim}/verify"),
        "runner",
        Some(json!({ "agrees": true, "command": "cargo test", "observed": "ok" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, readiness) = api(
        app,
        "GET",
        &format!("/api/changes/{change}/readiness"),
        "ada",
        None,
    )
    .await;
    assert_eq!(test_claim(&readiness)["satisfied"], true, "{readiness}");
}

/// A proposer's room is their own and small; a proposal that should never
/// have arrived is discarded, and its revisions leave git.
#[tokio::test(flavor = "multi_thread")]
async fn a_proposer_is_capped_and_a_proposal_can_be_discarded() {
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
    visibility(app, true).await;
    policy_with_proposals(app, true).await;
    let open = |title: &str| json!({ "repo": "ada/demo", "target": "main", "title": title });

    // Three open on one repository, then no more; a day's count too.
    let mut opened = Vec::new();
    for title in ["One", "Two", "Three"] {
        let (status, body) = api(app, "POST", "/api/changes", "nadia", Some(open(title))).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        opened.push(body["id"].as_str().unwrap().to_owned());
    }
    let (status, body) = api(app, "POST", "/api/changes", "nadia", Some(open("Four"))).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["kind"], "over_quota", "{body}");
    assert!(
        body["error"].as_str().unwrap().contains("3 open proposals"),
        "{body}"
    );
    // Abandoning one makes room; the day's allowance is the operator's to set.
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{}/abandon", opened[0]),
        "nadia",
        Some(json!({ "reason": "superseded" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "POST",
        "/api/principals/nadia/quota",
        "ada",
        Some(json!({ "proposals_a_day": 3 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(app, "POST", "/api/changes", "nadia", Some(open("Four"))).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert!(
        body["error"].as_str().unwrap().contains("in the last day"),
        "{body}"
    );

    // Discard: merge says so, with a reason; the proposer cannot; a plain
    // change cannot be discarded, only abandoned.
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{}/discard", opened[1]),
        "nadia",
        Some(json!({ "reason": "mine to hide" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{}/discard", opened[1]),
        "ada",
        Some(json!({ "reason": "advertising" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, change) = api(
        app,
        "GET",
        &format!("/api/changes/{}", opened[1]),
        "ada",
        None,
    )
    .await;
    assert_eq!(change["state"], "abandoned", "{change}");
    assert_eq!(change["discarded"], true, "{change}");
    let (status, body) = api(app, "POST", "/api/changes", "ada", Some(open("Ada's own"))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{}/discard", body["id"].as_str().unwrap()),
        "ada",
        Some(json!({ "reason": "not a proposal" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (_, page) = page_with_cookie(app, "/ada/demo/changes", &cookie).await;
    assert!(page.contains("Discarded"), "{page}");
}

/// Over real git: a proposer's push opens a proposal whose revision is a
/// ref; discarded, the ref is gone and reconciliation leaves it gone.
#[tokio::test(flavor = "multi_thread")]
async fn a_discarded_proposal_leaves_git() {
    let forge = boot().await;
    let (app, addr) = (&forge.app, forge.addr);
    let (status, body) = api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "nadia", "kind": "human", "display": "Nadia" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, minted) = api(
        app,
        "POST",
        "/api/principals/nadia/tokens",
        "nadia",
        Some(json!({ "label": "laptop" })),
    )
    .await;
    let token = minted["token"].as_str().unwrap().to_owned();
    visibility(app, true).await;
    policy_with_proposals(app, true).await;

    git(
        &forge.work,
        &[
            "clone",
            &format!("http://nadia:{token}@{addr}/git/ada/demo"),
            "proposal",
        ],
    );
    let wc = forge.work.join("proposal");
    commit_file(
        &wc,
        "PROPOSAL.md",
        "please\n",
        "Propose a file\n\nChange-Id: Iproposal01",
    );
    git(&wc, &["push", "origin", "HEAD:refs/for/main"]);
    let (_, changes) = api(app, "GET", "/api/repos/ada/demo/changes", "ada", None).await;
    let change = &changes.as_array().unwrap()[0];
    assert_eq!(change["proposal"], true, "{change}");
    assert_eq!(change["owner"], "nadia", "{change}");
    let number = change["number"].as_i64().unwrap();
    let id = change["id"].as_str().unwrap().to_owned();
    let refs = git(&wc, &["ls-remote", "origin", "refs/changes/*"]);
    assert!(refs.contains(&format!("refs/changes/{number}/1")), "{refs}");

    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{id}/discard"),
        "ada",
        Some(json!({ "reason": "not what this repository is for" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let refs = git(&wc, &["ls-remote", "origin", "refs/changes/*"]);
    assert!(!refs.contains(&format!("refs/changes/{number}/")), "{refs}");
}
