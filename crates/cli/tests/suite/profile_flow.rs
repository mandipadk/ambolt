//! A person's page carries their record: what the log says they did,
//! and one strip they wrote themselves.

use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;
use std::process::Command;

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

/// Who somebody is, asked over the API and from the terminal: what they
/// say, the time where they are, the agents in their name, their
/// record; an agent's model, harness and holder.
#[tokio::test(flavor = "multi_thread")]
async fn whois_says_who_somebody_is_over_the_api_and_the_binary() {
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
            "mark": 2
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // Anyone signed in may ask, the way anyone may read the page.
    let (status, who) = api(app, "GET", "/api/principals/ada/profile", "bee", None).await;
    assert_eq!(status, StatusCode::OK, "{who}");
    assert_eq!(who["kind"], "human", "{who}");
    assert_eq!(who["display"], "Ada Byron", "{who}");
    assert_eq!(who["says"]["line"], "Counting machines, mostly.", "{who}");
    assert_eq!(who["says"]["pronouns"], "she/her", "{who}");
    assert_eq!(who["says"]["zone"], "Europe/London", "{who}");
    let clock = who["says"]["local_time"].as_str().unwrap_or_default();
    assert!(
        clock.len() == 5 && clock.as_bytes()[2] == b':',
        "the time where they are, as a clock reads it: {who}"
    );
    assert_eq!(who["avatar"], "/avatars/2/person/ada.2.svg", "{who}");
    assert!(who["since"].is_string(), "{who}");
    assert_eq!(who["record"]["window_days"], 90, "{who}");
    let agents: Vec<&str> = who["agents"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|agent| agent["id"].as_str())
        .collect();
    assert!(agents.contains(&"scout"), "{who}");
    let (status, who) = api(
        app,
        "GET",
        "/api/principals/ada/profile?days=7",
        "bee",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{who}");
    assert_eq!(who["record"]["window_days"], 7, "{who}");

    // An agent: its model, its harness, whose it is; no strip of its own.
    let (status, who) = api(app, "GET", "/api/principals/scout/profile", "bee", None).await;
    assert_eq!(status, StatusCode::OK, "{who}");
    assert_eq!(who["kind"], "agent", "{who}");
    assert_eq!(who["owner"], "ada", "{who}");
    assert!(who["model"].is_string(), "{who}");
    assert_eq!(who["avatar"], "/avatars/2/agent/scout.svg", "{who}");
    assert!(who["agents"].as_array().unwrap().is_empty(), "{who}");
    let (status, _) = api(app, "GET", "/api/principals/nobody/profile", "bee", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // The same question from the terminal, as lines and as JSON.
    let server = format!("http://{}", forge.addr);
    let run = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_ambolt"))
            .args(["whois"])
            .args(args)
            .args(["--server", &server, "--token", &forge.ada_token])
            .output()
            .expect("run ambolt whois");
        (
            output.status.success(),
            format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        )
    };
    let (ok, out) = run(&["ada"]);
    assert!(ok, "{out}");
    assert!(out.starts_with("Ada Byron  @ada  person\n"), "{out}");
    assert!(
        out.contains("  says      Counting machines, mostly."),
        "{out}"
    );
    assert!(out.contains(" in Europe/London"), "{out}");
    assert!(out.contains("  agents    "), "{out}");
    assert!(out.contains(" landed, "), "{out}");
    let (ok, out) = run(&["scout"]);
    assert!(ok, "{out}");
    assert!(out.contains("@scout  agent in ada's name"), "{out}");
    assert!(out.contains("  model     "), "{out}");
    let (ok, out) = run(&["ada", "--json"]);
    assert!(ok, "{out}");
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("json out");
    assert_eq!(parsed["says"]["zone"], "Europe/London", "{out}");
    let (ok, out) = run(&["nobody"]);
    assert!(!ok, "{out}");
}

/// A person's record is signed like a receipt, checks offline, and fails
/// when a figure is changed; the page offers it in the rail.
#[tokio::test(flavor = "multi_thread")]
async fn a_record_is_signed_and_verifies_offline() {
    let forge = boot().await;
    let app = &forge.app;
    let (status, signed) = api(app, "GET", "/api/principals/ada/record/signed", "ada", None).await;
    assert_eq!(status, StatusCode::OK, "{signed}");
    assert_eq!(signed["record"]["principal"], "ada", "{signed}");
    assert_eq!(signed["record"]["display"], "Ada", "{signed}");
    assert_eq!(signed["record"]["window_days"], 90, "{signed}");
    assert!(signed["record"]["issued_at"].is_string(), "{signed}");
    assert_eq!(signed["signature"]["alg"], "ed25519", "{signed}");
    let key = signed["signature"]["key"].as_str().unwrap().to_owned();
    let (status, _) = api(
        app,
        "GET",
        "/api/principals/nobody/record/signed",
        "ada",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let verify = |name: &str, document: &serde_json::Value, key: Option<&str>| {
        let file = forge.work.join(name);
        std::fs::write(&file, document.to_string()).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_ambolt"));
        command.args(["record", "verify", file.to_str().unwrap()]);
        if let Some(key) = key {
            command.args(["--key", key]);
        }
        let output = command.output().expect("run ambolt record verify");
        (
            output.status.success(),
            format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        )
    };
    let (ok, out) = verify("ada-record.json", &signed, Some(&key));
    assert!(ok, "{out}");
    assert!(
        out.starts_with("verified  the record of Ada (@ada), person\n"),
        "{out}"
    );
    assert!(out.contains("  window    90 days, issued 20"), "{out}");
    assert!(out.contains("  key       "), "{out}");
    let (ok, out) = verify("ada-record.json", &signed, Some("0000000000000000"));
    assert!(!ok, "{out}");
    let mut forged = signed.clone();
    forged["record"]["landed"] = json!(999);
    let (ok, out) = verify("forged.json", &forged, None);
    assert!(!ok, "{out}");
    assert!(out.contains("does not match"), "{out}");

    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (_, page) = page_with_cookie(app, "/ada", &cookie).await;
    assert!(page.contains("/api/principals/ada/record/signed"), "{page}");
    assert!(page.contains("Verify this record"), "{page}");
}

/// A person picks up to four of their own landed changes, each with a
/// line; the page shows them in that order, and only what landed, only
/// theirs, at most four, none twice.
#[tokio::test(flavor = "multi_thread")]
async fn picked_changes_show_on_the_page_in_the_persons_words() {
    let forge = boot().await;
    let (app, addr) = (&forge.app, forge.addr);
    git(
        &forge.work,
        &[
            "clone",
            "-q",
            &format!("http://ada:x@{addr}/git/ada/demo"),
            "wc",
        ],
    );
    let wc = forge.work.join("wc");
    commit_file(
        &wc,
        "docs/pick.md",
        "# Pick\n",
        "Write the pick\n\nChange-Id: Ipick",
    );
    git(&wc, &["push", "-q", "origin", "HEAD:refs/for/main"]);
    let (_, changes) = api(app, "GET", "/api/repos/ada/demo/changes", "ada", None).await;
    let id = changes[0]["id"].as_str().unwrap().to_owned();
    assert_eq!(changes[0]["owner"], "ada", "{changes}");

    // Not landed yet: refused with the reason.
    let (status, body) = api(
        app,
        "PUT",
        "/api/you/picks",
        "ada",
        Some(json!({ "picks": [{ "change": id, "line": "Early." }] })),
    )
    .await;
    assert_ne!(status, StatusCode::OK, "{body}");
    assert!(
        body["error"].as_str().unwrap_or("").contains("not landed"),
        "{body}"
    );

    // Somebody other than the owner approves: bee, a person holding review.
    api(
        app,
        "POST",
        "/api/principals",
        "ada",
        Some(json!({ "id": "bee", "kind": "human", "display": "Bee" })),
    )
    .await;
    let (status, body) = api(
        app,
        "POST",
        "/api/grants",
        "ada",
        Some(json!({ "grantee": "bee", "actions": ["review"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{id}/claims"),
        "scout",
        Some(json!({ "kind": "test", "passed": true, "summary": "verified" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{id}/verdicts"),
        "bee",
        Some(json!({ "domain": "correctness", "disposition": "approve", "rationale": "Reviewed and correct." })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "POST",
        &format!("/api/changes/{id}/enqueue"),
        "ada",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    wait_for(app, "the change to land", async |app: &axum::Router| {
        let (_, c) = api(app, "GET", &format!("/api/changes/{id}"), "ada", None).await;
        c["state"] == "merged"
    })
    .await;
    let (status, body) = api(
        app,
        "PUT",
        "/api/you/picks",
        "ada",
        Some(json!({ "picks": [{ "change": id, "line": "The first thing I landed here." }] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["picked"][0]["change"], id, "{body}");
    assert_eq!(
        body["picked"][0]["line"], "The first thing I landed here.",
        "{body}"
    );

    // Five is too many; twice is once too often; scout may not pick for ada.
    let five: Vec<_> = (0..5).map(|_| json!({ "change": id })).collect();
    let (status, body) = api(
        app,
        "PUT",
        "/api/you/picks",
        "ada",
        Some(json!({ "picks": five })),
    )
    .await;
    assert_ne!(status, StatusCode::OK, "{body}");
    let (status, body) = api(
        app,
        "PUT",
        "/api/you/picks",
        "ada",
        Some(json!({ "picks": [{ "change": id }, { "change": id }] })),
    )
    .await;
    assert_ne!(status, StatusCode::OK, "{body}");
    assert!(
        body["error"].as_str().unwrap_or("").contains("twice"),
        "{body}"
    );
    let (status, body) = api(
        app,
        "PUT",
        "/api/you/picks",
        "scout",
        Some(json!({ "picks": [{ "change": id }] })),
    )
    .await;
    assert_ne!(status, StatusCode::OK, "an agent picks nothing: {body}");

    // The page shows it in her words, with the way to change it; so does
    // the profile over the API, to anyone.
    let (_, cookie) = sign_in_as(&forge, "ada").await;
    let (status, page) = page_with_cookie(app, "/ada", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("<h2>Picked</h2>"), "{page}");
    assert!(page.contains("The first thing I landed here."), "{page}");
    assert!(page.contains(r#"href="/you/picks""#), "{page}");
    let (_, who) = api(app, "GET", "/api/principals/ada/profile", "scout", None).await;
    assert_eq!(who["picked"][0]["change"], id, "{who}");
    let (_, mine) = api(app, "GET", "/api/you/profile", "ada", None).await;
    assert_eq!(
        mine["picked"][0]["line"], "The first thing I landed here.",
        "{mine}"
    );

    // The page to pick on lists what landed, and saves through the form.
    let (status, page) = page_with_cookie(app, "/you/picks", &cookie).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("Write the pick"), "{page}");
    assert!(
        page.contains(r#"value="The first thing I landed here.""#),
        "{page}"
    );
    let (status, _) = post_form_page(
        app,
        "/you/picks",
        &cookie,
        &format!("pick1={id}&line1=Said+again.&pick2=&line2="),
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let (_, page) = page_with_cookie(app, "/ada", &cookie).await;
    assert!(page.contains("Said again."), "{page}");

    // An empty list clears it, and the section invites her to pick.
    let (status, body) = api(
        app,
        "PUT",
        "/api/you/picks",
        "ada",
        Some(json!({ "picks": [] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["picked"].as_array().unwrap().is_empty(), "{body}");
    let (_, page) = page_with_cookie(app, "/ada", &cookie).await;
    assert!(!page.contains("Said again."), "{page}");
    assert!(page.contains(">Pick</a>"), "{page}");
}
