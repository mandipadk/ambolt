//! Watching a repository: hear when something lands, or needs a person.

use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;

/// A real commit pushed by scout, so the landing can advance main.
fn open(forge: &Forge, wc: &std::path::Path, title: &str, key: &str) {
    commit_file(
        wc,
        &format!("{key}.txt"),
        "watched\n",
        &format!("{title}\n\nChange-Id: I{key}"),
    );
    git(wc, &["push", "origin", "HEAD:refs/for/main"]);
    let _ = forge;
}

async fn latest_change(app: &axum::Router) -> String {
    let (_, changes) = api(app, "GET", "/api/repos/ada/demo/changes", "ada", None).await;
    changes
        .as_array()
        .unwrap()
        .iter()
        .max_by_key(|c| c["number"].as_i64().unwrap())
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_watcher_hears_of_a_landing_and_can_stop() {
    let forge = boot().await;
    let (app, addr) = (&forge.app, forge.addr);
    git(
        &forge.work,
        &[
            "clone",
            &format!("http://scout:{}@{addr}/git/ada/demo", forge.scout_token),
            "wc",
        ],
    );
    let wc = forge.work.join("wc");
    // arbiter is neither the change's owner nor the one who lands it;
    // given a way to read demo, it watches.
    let (status, body) = api(
        app,
        "POST",
        "/api/grants",
        "ada",
        Some(json!({ "grantee": "arbiter", "repo": "ada/demo", "actions": ["review"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(app, "POST", "/api/repos/ada/demo/watch", "arbiter", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["watching"], true, "{body}");
    let (_, mine) = api(app, "GET", "/api/you/watches", "arbiter", None).await;
    assert_eq!(mine["watches"], json!(["ada/demo"]), "{mine}");
    // Twice is once.
    let (status, _) = api(app, "POST", "/api/repos/ada/demo/watch", "arbiter", None).await;
    assert_eq!(status, StatusCode::OK);

    open(&forge, &wc, "First", "watch0001");
    let first = latest_change(app).await;
    approve_and_merge(app, &first).await;
    let (_, inbox) = api(app, "GET", "/api/inbox", "arbiter", None).await;
    let landed: Vec<&serde_json::Value> = inbox["notices"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "landed")
        .collect();
    assert_eq!(landed.len(), 1, "{inbox}");
    assert!(
        landed[0]["what"]
            .as_str()
            .unwrap()
            .contains("landed on main in ada/demo"),
        "{inbox}"
    );

    // Stopped, the next landing is not theirs to hear.
    let (status, body) = api(app, "POST", "/api/repos/ada/demo/unwatch", "arbiter", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    open(&forge, &wc, "Second", "watch0002");
    let second = latest_change(app).await;
    approve_and_merge(app, &second).await;
    let (_, inbox) = api(app, "GET", "/api/inbox", "arbiter", None).await;
    let landed = inbox["notices"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "landed")
        .count();
    assert_eq!(landed, 1, "{inbox}");

    // A stranger to a private repository is told nothing.
    let (status, body) = api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "nadia", "kind": "human", "display": "Nadia" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, _) = api(app, "POST", "/api/repos/ada/demo/watch", "nadia", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // From the page: the button, and its state.
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (_, page) = page_with_cookie(app, "/ada/demo", &cookie).await;
    assert!(page.contains(r#"value="watch""#), "{page}");
    let (status, location) = post_form(app, "/ada/demo/watch", &cookie, "action=watch").await;
    assert_eq!(status, StatusCode::SEE_OTHER, "{location}");
    let (_, page) = page_with_cookie(app, "/ada/demo", &cookie).await;
    assert!(
        page.contains("Watching") && page.contains(r#"value="unwatch""#),
        "{page}"
    );
    let (_, page) = page_with_cookie(app, "/ada/demo/activity", &cookie).await;
    assert!(page.contains("started watching"), "{page}");
}
