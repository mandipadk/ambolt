//! Mentions: @name in a thread lands in that person's inbox.

use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn a_mention_reaches_who_could_read_it_and_links_to_them() {
    let forge = boot().await;
    let app = &forge.app;
    // arbiter may read demo; nadia may not; nobody-here is nobody.
    let (status, body) = api(
        app,
        "POST",
        "/api/grants",
        "ada",
        Some(json!({ "grantee": "arbiter", "repo": "ada/demo", "actions": ["review"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
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
        "/api/changes",
        "scout",
        Some(json!({ "repo": "ada/demo", "target": "main", "title": "Mentioning" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let change = body["id"].as_str().unwrap().to_owned();
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{change}/revisions"),
        "scout",
        Some(json!({ "commit_oid": "a".repeat(40), "message": "Mentioning" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, opened) = api(
        app,
        "POST",
        &format!("/api/changes/{change}/threads"),
        "scout",
        Some(json!({
            "anchor": { "on": "change" }, "kind": "question",
            "body": "@arbiter could you look? cc @nadia, @nobody-here and mail@example.org; @scout too"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{opened}");
    let thread = opened["id"].as_str().unwrap().to_owned();

    let mentions = |inbox: &serde_json::Value| -> Vec<String> {
        inbox["notices"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["kind"] == "mentioned")
            .map(|n| n["what"].as_str().unwrap().to_owned())
            .collect()
    };
    let (_, inbox) = api(app, "GET", "/api/inbox", "arbiter", None).await;
    let got = mentions(&inbox);
    assert_eq!(got.len(), 1, "{inbox}");
    assert!(
        got[0].starts_with("scout mentioned you on #1: @arbiter could you look?"),
        "{inbox}"
    );
    let (_, inbox) = api(app, "GET", "/api/inbox", "nadia", None).await;
    assert!(
        mentions(&inbox).is_empty(),
        "a stranger to the repository hears nothing: {inbox}"
    );
    let (_, inbox) = api(app, "GET", "/api/inbox", "scout", None).await;
    assert!(
        mentions(&inbox).is_empty(),
        "nobody mentions themselves: {inbox}"
    );

    // A reply mentions too, once per person.
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/threads/{thread}/reply"),
        "ada",
        Some(json!({ "body": "@arbiter @arbiter yes please" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, inbox) = api(app, "GET", "/api/inbox", "arbiter", None).await;
    assert_eq!(mentions(&inbox).len(), 2, "{inbox}");

    // On the page, the name is a link to the person.
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (_, page) = page_with_cookie(app, "/ada/demo/changes/1", &cookie).await;
    assert!(
        page.contains(r#"<a class="mention" href="/arbiter">@arbiter</a>"#),
        "{page}"
    );
    assert!(page.contains("mail@example.org"), "{page}");
    let (_, page) = page_with_cookie(app, "/inbox", &cookie).await;
    assert!(
        page.contains("Mention") || !page.contains("mentioned"),
        "{page}"
    );
}
