//! A name can be refused wherever one is chosen, and the refusal says
//! only that it is not available, except for a page's name.

use crate::common::*;
use crate::operator_flow::post_public_form;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn a_name_the_forge_keeps_or_nobody_gets_is_refused_everywhere() {
    let forge = boot_open_signup().await;
    let app = &forge.app;
    let sign_up = |name: &str| {
        format!(
            "name={name}&display={name}&email={name}%40example.test&password=long-enough-passphrase"
        )
    };
    // At sign-up: the forge's word, a shape that passes for it, and a
    // word nobody gets to be called, however it is spelt.
    for name in ["support", "ambolt-help", "n1gga"] {
        let (status, location) = post_public_form(app, "/signup", &sign_up(name)).await;
        assert_eq!(status, StatusCode::SEE_OTHER, "{name}");
        assert!(
            location.contains("not+available") || location.contains("not%20available"),
            "{name}: {location}"
        );
        assert!(
            !location.contains("reserved"),
            "{name}: says only that: {location}"
        );
    }
    // A page's name says why.
    let (_, location) = post_public_form(app, "/signup", &sign_up("login")).await;
    assert!(location.contains("reserved"), "{location}");
    // An ordinary name goes through.
    let (status, location) = post_public_form(app, "/signup", &sign_up("bee")).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert!(!location.contains("error="), "{location}");
    // Repositories and teams too, by name alone; on a forge that knows
    // its callers by name.
    let forge = boot().await;
    let app = &forge.app;
    api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "cat", "kind": "human", "display": "Cat" })),
    )
    .await;
    let (status, body) = api(
        app,
        "POST",
        "/api/repos",
        "cat",
        Some(json!({ "name": "official" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert!(
        body["error"]
            .as_str()
            .unwrap_or("")
            .contains("not available"),
        "{body}"
    );
    api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "crew", "kind": "team", "display": "Crew", "owner": "cat" })),
    )
    .await;
    let (status, body) = api(
        app,
        "POST",
        "/api/teams/crew/teams",
        "cat",
        Some(json!({ "name": "crew-staff" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    // Whoever runs the forge may take the forge's own words on purpose,
    // and still not the rest.
    let (status, body) = api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "security", "kind": "team", "display": "Security" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "f4ggot", "kind": "human", "display": "No" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
}
