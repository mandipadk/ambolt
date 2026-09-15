//! Topics on a repository, and Explore: where a stranger starts.

use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn topics_are_set_searched_and_explored() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, cookie) = sign_in_as(&forge, "ada").await;

    // Set over the API: normalised, refused when not a topic, capped.
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/topics",
        "ada",
        Some(json!({ "topics": ["Rust", "forge", " agents ", "rust"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, repo) = api(app, "GET", "/api/repos/ada/demo", "ada", None).await;
    assert_eq!(repo["topics"], json!(["rust", "forge", "agents"]), "{repo}");
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/topics",
        "ada",
        Some(json!({ "topics": ["Not A Topic!"] })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    let nine: Vec<String> = (0..9).map(|i| format!("t{i}")).collect();
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/topics",
        "ada",
        Some(json!({ "topics": nine })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/topics",
        "scout",
        Some(json!({ "topics": ["mine"] })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");

    // Searchable, and on the repository's page and settings.
    let (status, page) = page_with_cookie(app, "/search?q=forge", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("ada/demo"), "{page}");
    let (_, page) = page_with_cookie(app, "/ada/demo", &cookie).await;
    assert!(page.contains("/explore?topic=agents"), "{page}");
    let (status, location) = post_form(
        app,
        "/ada/demo/settings/topics",
        &cookie,
        "topics=rust%2C+tools",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER, "{location}");
    let (_, repo) = api(app, "GET", "/api/repos/ada/demo", "ada", None).await;
    assert_eq!(repo["topics"], json!(["rust", "tools"]), "{repo}");

    // Explore: private repositories are not on it; public ones are, to
    // anyone, filtered by topic.
    let (_, page) = page_with_cookie(app, "/explore", "").await;
    assert!(page.contains("No public repository yet"), "{page}");
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/visibility",
        "ada",
        Some(json!({ "visibility": "public" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, page) = page_with_cookie(app, "/explore", "").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        page.contains("ada/demo") && page.contains("#rust"),
        "{page}"
    );
    let (_, page) = page_with_cookie(app, "/explore?topic=tools", "").await;
    assert!(page.contains("ada/demo"), "{page}");
    let (_, page) = page_with_cookie(app, "/explore?topic=python", "").await;
    assert!(page.contains("Nothing is filed under that"), "{page}");
    let (status, body) = api(app, "GET", "/api/explore?topic=rust", "ada", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["repositories"][0]["name"], "ada/demo", "{body}");
    assert_eq!(body["repositories"][0]["landed_week"], 0, "{body}");
    // Signed in, Explore is in the sidebar; so is it for a stranger.
    let (_, page) = page_with_cookie(app, "/explore", &cookie).await;
    assert!(page.contains(r#"href="/explore""#), "{page}");
}
