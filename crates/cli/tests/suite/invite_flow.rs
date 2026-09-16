//! Bringing a person in without handing them a terminal: an admin makes
//! a link, the link signs them in once, and then they set a password.

use crate::common::*;
use axum::http::StatusCode;

#[tokio::test(flavor = "multi_thread")]
async fn an_invitation_signs_somebody_in_exactly_once() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, ada) = sign_in_as(&forge, "ada").await;

    let (status, location) =
        post_form(app, "/people", &ada, "action=register&id=bee&display=Bee").await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    // The page shows the link once, as a link to this forge, and never
    // puts the secret in a URL of its own.
    let link = shown_once(app, &location, &ada).await;
    assert!(link.contains("/join?token="), "{link}");
    let secret = link.split("token=").nth(1).unwrap().to_owned();
    let (_, page) = page_with_cookie(app, "/people", &ada).await;
    assert!(page.contains("No password yet"));
    assert!(
        !page.contains(r#"class="repohead""#),
        "a section page is not a repository"
    );

    // Following it signs bee in and lands on the welcome page.
    let response = tower::ServiceExt::oneshot(
        app.clone(),
        axum::http::Request::builder()
            .uri(format!("/join?token={secret}"))
            .body(axum::body::Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers()["location"], "/welcome");
    let cookie = response.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let (status, page) = page_with_cookie(app, "/welcome", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("This is you"), "{page}");
    assert!(
        page.contains("<code>bee</code>"),
        "signed in as bee: {page}"
    );
    let (status, page) = page_with_cookie(app, "/welcome?step=2", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("Set a password"), "{page}");
    assert!(page.contains("Next time you sign in"), "the second step");

    // The link is spent.
    let response = tower::ServiceExt::oneshot(
        app.clone(),
        axum::http::Request::builder()
            .uri(format!("/join?token={secret}"))
            .body(axum::body::Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert!(
        response.headers()["location"]
            .to_str()
            .unwrap()
            .starts_with("/login?error="),
        "a used invitation is refused"
    );

    // Bee sets a password and can now sign in the ordinary way.
    let (status, location) = post_form(
        app,
        "/you/settings",
        &cookie,
        "password=a+perfectly+ordinary+password&confirm=a+perfectly+ordinary+password",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER, "{location}");
    // Changing a password ends every session, this one included.
    assert!(location.contains("Password+changed"), "{location}");
    let redirect = redirect_of(app, "bee", "a perfectly ordinary password").await;
    assert_eq!(
        redirect, "/welcome",
        "a password set from an invitation signs in, to the welcome page once"
    );
    let (_, page) = page_with_cookie(app, "/people", &ada).await;
    assert!(page.contains("Can sign in"), "{page}");
}

#[tokio::test(flavor = "multi_thread")]
async fn only_whoever_runs_the_forge_sees_people() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, ada) = sign_in_as(&forge, "ada").await;
    post_form(app, "/people", &ada, "action=register&id=bee&display=Bee").await;
    let (_, bee) = sign_in_as(&forge, "bee").await;
    assert_eq!(
        get_with_cookie(app, "/people", &bee).await,
        StatusCode::NOT_FOUND
    );
    let (status, _) = post_form(app, "/people", &bee, "action=register&id=cat&display=Cat").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    // A bare API token is a credential, not an invitation.
    let (_, page) = page_with_cookie(app, "/", &bee).await;
    assert!(
        !page.contains(r#"href="/people""#),
        "no People link for bee"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn an_invitation_can_be_cancelled_and_only_the_newest_link_works() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, ada) = sign_in_as(&forge, "ada").await;

    let (_, first) = post_form(app, "/people", &ada, "action=register&id=bee&display=Bee").await;
    let first = shown_once(app, &first, &ada)
        .await
        .split("token=")
        .nth(1)
        .unwrap()
        .to_owned();
    let (_, page) = page_with_cookie(app, "/people", &ada).await;
    assert!(page.contains("Invited, until"), "{page}");

    // A new link retires the old one.
    let (_, second) = post_form(app, "/people", &ada, "action=relink&id=bee").await;
    let second = shown_once(app, &second, &ada)
        .await
        .split("token=")
        .nth(1)
        .unwrap()
        .to_owned();
    let (status, location) = get_redirect(app, &format!("/join?token={first}"), "").await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert!(
        location.starts_with("/login?error="),
        "the old link is dead: {location}"
    );

    // Cancelling retires the newest too, and the page stops saying invited.
    post_form(app, "/people", &ada, "action=cancel&id=bee").await;
    let (_, location) = get_redirect(app, &format!("/join?token={second}"), "").await;
    assert!(location.starts_with("/login?error="), "{location}");
    let (_, page) = page_with_cookie(app, "/people", &ada).await;
    assert!(!page.contains("Invited, until"), "{page}");
}

/// One invitation at a time, whichever way it was asked for: the People
/// page's own button kills the link it replaces, the same as the API
/// does, because the minting revokes what is open rather than each
/// caller remembering to.
#[tokio::test(flavor = "multi_thread")]
async fn a_second_invitation_kills_the_first_link() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, ada) = sign_in_as(&forge, "ada").await;

    let (_, location) = post_form(app, "/people", &ada, "action=register&id=bee&display=Bee").await;
    let first = shown_once(app, &location, &ada).await;
    let first = first.split("token=").nth(1).unwrap().to_owned();

    let (_, location) = post_form(app, "/people", &ada, "action=relink&id=bee").await;
    let second = shown_once(app, &location, &ada).await;
    let second = second.split("token=").nth(1).unwrap().to_owned();
    assert_ne!(first, second, "a new invitation is a new link");

    let (status, where_to) = get_redirect(app, &format!("/join?token={first}"), "").await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert!(
        where_to.starts_with("/login?error="),
        "the link the second invitation replaced is dead: {where_to}"
    );
    let (status, where_to) = get_redirect(app, &format!("/join?token={second}"), "").await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(where_to, "/welcome", "the newest link works");
}

