//! A person's page carries their record: what the log says they did,
//! and one strip they wrote themselves.

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
    assert!(page.contains("How this is counted"), "{page}");
    let (_, record) = api(app, "GET", "/api/principals/ada/record", "ada", None).await;
    assert_eq!(record["abandoned"], 1, "{record}");
    assert!(
        page.contains(r#"<span class="d">1 abandoned</span>"#),
        "{page}"
    );

    // An agent has one too; an organisation does not.
    let (_, page) = page_with_cookie(app, "/scout", &cookie).await;
    assert!(page.contains("<h2>Standing</h2>"), "{page}");
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
    assert!(!page.contains("<h2>Standing</h2>"), "{page}");
}

#[tokio::test(flavor = "multi_thread")]
async fn what_a_person_says_shows_on_their_page_and_in_the_log() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (status, body) = api(
        app,
        "PATCH",
        "/api/you/profile",
        "ada",
        Some(json!({
            "display": "Ada Byron",
            "line": "Counting machines, mostly.",
            "zone": "Europe/London",
            "pronouns": "she/her",
            "links": ["https://ada.example/"],
            "mark": 4
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["display"], "Ada Byron", "{body}");
    assert_eq!(body["mark"], 4, "{body}");
    assert_eq!(body["zone"], "Europe/London", "{body}");

    let (status, page) = page_with_cookie(app, "/ada", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("Ada Byron"), "{page}");
    assert!(page.contains("Counting machines, mostly."), "{page}");
    assert!(page.contains("Local time"), "{page}");
    assert!(page.contains("London"), "{page}");
    assert!(page.contains("she/her"), "{page}");
    assert!(page.contains("ada.example"), "{page}");
    assert!(page.contains("/avatars/2/person/ada.4.svg"), "{page}");
    // The sidebar draws the chosen mark too.
    let (_, home) = page_with_cookie(app, "/", &cookie).await;
    assert!(home.contains("/avatars/2/person/ada.4.svg"), "{home}");
    // The chosen drawing is served, and differs from the first.
    let (status, drawn) = page_with_cookie(app, "/avatars/2/person/ada.4.svg", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    let (_, first) = page_with_cookie(app, "/avatars/2/person/ada.svg", &cookie).await;
    assert_ne!(drawn, first);

    // An empty string clears a field; the rest stays.
    let (status, body) = api(
        app,
        "PATCH",
        "/api/you/profile",
        "ada",
        Some(json!({ "pronouns": "" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["pronouns"].is_null(), "{body}");
    assert_eq!(body["line"], "Counting machines, mostly.", "{body}");

    // A zone that is not one, and a link that is not a web address, are refused.
    let (status, body) = api(
        app,
        "PATCH",
        "/api/you/profile",
        "ada",
        Some(json!({ "zone": "Mars/Olympus" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    let (status, body) = api(
        app,
        "PATCH",
        "/api/you/profile",
        "ada",
        Some(json!({ "links": ["javascript:alert(1)"] })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");

    // Only a person says things about themself.
    let (status, body) = api(
        app,
        "PATCH",
        "/api/you/profile",
        "scout",
        Some(json!({ "line": "beep" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");

    // The log says who changed what, with their name on it.
    let (_, log) = page_with_cookie(app, "/log", &cookie).await;
    assert!(
        log.contains("changed what Ada Byron says about themself"),
        "{log}"
    );
    assert!(log.contains("set the name ada is shown as to"), "{log}");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_account_panel_is_the_same_strip_as_a_form() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (status, location) = post_form(
        app,
        "/you/settings/profile",
        &cookie,
        "display=Ada+Byron&line=Counting+machines.&zone=Asia%2FKathmandu&pronouns=&link1=https%3A%2F%2Fada.example&link2=&link3=&mark=2",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER, "{location}");
    assert_eq!(location, "/you/settings?done=1#account");
    let (_, settings) = page_with_cookie(app, "/you/settings", &cookie).await;
    assert!(settings.contains(r#"value="Ada Byron""#), "{settings}");
    assert!(
        settings.contains(r#"value="Counting machines.""#),
        "{settings}"
    );
    assert!(settings.contains(r#"value="2" checked"#), "{settings}");
    let (_, page) = page_with_cookie(app, "/ada", &cookie).await;
    assert!(page.contains("Kathmandu"), "{page}");
    assert!(page.contains("/avatars/2/person/ada.2.svg"), "{page}");

    // A bad zone comes back to the panel with the reason.
    let (_, location) = post_form(
        app,
        "/you/settings/profile",
        &cookie,
        "display=Ada+Byron&line=&zone=Nowhere&pronouns=&link1=&link2=&link3=&mark=2",
    )
    .await;
    assert!(location.starts_with("/you/settings?error="), "{location}");
}

/// A person's page is five: what stands, what landed, the agents in
/// their name, what they judged, and the allowance only they and the
/// operator see. Every figure is the log's; the diary shows a stranger
/// only what they may read.
#[tokio::test(flavor = "multi_thread")]
async fn a_persons_page_has_tabs_and_each_counts_from_the_log() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    // scout, ada's agent, opens a change; ada approves it; it lands.
    let (status, body) = api(
        app,
        "POST",
        "/api/changes",
        "scout",
        Some(json!({ "repo": "ada/demo", "target": "main", "title": "Scout's change" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let change = body["id"].as_str().unwrap().to_owned();
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{change}/revisions"),
        "scout",
        Some(json!({ "commit_oid": format!("{:0>40}", 7), "message": "Scout's change" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{change}/verdicts"),
        "ada",
        Some(json!({ "domain": "correctness", "disposition": "approve", "rationale": "Fine." })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, page) = page_with_cookie(app, "/ada", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains(r#"href="/ada/landed""#), "{page}");
    assert!(page.contains(r#"href="/ada/agents""#), "{page}");
    assert!(page.contains(r#"href="/ada/judgement""#), "{page}");
    assert!(
        page.contains(r#"href="/ada/allowance""#),
        "the holder sees the allowance tab"
    );
    assert!(page.contains("Landed by week"), "{page}");

    let (status, page) = page_with_cookie(app, "/ada/judgement", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("Looks given"), "{page}");
    assert!(
        page.contains(r#"<span class="v">1</span>"#),
        "one look, on scout's change: {page}"
    );

    let (status, page) = page_with_cookie(app, "/ada/agents", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        page.contains("Acting in Ada's name") || page.contains("Acting in"),
        "{page}"
    );
    assert!(page.contains(r#"href="/scout""#), "{page}");

    let (status, page) = page_with_cookie(app, "/ada/allowance", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("Repositories"), "{page}");

    // An agent's page: overview and landed, nothing else.
    let (status, page) = page_with_cookie(app, "/scout", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("Held by"), "{page}");
    assert!(!page.contains(r#"href="/scout/judgement""#), "{page}");
    let (status, _) = page_with_cookie(app, "/scout/judgement", &cookie).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Somebody else may not see the allowance.
    let (status, body) = api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "bee", "kind": "human", "display": "Bee" })),
    )
    .await;
    assert!(status.is_success(), "{body}");
    let (_, bee) = sign_in_as(&forge, "bee").await;
    let (status, page) = page_with_cookie(app, "/ada", &bee).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!page.contains(r#"href="/ada/allowance""#), "{page}");
    let (status, _) = page_with_cookie(app, "/ada/allowance", &bee).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // No repository may take a tab's name.
    let (status, body) = api(
        app,
        "POST",
        "/api/repos",
        "ada",
        Some(json!({ "name": "landed" })),
    )
    .await;
    assert!(!status.is_success(), "{body}");
}

/// Standing and Judgement count the last ninety days unless the reader
/// asks for everything since the person arrived; the tabs keep the
/// choice, and the head shows no chip for a person.
#[tokio::test(flavor = "multi_thread")]
async fn standing_counts_since_joining_when_asked() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (status, page) = page_with_cookie(app, "/ada", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        page.contains(r#"class="on" href="/ada">90 days</a>"#),
        "{page}"
    );
    assert!(
        page.contains(r#"href="/ada?since=joining">Since joining</a>"#),
        "{page}"
    );
    assert!(
        !page.contains(">Person</span>"),
        "no kind chip on a person: {page}"
    );
    assert!(page.contains(r#"class="av xl""#), "{page}");
    let (status, page) = page_with_cookie(app, "/ada?since=joining", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        page.contains(r#"class="on" href="/ada?since=joining">Since joining</a>"#),
        "{page}"
    );
    assert!(
        page.contains(r#"href="/ada/judgement?since=joining""#),
        "the tabs keep the window: {page}"
    );
    let (status, page) = page_with_cookie(app, "/ada/judgement?since=joining", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        page.contains(r#"class="on" href="/ada/judgement?since=joining">Since joining</a>"#),
        "{page}"
    );
}
