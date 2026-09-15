//! Reports: anyone says what broke, whoever runs the forge hears of it.

use crate::common::*;
use axum::http::StatusCode;

fn link_in(mail: &str) -> String {
    let start = mail.find("http").expect("a link in the mail");
    let end = mail[start..]
        .find(char::is_whitespace)
        .map_or(mail.len(), |i| start + i);
    mail[start..end].to_owned()
}

fn path_of(link: &str) -> String {
    let after = link.split_once("://").map_or(link, |(_, rest)| rest);
    after
        .find('/')
        .map_or("/".to_owned(), |i| after[i..].to_owned())
}

#[tokio::test(flavor = "multi_thread")]
async fn a_stranger_reports_and_whoever_runs_the_forge_hears_of_it() {
    let outbox = tempfile::tempdir().unwrap();
    let mail_file = outbox.path().join("mail.txt");
    let forge = boot_mailing(&format!("cat > '{}'", mail_file.display())).await;
    let app = &forge.app;

    // ada runs the forge; once her address is confirmed, reports reach it.
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (status, location) = post_form(
        app,
        "/you/settings/email",
        &cookie,
        "email=ada%40example.org",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER, "{location}");
    let confirm = std::fs::read_to_string(&mail_file).expect("a confirmation was mailed");
    let (status, _) = get_redirect(app, &path_of(&link_in(&confirm)), &cookie).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    std::fs::remove_file(&mail_file).unwrap();

    // The form answers a stranger.
    let (status, page) = page_with_cookie(app, "/report", "").await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("Say what broke"), "{page}");

    let (status, location) = post_form(
        app,
        "/report",
        "",
        "what=The+landing+page+said+nothing+after+I+pushed+a+stack.&place=%2Fada%2Fdemo%2Flanding&contact=Someone%40Example.test",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER, "{location}");
    assert_eq!(location, "/report?filed=1");
    let (_, page) = page_with_cookie(app, "/report?filed=1", "").await;
    assert!(page.contains("Recorded as report 1"), "{page}");

    // It reached ada by mail, with the version and the address.
    let mail = std::fs::read_to_string(&mail_file).expect("the report was mailed");
    assert!(mail.contains("To: ada@example.org"), "{mail}");
    assert!(mail.contains("Subject: ambolt report 1"), "{mail}");
    assert!(mail.contains("landing page said nothing"), "{mail}");
    assert!(mail.contains("someone@example.test"), "{mail}");
    assert!(mail.contains(ambolt_core::VERSION), "{mail}");

    // And it is on her page, and on nobody else's.
    let (status, page) = page_with_cookie(app, "/reports", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("landing page said nothing"), "{page}");
    assert!(page.contains("someone@example.test"), "{page}");
    assert!(page.contains("/ada/demo/landing"), "{page}");
    assert_ne!(
        get_with_cookie(app, "/reports", "").await,
        StatusCode::OK,
        "a stranger does not read reports"
    );

    // Dismissed, it is gone from the list and the store.
    let (status, location) = post_form(app, "/reports", &cookie, "id=1").await;
    assert_eq!(status, StatusCode::SEE_OTHER, "{location}");
    assert!(forge.state.reports().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_report_needs_words_and_a_real_address_if_any() {
    let forge = boot_token_only().await;
    let app = &forge.app;

    let (status, location) = post_form(app, "/report", "", "what=broken").await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert!(location.starts_with("/report?error="), "{location}");

    let (_, location) = post_form(
        app,
        "/report",
        "",
        "what=Something+long+enough+to+be+a+report&contact=not-an-address",
    )
    .await;
    assert!(location.starts_with("/report?error="), "{location}");
    assert!(forge.state.reports().unwrap().is_empty());

    // Signed in, the report remembers who said it.
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (_, location) = post_form(
        app,
        "/report",
        &cookie,
        "what=The+debt+page+takes+a+while+on+a+big+repository",
    )
    .await;
    assert_eq!(location, "/report?filed=1");
    let reports = forge.state.reports().unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].by.as_deref(), Some("ada"));
    assert_eq!(reports[0].contact, None);
    assert_eq!(reports[0].version, ambolt_core::VERSION);
}

