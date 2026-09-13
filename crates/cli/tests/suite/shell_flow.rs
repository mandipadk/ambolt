//! Every page renders inside one frame: the sidebar, the head row and,
//! in a repository, its tabs. The tabs are named for what they are for,
//! the old addresses still answer, and the small script the frame
//! carries is served like every other asset.

use crate::common::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::json;

async fn asset(app: &axum::Router, path: &str) -> (StatusCode, String, String, String) {
    let response = tower::ServiceExt::oneshot(
        app.clone(),
        Request::builder().uri(path).body(Body::empty()).unwrap(),
    )
    .await
    .unwrap();
    let header = |name: &str| {
        response
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_owned()
    };
    let status = response.status();
    let cache = header("cache-control");
    let kind = header("content-type");
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        cache,
        kind,
        String::from_utf8_lossy(&bytes).into_owned(),
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn a_repository_has_tabs_named_for_what_they_are_for() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, ada) = sign_in_as(&forge, "ada").await;
    let (status, page) = page_with_cookie(app, "/ada/demo", &ada).await;
    assert_eq!(status, StatusCode::OK);
    for (href, label) in [
        ("/ada/demo/changes", "Changes"),
        ("/tasks?repo=ada/demo", "Tasks"),
        ("/ada/demo/review", "Review"),
        ("/ada/demo/coverage", "Coverage"),
        ("/ada/demo/activity", "Activity"),
        ("/ada/demo/settings", "Settings"),
    ] {
        let at = page
            .find(&format!(r#"class="tab" href="{href}""#))
            .unwrap_or_else(|| panic!("{href} is a tab: {page}"));
        assert!(
            page[at..at + 160].contains(label),
            "{href} is called {label}"
        );
    }
    assert!(
        page.contains(r#"class="tab on" href="/ada/demo""#),
        "the code tab is the one you are on: {page}"
    );
    // The head row says where you are, and the sidebar marks the repository.
    assert!(page.contains(r#"class="where""#), "{page}");
    assert!(
        page.contains(r#"class="item repo on" href="/ada/demo""#),
        "{page}"
    );

    // The old addresses stay good.
    for (old, new) in [
        ("/ada/demo/landing", "/ada/demo/review"),
        ("/ada/demo/debt", "/ada/demo/coverage"),
        ("/ada/demo/log", "/ada/demo/activity"),
    ] {
        let (status, location) = get_redirect(app, old, &ada).await;
        assert_eq!(status, StatusCode::PERMANENT_REDIRECT, "{old}");
        assert_eq!(location, new, "{old}");
    }
    for path in [
        "/ada/demo/review",
        "/ada/demo/coverage",
        "/ada/demo/activity",
    ] {
        let (status, page) = page_with_cookie(app, path, &ada).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert!(
            page.contains(&format!(r#"class="tab on" href="{path}""#)),
            "{path} marks its own tab: {page}"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn the_tasks_tab_shows_one_repository_s_tasks() {
    let forge = boot().await;
    let app = &forge.app;
    let (status, _) = api(
        app,
        "POST",
        "/api/repos",
        "ada",
        Some(json!({ "name": "other" })),
    )
    .await;
    assert!(status.is_success(), "{status}");
    for (title, repo) in [("Page the list", "ada/demo"), ("Elsewhere", "ada/other")] {
        let (status, _) = api(
            app,
            "POST",
            "/api/tasks",
            "ada",
            Some(json!({ "title": title, "spec": "Do it.", "repo": repo })),
        )
        .await;
        assert!(status.is_success(), "{title}: {status}");
    }
    let (_, ada) = sign_in_as(&forge, "ada").await;
    let (status, page) = page_with_cookie(app, "/tasks?repo=ada/demo", &ada).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("Page the list"), "{page}");
    assert!(!page.contains("Elsewhere"), "{page}");
    let (_, all) = page_with_cookie(app, "/tasks", &ada).await;
    assert!(
        all.contains("Page the list") && all.contains("Elsewhere"),
        "{all}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_frame_s_script_is_served_hashed_and_optional() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, ada) = sign_in_as(&forge, "ada").await;
    let (_, page) = page_with_cookie(app, "/", &ada).await;
    let start = page.find("/assets/app.").expect("the stylesheet");
    let start = page[start + 1..].find("/assets/app.").expect("the script") + start + 1;
    let end = page[start..].find(".js").unwrap() + start + 3;
    let href = &page[start..end];
    assert_ne!(href, "/assets/app.js", "the link carries a hash");
    let (status, cache, kind, body) = asset(app, href).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cache, "public, max-age=31536000, immutable");
    assert!(kind.starts_with("text/javascript"), "{kind}");
    assert!(body.contains("search.json"), "the real script");
    let (status, _, _, _) = asset(app, "/assets/app.000000000000.js").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Without the script the same page still has a search, a menu and a
    // way to switch the theme.
    assert!(page.contains(r#"href="/search""#), "{page}");
    assert!(page.contains(r#"<details class="me">"#), "{page}");
    assert!(page.contains(r#"action="/theme""#), "{page}");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_palette_asks_the_same_search_as_the_page() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, ada) = sign_in_as(&forge, "ada").await;
    let (status, body) = page_with_cookie(app, "/search.json?q=demo", &ada).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let hits = json["hits"].as_array().expect("hits");
    let repo = hits
        .iter()
        .find(|h| h["kind"] == "repository")
        .unwrap_or_else(|| panic!("the repository is a hit: {body}"));
    assert_eq!(repo["label"], "ada/demo");
    assert_eq!(repo["href"], "/ada/demo");

    // A stranger gets nothing from it.
    let (status, _) = page_with_cookie(app, "/search.json?q=demo", "").await;
    assert_ne!(status, StatusCode::OK);
}
