//! Marks: a face for a person, a Lens for an agent, drawn from the id.

use crate::common::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn fetch(app: &axum::Router, path: &str) -> (StatusCode, String, String) {
    let request = Request::builder().uri(path).body(Body::empty()).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let cache = response
        .headers()
        .get("cache-control")
        .map(|v| v.to_str().unwrap_or("").to_owned())
        .unwrap_or_default();
    let body = String::from_utf8_lossy(&response.into_body().collect().await.unwrap().to_bytes())
        .into_owned();
    (status, cache, body)
}

#[tokio::test(flavor = "multi_thread")]
async fn marks_are_drawn_from_the_id_and_cached_for_good() {
    let forge = boot().await;
    let app = &forge.app;
    let (status, cache, ada) = fetch(app, "/avatars/1/person/ada.svg").await;
    assert_eq!(status, StatusCode::OK);
    assert!(cache.contains("immutable"), "{cache}");
    assert!(ada.starts_with("<svg"), "{ada}");
    let (_, _, again) = fetch(app, "/avatars/1/person/ada.svg").await;
    assert_eq!(ada, again, "the same id draws the same face");
    let (_, _, bee) = fetch(app, "/avatars/1/person/bee.svg").await;
    assert_ne!(ada, bee);

    let (status, _, scout) = fetch(app, "/avatars/1/agent/scout.svg").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!scout.contains("@keyframes"), "{scout}");
    let (_, _, live) = fetch(app, "/avatars/1/agent/scout.svg?live").await;
    assert!(live.contains("@keyframes"), "at work, the Lens blinks");

    // Nobody is looked up, so a stranger's id draws too; a wrong kind or
    // generation does not.
    let (status, _, _) = fetch(app, "/avatars/1/person/nobody-at-all.svg").await;
    assert_eq!(status, StatusCode::OK);
    let (status, _, _) = fetch(app, "/avatars/1/team/crew.svg").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = fetch(app, "/avatars/9/person/ada.svg").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Pages carry the marks as images; the agent at work is marked live.
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (status, page) = page_with_cookie(app, "/ada", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("/avatars/1/person/ada.svg"), "{page}");
    let (_, page) = page_with_cookie(app, "/agents", &cookie).await;
    assert!(page.contains("/avatars/1/agent/scout.svg"), "{page}");
}