/// A name can be put right while nobody has been the account yet, and
/// only until then: once somebody has arrived, what they are shown as
/// is theirs, and an invitation does not rename them.
#[tokio::test(flavor = "multi_thread")]
async fn an_invitation_may_correct_the_name_until_somebody_arrives() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, ada) = sign_in_as(&forge, "ada").await;

    let (_, location) = post_form(app, "/people", &ada, "action=register&id=bee&display=Be").await;
    let _ = shown_once(app, &location, &ada).await;

    // Still nobody's: the invitation carries the better name.
    let (_, location) = post_form(
        app,
        "/people",
        &ada,
        "action=relink&id=bee&display=Bee%20Okoro",
    )
    .await;
    let link = shown_once(app, &location, &ada).await;
    let (_, page) = page_with_cookie(app, "/people", &ada).await;
    assert!(page.contains("Bee Okoro"), "{page}");

    // Bee arrives; the account is hers now.
    let secret = link.split("token=").nth(1).unwrap().to_owned();
    let (status, _) = get_redirect(app, &format!("/join?token={secret}"), "").await;
    assert_eq!(status, StatusCode::SEE_OTHER);

    // A later invitation leaves her name alone.
    let (_, _) = post_form(
        app,
        "/people",
        &ada,
        "action=relink&id=bee&display=Somebody%20Else",
    )
    .await;
    let (_, page) = page_with_cookie(app, "/people", &ada).await;
    assert!(page.contains("Bee Okoro"), "her name is hers: {page}");
    assert!(!page.contains("Somebody Else"), "{page}");
}

/// The welcome page asks once. Its steps may all be skipped, the door at
/// the end records that it has had its say, and the next sign-in goes
/// Home. An account that existed before the page did is asked too.
#[tokio::test(flavor = "multi_thread")]
async fn the_welcome_page_asks_once_and_every_step_may_be_skipped() {
    let forge = boot().await;
    let app = &forge.app;
    // ada has been here all along and was never welcomed: her next
    // sign-in lands there, once.
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (status, location) =
        crate::common::sign_in_redirect(app, "ada", "a perfectly ordinary password").await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(location, "/welcome");

    // Step one records the strip and the mark.
    let (status, location) = post_form(
        app,
        "/welcome",
        &cookie,
        "step=you&display=Ada+Byron&line=Counting+machines.&zone=Europe%2FLondon&pronouns=she%2Fher&link1=&mark=5",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER, "{location}");
    assert_eq!(location, "/welcome?step=2");
    let (_, page) = page_with_cookie(app, "/ada", &cookie).await;
    assert!(page.contains("Counting machines."), "{page}");
    assert!(page.contains("/avatars/2/person/ada.5.svg"), "{page}");

    // Step two: a password, and the person stays signed in on a new session.
    let (status, location) = post_form(
        app,
        "/welcome",
        &cookie,
        "step=password&password=twelve+letters+at+least&confirm=twelve+letters+at+least",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER, "{location}");
    assert_eq!(location, "/welcome?step=3");

    // Step three: a door, which marks the welcome had.
    let (_, cookie) = sign_in(app, "ada", "twelve letters at least").await;
    let cookie = cookie.unwrap().split(';').next().unwrap().to_owned();
    let (status, page) = page_with_cookie(app, "/welcome?step=3", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("What brought you here?"), "{page}");
    let (status, location) = post_form(app, "/welcome", &cookie, "step=door&door=agents").await;
    assert_eq!(status, StatusCode::SEE_OTHER, "{location}");
    assert_eq!(location, "/agents");
    let (_, body) = api(app, "GET", "/api/you/profile", "ada", None).await;
    assert_eq!(body["welcomed"], true, "{body}");

    // Welcomed: the next sign-in goes Home, and the page is still there
    // for anyone who asks for it.
    let (_, location) =
        crate::common::sign_in_redirect(app, "ada", "twelve letters at least").await;
    assert_eq!(location, "/");
    let (status, _) = page_with_cookie(app, "/welcome", &cookie).await;
    assert_eq!(status, StatusCode::OK);
}
