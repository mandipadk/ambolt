//! A person's page carries their record: what the log says they did.

use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn a_persons_page_carries_the_record_and_an_organisations_does_not() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    // Something on the record: a change ada opened and abandoned.
    let (status, body) = api(
        app,
        "POST",
        "/api/changes",
        "ada",
        Some(json!({ "repo": "ada/demo", "target": "main", "title": "A thought" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{}/abandon", body["id"].as_str().unwrap()),
        "ada",
        Some(json!({ "reason": "second thoughts" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, page) = page_with_cookie(app, "/ada", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        page.contains("the last 90 days, counted from the log"),
        "{page}"
    );
    let (_, record) = api(app, "GET", "/api/principals/ada/record", "ada", None).await;
    assert_eq!(record["abandoned"], 1, "{record}");
    assert!(
        page.contains(r#"<div class="v">1</div><div class="k">abandoned</div>"#),
        "{page}"
    );

    // An agent has one too; an organisation does not.
    let (_, page) = page_with_cookie(app, "/scout", &cookie).await;
    assert!(page.contains("counted from the log"), "{page}");
    let (status, body) = api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "crew", "kind": "team", "display": "Crew" })),
    )
    .await;
    assert!(status.is_success(), "{body}");
    let (status, page) = page_with_cookie(app, "/crew", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!page.contains("counted from the log"), "{page}");
}
