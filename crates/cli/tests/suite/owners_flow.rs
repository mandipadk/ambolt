//! An owner's page, and an organisation as an owner.

use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn an_owner_page_shows_what_the_reader_may_see() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, ada) = sign_in_as(&forge, "ada").await;
    let (status, page) = page_with_cookie(app, "/ada", &ada).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains(r#"href="/ada/demo""#), "{page}");

    // With nothing public, the name is not confirmed to a stranger: an
    // owner who exists and one who does not look the same.
    let (status, _) = get_redirect(app, "/ada", "").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = get_redirect(app, "/nobody", "").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Once something is public, the stranger sees that and nothing else.
    api(
        app,
        "POST",
        "/api/repos",
        "ada",
        Some(json!({ "name": "secret" })),
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
    let (status, page) = page_with_cookie(app, "/ada", "").await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains(r#"href="/ada/demo""#), "{page}");
    assert!(!page.contains("secret"), "{page}");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_organisation_owns_what_its_members_make_under_it() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, ada) = sign_in_as(&forge, "ada").await;
    api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "bee", "kind": "human", "display": "Bee" })),
    )
    .await;
    let (_, location) = post_form(app, "/teams", &ada, "action=create&id=crew&display=Crew").await;
    assert_eq!(location, "/teams", "{location}");
    let (_, location) = post_form(app, "/teams", &ada, "action=add&team=crew&member=bee").await;
    assert_eq!(location, "/teams", "{location}");

    // Bee, a member and not running the forge, creates under crew from New.
    let (_, bee) = sign_in_as(&forge, "bee").await;
    let (_, new) = page_with_cookie(app, "/new", &bee).await;
    assert!(
        new.contains(r#"value="crew""#),
        "New offers the organisation: {new}"
    );
    let (status, location) = post_form(
        app,
        "/new",
        &bee,
        "owner=crew&name=shared&default_branch=main",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(location, "/crew/shared", "{location}");
    let (_, repo) = api(app, "GET", "/api/repos/crew/shared", "ada", None).await;
    assert_eq!(repo["owner"], "crew");

    // The organisation's page lists it and its people; bee governs it as
    // an owner would, and a member of nothing may not create under it.
    let (status, page) = page_with_cookie(app, "/crew", &bee).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        page.contains(r#"href="/crew/shared""#) && page.contains("bee"),
        "{page}"
    );
    assert_eq!(
        get_with_cookie(app, "/crew/shared/settings", &bee).await,
        StatusCode::OK
    );
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
        Some(json!({ "name": "mine", "owner": "crew" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    let (status, body) = api(
        app,
        "POST",
        "/api/repos",
        "cat",
        Some(json!({ "name": "mine", "owner": "ada" })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "not under another person either: {body}"
    );

    // An offer to an organisation is answered by a member, and the
    // repository takes the organisation's name.
    api(
        app,
        "POST",
        "/api/repos/ada/demo/transfer",
        "ada",
        Some(json!({ "to": "crew" })),
    )
    .await;
    let (status, body) = api(
        app,
        "POST",
        "/api/repos/ada/demo/transfer/accept",
        "bee",
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, repo) = api(app, "GET", "/api/repos/crew/demo", "bee", None).await;
    assert_eq!(repo["owner"], "crew");
    assert_eq!(repo["name"], "crew/demo");
}

/// An organisation's members change who is on it; nobody but the
/// operator empties it.
#[tokio::test(flavor = "multi_thread")]
async fn owners_run_the_organisation_and_the_last_owner_stays() {
    let forge = boot().await;
    let app = &forge.app;
    for (id, display) in [("bee", "Bee"), ("cat", "Cat"), ("dan", "Dan")] {
        api(
            app,
            "POST",
            "/api/principals",
            "ada",
            Some(json!({ "id": id, "kind": "human", "display": display })),
        )
        .await;
    }
    // Whoever runs the forge makes the organisation and names who runs it.
    let (_, ada) = sign_in_as(&forge, "ada").await;
    let (status, location) = post_form(
        app,
        "/teams",
        &ada,
        "action=create&id=crew&display=Crew&owner=bee",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(location, "/teams", "{location}");
    let (_, page) = page_with_cookie(app, "/teams", &ada).await;
    assert!(page.contains("crew") && page.contains("owner"), "{page}");
    // Bee, its owner, brings cat in from the organisation's page, and the
    // sidebar says what bee is to it.
    let (_, bee) = sign_in_as(&forge, "bee").await;
    let (status, location) = post_form(app, "/crew/members", &bee, "action=add&member=cat").await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(location, "/crew", "{location}");
    let (_, page) = page_with_cookie(app, "/crew", &bee).await;
    assert!(page.contains(r#"href="/cat""#), "{page}");
    assert!(
        page.contains("Add member") && page.contains("Make owner"),
        "{page}"
    );
    assert!(page.contains("Organisations"), "{page}");
    // Cat, a member, creates under it, but changes nobody's standing.
    let (status, body) = api(
        app,
        "POST",
        "/api/repos",
        "cat",
        Some(json!({ "name": "shared", "owner": "crew" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, cat) = sign_in_as(&forge, "cat").await;
    let (_, location) = post_form(app, "/crew/members", &cat, "action=add&member=dan").await;
    assert!(location.contains("error="), "{location}");
    let (_, page) = page_with_cookie(app, "/crew", &cat).await;
    assert!(!page.contains("Add member"), "{page}");
    assert!(page.contains("Leave"), "{page}");
    // Dan is not on it and changes nothing; the page shows no form.
    let (_, dan) = sign_in_as(&forge, "dan").await;
    let (_, location) = post_form(app, "/crew/members", &dan, "action=add&member=dan").await;
    assert!(location.contains("error="), "{location}");
    // Bee cannot leave: the last owner stays until another is named.
    let (_, location) = post_form(app, "/crew/members", &bee, "action=remove&member=bee").await;
    assert!(
        location.contains("last+owner") || location.contains("last%20owner"),
        "{location}"
    );
    // Cat may leave, being a member; then bee brings dan in, names dan an
    // owner, and may leave.
    let (_, location) = post_form(app, "/crew/members", &cat, "action=remove&member=cat").await;
    assert_eq!(location, "/crew", "{location}");
    post_form(app, "/crew/members", &bee, "action=add&member=dan").await;
    let (_, location) = post_form(app, "/crew/members", &bee, "action=owner&member=dan").await;
    assert_eq!(location, "/crew", "{location}");
    let (_, location) = post_form(app, "/crew/members", &bee, "action=remove&member=bee").await;
    assert_eq!(location, "/crew", "{location}");
    // The API says the same, on the public listener too: an owner's act,
    // not the door's.
    let (public, _) = split_listeners(&forge);
    let (status, body) = api(
        &public,
        "POST",
        "/api/teams/crew/members",
        "dan",
        Some(json!({ "member": "cat" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(&public, "GET", "/api/teams/crew/members", "cat", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["owners"], json!(["dan"]), "{body}");
    let (status, body) = api(
        &public,
        "POST",
        "/api/teams/crew/owners/remove",
        "dan",
        Some(json!({ "member": "dan" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    // Whoever runs the forge may empty it, the last owner included.
    for member in ["cat", "dan"] {
        let (status, body) = api(
            app,
            "POST",
            "/api/teams/crew/members/remove",
            "ada",
            Some(json!({ "member": member })),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
}

/// An owner brings people onto the forge straight into the organisation:
/// the account, the membership and the link, with nobody at the door.
#[tokio::test(flavor = "multi_thread")]
async fn an_owner_invites_people_into_the_organisation() {
    let forge = boot().await;
    let app = &forge.app;
    for (id, display) in [("bee", "Bee"), ("cat", "Cat")] {
        api(
            app,
            "POST",
            "/api/principals",
            "ada",
            Some(json!({ "id": id, "kind": "human", "display": display })),
        )
        .await;
    }
    api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "crew", "kind": "team", "display": "Crew", "owner": "bee" })),
    )
    .await;
    api(
        app,
        "POST",
        "/api/teams/crew/members",
        "bee",
        Some(json!({ "member": "cat" })),
    )
    .await;
    // From the page: the link is parked and shown once, since the test
    // forge cannot mail; the page says eve is invited.
    let (_, bee) = sign_in_as(&forge, "bee").await;
    let (status, location) = post_form(
        app,
        "/crew/members",
        &bee,
        "action=invite&member=eve&display=Eve&email=eve%40example.org",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert!(location.starts_with("/crew?once="), "{location}");
    let (_, page) = page_with_cookie(app, &location, &bee).await;
    let start = page.find("/join?token=").expect("the link is on the page");
    let token: String = page[start + "/join?token=".len()..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    assert!(
        page.contains(r#"href="/eve""#) && page.contains("Invited"),
        "{page}"
    );
    // Shown once: the same address again shows no link.
    let (_, again) = page_with_cookie(app, &location, &bee).await;
    assert!(!again.contains("/join?token="), "{again}");
    // Eve follows it and is signed in, a member of crew.
    let (status, headers_or_body) =
        page_with_cookie(app, &format!("/join?token={token}"), "").await;
    assert!(
        status == StatusCode::SEE_OTHER || status == StatusCode::OK,
        "{status} {headers_or_body}"
    );
    let (status, body) = api(app, "GET", "/api/teams/crew/members", "bee", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body["members"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m == "eve"),
        "{body}"
    );
    // A member cannot invite; the API says so on the public listener,
    // and an owner may, from there too.
    let (public, _) = split_listeners(&forge);
    let (status, body) = api(
        &public,
        "POST",
        "/api/teams/crew/invitations",
        "cat",
        Some(json!({ "id": "fay", "email": "fay@example.org" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    let (status, body) = api(
        &public,
        "POST",
        "/api/teams/crew/invitations",
        "bee",
        Some(json!({ "id": "fay", "display": "Fay", "email": "fay@example.org" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["organisation"], "crew", "{body}");
    assert!(
        body["link"]
            .as_str()
            .is_some_and(|l| l.contains("/join?token=")),
        "{body}"
    );
    // The invited count against the organisation's members.
    let (_, body) = api(app, "GET", "/api/principals/crew/quota", "bee", None).await;
    assert_eq!(body["usage"]["members"], 4, "{body}");
}