/// A repository or a person is reported from their own page, and
/// whoever runs the forge hides or stops them from the reports page.
#[tokio::test(flavor = "multi_thread")]
async fn a_place_is_reported_and_hidden_or_stopped() {
    let forge = boot().await;
    let app = &forge.app;
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (status, _) = api(
        app,
        "POST",
        "/api/repos/ada/demo/visibility",
        "ada",
        Some(serde_json::json!({ "visibility": "public" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // The public page links to the form about itself; a private one
    // has nobody to show it to.
    let (_, page) = page_with_cookie(app, "/ada/demo", "").await;
    assert!(
        page.contains("/report?kind=abuse&amp;place=%2Fada%2Fdemo"),
        "{page}"
    );
    let (_, page) = page_with_cookie(app, "/report?kind=abuse&place=%2Fada%2Fdemo", "").await;
    assert!(page.contains("Report this"), "{page}");
    assert!(page.contains("value=\"/ada/demo\""), "{page}");

    let (status, location) = post_form(
        app,
        "/report",
        "",
        "kind=abuse&what=This+repository+is+a+copy+of+mine+with+my+name+filed+off.&place=%2Fada%2Fdemo",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER, "{location}");
    let reports = forge.state.reports().unwrap();
    assert_eq!(reports[0].kind, "abuse");

    // The reports page says so and offers to hide it; hidden means private.
    let (_, page) = page_with_cookie(app, "/reports", &cookie).await;
    assert!(page.contains("Abuse"), "{page}");
    assert!(page.contains("value=\"hide\""), "{page}");
    let (status, location) = post_form(
        app,
        "/reports",
        &cookie,
        "id=1&action=hide&target=%2Fada%2Fdemo",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER, "{location}");
    assert_eq!(location, "/reports");
    let (_, repo) = api(app, "GET", "/api/repos/ada/demo", "ada", None).await;
    assert_eq!(repo["visibility"], "private", "{repo}");
    let (_, page) = page_with_cookie(app, "/ada/demo", "").await;
    assert!(!page.contains("kind=abuse"), "{page}");
    // The report stays until dismissed.
    assert_eq!(forge.state.reports().unwrap().len(), 1);
    post_form(app, "/reports", &cookie, "id=1&action=dismiss").await;
    assert!(forge.state.reports().unwrap().is_empty());

    // A person is reported from their page and stopped.
    let (_, page) = page_with_cookie(app, "/scout", &cookie).await;
    assert!(
        page.contains("/report?kind=abuse&amp;place=%2Fscout"),
        "{page}"
    );
    let (_, page) = page_with_cookie(app, "/ada", &cookie).await;
    assert!(
        !page.contains("kind=abuse"),
        "nobody reports themselves: {page}"
    );
    post_form(
        app,
        "/report",
        &cookie,
        "kind=abuse&what=scout+keeps+opening+changes+full+of+advertising.&place=%2Fscout",
    )
    .await;
    let (_, page) = page_with_cookie(app, "/reports", &cookie).await;
    assert!(page.contains("value=\"stop\""), "{page}");
    let (status, location) =
        post_form(app, "/reports", &cookie, "id=2&action=stop&target=%2Fscout").await;
    assert_eq!(status, StatusCode::SEE_OTHER, "{location}");
    assert_eq!(location, "/reports");
    let (_, scout) = api(app, "GET", "/api/principals/scout", "ada", None).await;
    assert_eq!(scout["active"], false, "{scout}");

    // A bug report about a place offers nothing but dismissal.
    post_form(
        app,
        "/report",
        "",
        "what=The+log+page+is+blank+today.&place=%2Fada%2Fdemo",
    )
    .await;
    let (_, page) = page_with_cookie(app, "/reports", &cookie).await;
    assert!(page.contains("value=\"dismiss\""), "{page}");
    assert_eq!(page.matches("value=\"hide\"").count(), 0, "{page}");

    // Nobody but whoever runs the forge acts from here.
    let (_, location) =
        post_form(app, "/reports", "", "id=3&action=hide&target=%2Fada%2Fdemo").await;
    assert_ne!(location, "/reports", "a stranger is sent to sign in");
    assert_eq!(forge.state.reports().unwrap().len(), 2);
}
