//! The stylesheet is addressed by its content, so a deploy that changes
//! it changes the URL and nothing in between can serve a stale one.

use crate::common::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;

async fn get(app: &axum::Router, path: &str) -> (StatusCode, String, String) {
    let response = tower::ServiceExt::oneshot(
        app.clone(),
        Request::builder().uri(path).body(Body::empty()).unwrap(),
    )
    .await
    .unwrap();
    let status = response.status();
    let cache = response
        .headers()
        .get("cache-control")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let body = String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    (status, cache, body)
}

/// Status and the two headers that matter, for a body that is not text.
async fn head(app: &axum::Router, path: &str) -> (StatusCode, String, String) {
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
    (
        response.status(),
        header("cache-control"),
        header("content-type"),
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn the_page_links_a_hashed_stylesheet_that_may_be_cached_forever() {
    let forge = boot().await;
    let app = &forge.app;
    let (status, _, html) = get(app, "/login").await;
    assert_eq!(status, StatusCode::OK);
    let start = html.find("/assets/app.").expect("a stylesheet link");
    let end = html[start..].find(".css").unwrap() + start + 4;
    let href = &html[start..end];
    assert_ne!(href, "/assets/app.css", "the link carries a hash");

    let (status, cache, css) = get(app, href).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cache, "public, max-age=31536000, immutable");
    assert!(css.contains("max-width: 760px"), "the real stylesheet");

    // The bare name still answers, but must be revalidated every time.
    let (status, cache, _) = get(app, "/assets/app.css").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cache, "no-cache");

    // A hash that is not this binary's is not this binary's stylesheet.
    let (status, _, _) = get(app, "/assets/app.000000000000.css").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_fonts_come_from_the_forge_itself_under_hashed_names() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, _, html) = get(app, "/login").await;
    let start = html.find("/assets/app.").expect("a stylesheet link");
    let end = html[start..].find(".css").unwrap() + start + 4;
    let (_, _, css) = get(app, &html[start..end]).await;

    // Every face the stylesheet names is a hashed address on this origin,
    // and nothing on the page asks another host for anything.
    let mut fonts = 0;
    for face in css.split("@font-face").skip(1) {
        let url = face
            .split("url(")
            .nth(1)
            .expect("a src")
            .split(')')
            .next()
            .unwrap();
        assert!(
            url.starts_with("/assets/") && url.ends_with(".woff2"),
            "{url}"
        );
        assert!(
            !url.contains("{{"),
            "a placeholder was left unfilled: {url}"
        );
        let (status, cache, kind) = head(app, url).await;
        assert_eq!(status, StatusCode::OK, "{url}");
        assert_eq!(cache, "public, max-age=31536000, immutable");
        assert_eq!(kind, "font/woff2");
        fonts += 1;
    }
    assert_eq!(
        fonts, 3,
        "Familjen Grotesk, Instrument Sans, JetBrains Mono"
    );
    assert!(
        !css.contains("googleapis") && !css.contains("gstatic"),
        "no third party"
    );
    assert!(!html.contains("googleapis"), "no third party");

    // A font under a hash that is not this binary's is not served.
    let (status, _, _) = head(app, "/assets/instrument-sans.000000000000.woff2").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_content_policy_admits_the_fonts_and_nothing_else_new() {
    let forge = boot().await;
    let response = tower::ServiceExt::oneshot(
        forge.app.clone(),
        Request::builder()
            .uri("/login")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();
    let csp = response
        .headers()
        .get("content-security-policy")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();
    assert!(csp.contains("font-src 'self'"), "{csp}");
    assert!(csp.contains("default-src 'none'"), "{csp}");
    assert!(!csp.contains("unsafe"), "{csp}");
}
