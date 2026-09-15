//! Saving a repository keeps it in reach: the sidebar lists what you
//! chose, and only what is yours to work in otherwise.

use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn a_saved_repository_sits_in_the_sidebar_until_let_go() {
    let forge = boot().await;
    let app = &forge.app;
    api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "bee", "kind": "human", "display": "Bee" })),
    )
    .await;
    api(
        app,
        "POST",
        "/api/repos/ada/demo/visibility",
        "ada",
        Some(json!({ "visibility": "public" })),
    )
    .await;
    // Bee may read ada/demo but it is not bee's: the sidebar does not
    // list it until bee saves it.
    let (_, bee) = sign_in_as(&forge, "bee").await;
    let (status, page) = page_with_cookie(app, "/ada/demo", &bee).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        page.contains(">Save<") || page.contains("Save</button>"),
        "{page}"
    );
    assert!(!page.contains(r#"class="item repo""#), "{page}");
    let (status, location) = post_form(app, "/ada/demo/save", &bee, "action=save").await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(location, "/ada/demo", "{location}");
    let (_, page) = page_with_cookie(app, "/ada/demo", &bee).await;
    assert!(page.contains("Saved"), "{page}");
    assert!(
        page.contains(r#"href="/ada/demo""#) && page.contains(">Saved<"),
        "{page}"
    );
    let (status, body) = api(app, "GET", "/api/you/bookmarks", "bee", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["saved"], json!(["ada/demo"]), "{body}");
    // Let it go, from the page.
    let (_, location) = post_form(app, "/ada/demo/save", &bee, "action=unsave").await;
    assert_eq!(location, "/ada/demo", "{location}");
    let (_, body) = api(app, "GET", "/api/you/bookmarks", "bee", None).await;
    assert_eq!(body["saved"], json!([]), "{body}");
    // What cannot be read cannot be saved, and looks like it is not there.
    api(
        app,
        "POST",
        "/api/repos/ada/demo/visibility",
        "ada",
        Some(json!({ "visibility": "private" })),
    )
    .await;
    let (status, body) = api(app, "POST", "/api/repos/ada/demo/bookmark", "bee", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    // Ada's own repository is in ada's sidebar without saving.
    let (_, ada) = sign_in_as(&forge, "ada").await;
    let (_, page) = page_with_cookie(app, "/", &ada).await;
    assert!(page.contains(r#"class="item repo"#), "{page}");
    // And the account panel says who ada is.
    let (_, page) = page_with_cookie(app, "/you/settings", &ada).await;
    assert!(
        page.contains(r#"id="account""#) && page.contains("<code>ada</code>"),
        "{page}"
    );
}
