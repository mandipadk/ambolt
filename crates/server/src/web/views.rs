//! Every page as a maud template. All interpolation is escaped by
//! maud; the one deliberate exception is README markdown, which is
//! rendered with raw HTML events stripped before it gets here.

use super::diff::{FileDiff, LineKind};
use super::{Chrome, LandingData, Sidebar, Viewer};
use ambolt_core::{
    Anchor, Independence, Resolution, ReviewDomain, Session, SessionState, Side, TaskState, Thread,
    ThreadKind, Waiver,
};
use ambolt_core::{
    BrowserSession, Change, ChangeState, Claim, Contact, Disposition, Envelope, Event, HitKind,
    Notice, Origin, OriginKind, OriginState, PasskeyRecord, PolicyTrace, PrincipalId, Repo,
    Revision, Settlement, Task, Verdict, Verification, Visibility,
};
use maud::{DOCTYPE, Markup, PreEscaped, html};

/// The forge's mark: an anvil in the page's ink, drawn once and used by
/// every shell. Face, horn to the left, waist, foot; 18 by 13 so it sits
/// on the wordmark's x-height.
fn mark() -> Markup {
    PreEscaped(
        r#"<svg class="mark" width="18" height="13" viewBox="0 0 18 13" aria-hidden="true"><path d="M0 2.2C0 1.6 0.4 1 1.2 1H17c0.6 0 1 0.4 1 1v2.4c0 0.5-0.4 0.9-0.9 0.9H10.5v3.6h3.1c0.5 0 0.9 0.4 0.9 0.9v2.3c0 0.5-0.4 0.9-0.9 0.9H4.4c-0.5 0-0.9-0.4-0.9-0.9v-2.3c0-0.5 0.4-0.9 0.9-0.9h3.1V5.3H4.6C2 5.3 0.6 4.3 0 2.2z"/></svg>"#
            .to_owned(),
    )
}
use std::collections::HashMap;

/// Which palette the page renders in. Dark is the default; a viewer
/// can switch, and the choice rides in a cookie.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Theme {
    /// No choice made: the page follows the system's preference.
    System,
    Dark,
    Light,
}

impl Theme {
    /// The root attribute a chosen theme stamps; none when following the
    /// system, so the stylesheet's `prefers-color-scheme` rule decides.
    pub fn attr(self) -> Option<&'static str> {
        match self {
            Theme::System => None,
            Theme::Dark => Some("dark"),
            Theme::Light => Some("light"),
        }
    }
}

/// Every icon the pages use, once per page, referenced by `ic`. Adding an
/// icon is adding a symbol here; there is no icon font and no per-icon
/// request.
const SPRITE: &str = r##"<svg class="sprite" xmlns="http://www.w3.org/2000/svg" aria-hidden="true"><symbol id="i-home" viewBox="0 0 24 24"><path d="M3 11l9-8 9 8v9a1 1 0 0 1-1 1h-5v-6h-6v6H4a1 1 0 0 1-1-1z"/></symbol><symbol id="i-inbox" viewBox="0 0 24 24"><path d="M4 4h16v16H4z" rx="2"/><path d="M4 13h5l2 3h2l2-3h5"/></symbol><symbol id="i-tasks" viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="3"/><path d="M8 12l3 3 5-6"/></symbol><symbol id="i-agents" viewBox="0 0 24 24"><path d="M12 2v3"/><rect x="4" y="7" width="16" height="13" rx="3"/><path d="M9 13h.01M15 13h.01M9 17h6"/></symbol><symbol id="i-repo" viewBox="0 0 24 24"><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></symbol><symbol id="i-search" viewBox="0 0 24 24"><circle cx="11" cy="11" r="7"/><path d="M20 20l-3.5-3.5"/></symbol><symbol id="i-plus" viewBox="0 0 24 24"><path d="M12 5v14M5 12h14"/></symbol><symbol id="i-settings" viewBox="0 0 24 24"><path d="M4 7h10M18 7h2M4 17h4M12 17h8"/><circle cx="16" cy="7" r="2"/><circle cx="10" cy="17" r="2"/></symbol><symbol id="i-code" viewBox="0 0 24 24"><path d="M8 7l-5 5 5 5M16 7l5 5-5 5"/></symbol><symbol id="i-changes" viewBox="0 0 24 24"><circle cx="6" cy="6" r="2.5"/><circle cx="6" cy="18" r="2.5"/><circle cx="18" cy="18" r="2.5"/><path d="M6 8.5v7M18 15.5V10a3 3 0 0 0-3-3h-3M13 4l-3 3 3 3"/></symbol><symbol id="i-review" viewBox="0 0 24 24"><path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7-10-7-10-7z"/><circle cx="12" cy="12" r="3"/></symbol><symbol id="i-coverage" viewBox="0 0 24 24"><path d="M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6z"/><path d="M9 12l2 2 4-4"/></symbol><symbol id="i-activity" viewBox="0 0 24 24"><path d="M3 12h4l3-7 4 14 3-7h4"/></symbol><symbol id="i-chev" viewBox="0 0 24 24"><path d="M6 9l6 6 6-6"/></symbol><symbol id="i-file" viewBox="0 0 24 24"><path d="M6 3h8l5 5v13a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1z"/><path d="M14 3v5h5"/></symbol><symbol id="i-folder" viewBox="0 0 24 24"><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></symbol><symbol id="i-tag" viewBox="0 0 24 24"><path d="M3 12V5a2 2 0 0 1 2-2h7l9 9-9 9z"/><circle cx="8" cy="8" r="1"/></symbol><symbol id="i-clock" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/></symbol><symbol id="i-branch" viewBox="0 0 24 24"><path d="M6 3v12"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="6" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/></symbol><symbol id="i-user" viewBox="0 0 24 24"><circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/></symbol><symbol id="i-key" viewBox="0 0 24 24"><circle cx="8" cy="15" r="4"/><path d="M11 12l9-9M17 6l3 3M14 9l3 3"/></symbol><symbol id="i-sun" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4 12h2M18 12h2M5 5l1.5 1.5M17.5 17.5L19 19M5 19l1.5-1.5M17.5 6.5L19 5"/></symbol><symbol id="i-moon" viewBox="0 0 24 24"><path d="M21 13A9 9 0 1 1 11 3a7 7 0 0 0 10 10z"/></symbol><symbol id="i-terminal" viewBox="0 0 24 24"><path d="M4 17l6-5-6-5M12 19h8"/></symbol><symbol id="i-check" viewBox="0 0 24 24"><path d="M5 12l5 5 9-11"/></symbol><symbol id="i-x" viewBox="0 0 24 24"><path d="M6 6l12 12M18 6L6 18"/></symbol><symbol id="i-alert" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9"/><path d="M12 8v4M12 16h.01"/></symbol><symbol id="i-more" viewBox="0 0 24 24"><circle cx="5" cy="12" r="1.3"/><circle cx="12" cy="12" r="1.3"/><circle cx="19" cy="12" r="1.3"/></symbol><symbol id="i-copy" viewBox="0 0 24 24"><rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15V5a2 2 0 0 1 2-2h10"/></symbol><symbol id="i-bell" viewBox="0 0 24 24"><path d="M6 8a6 6 0 0 1 12 0v5l2 3H4l2-3z"/><path d="M10 20a2 2 0 0 0 4 0"/></symbol><symbol id="i-play" viewBox="0 0 24 24"><path d="M6 4l14 8-14 8z"/></symbol><symbol id="i-lock" viewBox="0 0 24 24"><rect x="4" y="11" width="16" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/></symbol><symbol id="i-globe" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3a15 15 0 0 1 0 18a15 15 0 0 1 0-18"/></symbol><symbol id="i-rerun" viewBox="0 0 24 24"><path d="M21 12a9 9 0 1 1-3-6.7"/><path d="M21 3v6h-6"/></symbol><symbol id="i-message" viewBox="0 0 24 24"><path d="M21 12a8 8 0 0 1-11.6 7.1L4 20l1.1-4.3A8 8 0 1 1 21 12z"/></symbol><symbol id="i-sparkle" viewBox="0 0 24 24"><path d="M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z"/></symbol><symbol id="i-download" viewBox="0 0 24 24"><path d="M12 4v11M7 10l5 5 5-5M4 19h16"/></symbol><symbol id="i-shield" viewBox="0 0 24 24"><path d="M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6z"/></symbol><symbol id="i-menu" viewBox="0 0 24 24"><path d="M4 7h16M4 12h16M4 17h16"/></symbol><symbol id="i-logout" viewBox="0 0 24 24"><path d="M14 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2v-2M9 12h11M17 9l3 3-3 3"/></symbol><symbol id="i-palette" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9"/><circle cx="8" cy="10" r="1.2"/><circle cx="12" cy="7" r="1.2"/><circle cx="16" cy="10" r="1.2"/><path d="M12 21a3 3 0 0 0 0-6h-1a1.5 1.5 0 0 1 0-3h1"/></symbol><symbol id="i-star" viewBox="0 0 24 24"><path d="M12 3l2.8 5.7 6.2.9-4.5 4.4 1.1 6.2L12 17.3 6.4 20.2l1.1-6.2L3 9.6l6.2-.9z"/></symbol><symbol id="i-send" viewBox="0 0 24 24"><path d="M21 3L10 14M21 3l-7 18-4-7-7-4z"/></symbol><symbol id="i-archive" viewBox="0 0 24 24"><rect x="3" y="4" width="18" height="5" rx="1"/><path d="M5 9v10a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V9M10 13h4"/></symbol><symbol id="i-receipt" viewBox="0 0 24 24"><path d="M6 3h12v18l-3-2-3 2-3-2-3 2z"/><path d="M9 8h6M9 12h6"/></symbol><symbol id="i-bookmark" viewBox="0 0 24 24"><path d="M6 3h12v18l-6-4-6 4z"/></symbol></svg>"##;

fn sprite() -> Markup {
    PreEscaped(SPRITE.to_owned())
}

/// A line icon from the sprite, sized by class: `ic` alone is 18 px,
/// `ic sm` 15, `ic lg` 22.
pub fn ic(name: &str, size: &str) -> Markup {
    html! {
        svg class={ "ic" @if !size.is_empty() { " " (size) } } aria-hidden="true" {
            use href={ "#i-" (name) } {}
        }
    }
}

/// Who did a thing, at a glance: initials in a circle for a person, in a
/// squircle with a live dot for an agent. The hue comes from the id so it
/// is the same on every page and every day.
pub fn avatar(id: &str, display: &str, agent: bool, live: bool) -> Markup {
    let kind = if agent {
        ambolt_core::PrincipalKind::Agent
    } else {
        ambolt_core::PrincipalKind::Human
    };
    avatar_of(id, display, kind, live)
}

/// The mark for a principal of a known kind: a face for a person, a Lens
/// for an agent (blinking while at work), a lettered tile for an
/// organisation, which is many people and has no face of its own.
pub fn avatar_of(id: &str, display: &str, kind: ambolt_core::PrincipalKind, live: bool) -> Markup {
    let drawn = super::avatars::GENERATION;
    match kind {
        ambolt_core::PrincipalKind::Agent => html! {
            img class={ "av agent" @if live { " live" } } alt="" title=(display)
                src={ "/avatars/" (drawn) "/agent/" (id) ".svg" @if live { "?live" } };
        },
        ambolt_core::PrincipalKind::Human => html! {
            img class="av" alt="" title=(display) src={ "/avatars/" (drawn) "/person/" (id) ".svg" };
        },
        ambolt_core::PrincipalKind::Team => html! {
            span class="av org" title=(display) { (initials(display)) }
        },
    }
}

/// The first letter of the first two words, or the first two letters of
/// a single word, upper-cased.
fn initials(display: &str) -> String {
    let words: Vec<&str> = display.split_whitespace().collect();
    let picked: String = match words.as_slice() {
        [] => "?".to_owned(),
        [one] => one.chars().take(2).collect(),
        [first, second, ..] => first
            .chars()
            .take(1)
            .chain(second.chars().take(1))
            .collect(),
    };
    picked.to_uppercase()
}

/// One line of a file, with everything the graph knows about it.
pub struct BlameRow {
    pub number: usize,
    pub text: String,
    pub provenance: Option<std::sync::Arc<ambolt_core::Provenance>>,
}

/// One row of a tree listing, with the change that last touched it.
pub struct Entry {
    pub is_dir: bool,
    pub name: String,
    pub subject: Option<String>,
    pub change: Option<Change>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Tab {
    Code,
    Changes,
    Community,
    Tasks,
    Review,
    Coverage,
    Activity,
    /// The lessons page has no tab of its own; it is reached from Tasks.
    Lessons,
    Settings,
}

fn layout(
    theme: Theme,
    viewer: Option<&Viewer>,
    repo: Option<&str>,
    active: Option<Tab>,
    title: &str,
    body: Markup,
) -> Markup {
    layout_with(theme, viewer, repo, active, title, body, None)
}

/// The frame every signed-in page renders inside: a global bar, a
/// sidebar of what you have and who is working, the page itself, and
/// optionally a rail on the right.
fn layout_with(
    theme: Theme,
    viewer: Option<&Viewer>,
    repo: Option<&str>,
    active: Option<Tab>,
    title: &str,
    body: Markup,
    rail: Option<Markup>,
) -> Markup {
    frame(theme, viewer, repo, None, active, title, body, rail, None)
}

/// Who is reading a repository page: somebody signed in, or nobody,
/// who sees a public repository with the forge's public chrome and no
/// way to act.
#[derive(Clone, Copy)]
pub enum Reading<'a> {
    Signed(&'a Viewer),
    Anonymous(&'a Chrome),
}

impl<'a> Reading<'a> {
    pub fn viewer(&self) -> Option<&'a Viewer> {
        match self {
            Reading::Signed(viewer) => Some(viewer),
            Reading::Anonymous(_) => None,
        }
    }

    fn chrome(&self) -> &'a Chrome {
        match self {
            Reading::Signed(viewer) => &viewer.1,
            Reading::Anonymous(chrome) => chrome,
        }
    }
}

fn layout_reading(
    theme: Theme,
    who: Reading<'_>,
    repo: Option<&str>,
    active: Option<Tab>,
    title: &str,
    body: Markup,
) -> Markup {
    frame_in(
        theme,
        Some(who),
        repo,
        None,
        active,
        title,
        body,
        None,
        None,
    )
}

fn layout_reading_with(
    theme: Theme,
    who: Reading<'_>,
    repo: Option<&str>,
    active: Option<Tab>,
    title: &str,
    body: Markup,
    rail: Option<Markup>,
) -> Markup {
    frame_in(
        theme,
        Some(who),
        repo,
        None,
        active,
        title,
        body,
        rail,
        None,
    )
}

/// A page that belongs to a section of the sidebar - the inbox, people,
/// teams - rather than to a repository. It highlights its entry in the
/// sidebar and renders no repository header, because it is not one.
fn layout_section(
    theme: Theme,
    viewer: &Viewer,
    section: &str,
    title: &str,
    body: Markup,
) -> Markup {
    frame(
        theme,
        Some(viewer),
        None,
        Some(section),
        None,
        title,
        body,
        None,
        None,
    )
}

/// What a page adds to its head row beside the title: a count, and the
/// actions that belong to the page. The title itself is the head row's,
/// so the page does not say it twice.
#[derive(Default)]
pub struct Head {
    pub count: Option<String>,
    pub acts: Option<Markup>,
}

fn layout_section_head(
    theme: Theme,
    viewer: &Viewer,
    section: &str,
    title: &str,
    head: Head,
    body: Markup,
) -> Markup {
    frame(
        theme,
        Some(viewer),
        None,
        Some(section),
        None,
        title,
        body,
        None,
        Some(head),
    )
}

#[allow(clippy::too_many_arguments)]
fn frame(
    theme: Theme,
    viewer: Option<&Viewer>,
    repo: Option<&str>,
    section: Option<&str>,
    active: Option<Tab>,
    title: &str,
    body: Markup,
    rail: Option<Markup>,
    head: Option<Head>,
) -> Markup {
    frame_in(
        theme,
        viewer.map(Reading::Signed),
        repo,
        section,
        active,
        title,
        body,
        rail,
        head,
    )
}

#[allow(clippy::too_many_arguments)]
fn frame_in(
    theme: Theme,
    who: Option<Reading<'_>>,
    repo: Option<&str>,
    section: Option<&str>,
    active: Option<Tab>,
    title: &str,
    body: Markup,
    rail: Option<Markup>,
    head: Option<Head>,
) -> Markup {
    let head = head.unwrap_or_default();
    html! {
        (DOCTYPE)
        html lang="en" data-theme=[theme.attr()] {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " · ambolt" }
                link rel="stylesheet" href=(super::stylesheet_href());
                script defer src=(super::script_href()) {}
                script defer src=(super::app_script_href()) {}
            }
            body {
                (sprite())
                @match who {
                    Some(who) => {
                        @let viewer = who.viewer();
                        div class="app" {
                            a class="skip" href="#content" { "Skip to content" }
                            (sidebar(theme, who, section.or(repo)))
                            main class="main" {
                                (headrow(viewer, repo, section, title, &head))
                                @if let Some(repo) = repo {
                                    (repohead(repo, active, viewer, who.chrome()))
                                }
                                div class={ "cols" @if rail.is_some() { " railed" } } {
                                    div class="content" id="content" { (body) }
                                    @if let Some(rail) = rail {
                                        aside class="rail" { (rail) }
                                    }
                                }
                            }
                        }
                    }
                    None => (body),
                }
            }
        }
    }
}

/// The sidebar: what is yours, where you can go, who is at work, and
/// who you are. Every signed-in page renders inside it, so it is the
/// product's shape rather than one page's.
fn sidebar(theme: Theme, who: Reading<'_>, current: Option<&str>) -> Markup {
    let chrome = who.chrome();
    let viewer = who.viewer();
    let on = |key: &str| if current == Some(key) { " on" } else { "" };
    let here = |key: &str| (current == Some(key)).then_some("page");
    let only = |key: &str| (current == Some(key)).then_some("on");
    let chosen = |t: Theme| (theme == t).then_some("on");
    html! {
        nav class="side" id="nav" {
            a class="org" href="/" aria-label="Home" {
                span class="orgmark" { (mark()) }
                span class="name" { "ambolt" }
            }
            @if viewer.is_some() {
                a class="search" href="/search" id="palette-open" {
                    (ic("search", "sm"))
                    span { "Search or jump to" }
                    kbd { "⌘K" }
                }
                div class="nav" {
                    a class={ "item" (on("home")) } aria-current=[here("home")] href="/" { (ic("home", "")) span { "Home" } }
                    a class={ "item" (on("inbox")) } aria-current=[here("inbox")] href="/inbox" {
                        (ic("inbox", "")) span { "Inbox" }
                        @if chrome.unread > 0 { span class="badge" { (chrome.unread) } }
                    }
                    a class={ "item" (on("tasks")) } aria-current=[here("tasks")] href="/tasks" { (ic("tasks", "")) span { "Tasks" } }
                    a class={ "item" (on("agents")) } aria-current=[here("agents")] href="/agents" { (ic("agents", "")) span { "Agents" } }
                    a class={ "item" (on("explore")) } aria-current=[here("explore")] href="/explore" { (ic("globe", "")) span { "Explore" } }
                }
            } @else {
                div class="nav" {
                    a class={ "item" (on("explore")) } aria-current=[here("explore")] href="/explore" { (ic("globe", "")) span { "Explore" } }
                }
            }
            h4 {
                "Repositories"
                @if viewer.is_some() { a class="plus" href="/new" aria-label="New repository" title="New repository" { (ic("plus", "sm")) } }
            }
            @if chrome.repos.len() > 4 {
                @let owners: Vec<&str> = {
                    let mut seen: Vec<&str> = Vec::new();
                    for repo in &chrome.repos {
                        let owner = repo.name.split('/').next().unwrap_or("");
                        if !seen.contains(&owner) { seen.push(owner); }
                    }
                    seen
                };
                div class="repofilter" id="repofilter" {
                    input class="filter" type="search" placeholder="Filter repositories" aria-label="Filter repositories" autocomplete="off";
                    @if owners.len() > 1 {
                        div class="owners" role="group" aria-label="Owner" {
                            button type="button" class="on" data-owner="" { "All" }
                            @for owner in owners { button type="button" data-owner=(owner) { (owner) } }
                        }
                    }
                }
            }
            div class="nav" id="repolist" {
                @if chrome.repos.is_empty() {
                    div class="none" { "None yet" }
                }
                @for repo in &chrome.repos {
                    a class={ "item repo" @if current == Some(repo.name.as_str()) { " on" } } href={ "/" (repo.name) } data-owner=(repo.name.split('/').next().unwrap_or("")) {
                        (ic("repo", ""))
                        span class="t" { (repo.name) }
                        @if repo.open > 0 { span class="n" { (repo.open) } }
                    }
                }
                div class="none" id="repolist-none" hidden { "No repository matches" }
            }
            @if !chrome.orgs.is_empty() {
                h4 { "Organisations" }
                div class="nav" {
                    @for (org, role) in &chrome.orgs {
                        a class={ "item" @if current == Some(org.as_str()) { " on" } } href={ "/" (org) } {
                            (ic("agents", ""))
                            span class="t" { (org) }
                            @if *role == ambolt_core::TeamRole::Owner { span class="n" { "owner" } }
                        }
                    }
                }
            }
            @if !chrome.saved.is_empty() {
                h4 { "Saved" }
                div class="nav" {
                    @for repo in &chrome.saved {
                        a class={ "item repo" @if current == Some(repo.as_str()) { " on" } } href={ "/" (repo) } {
                            (ic("bookmark", ""))
                            span class="t" { (repo) }
                        }
                    }
                }
            }
            @if !chrome.working.is_empty() {
                h4 { "At work now" }
                div class="nav" {
                    @for worker in &chrome.working {
                        div class="who" title=(worker.paths.join(", ")) {
                            (avatar(&worker.who, &worker.display, worker.agent, true))
                            span class="t" {
                                b { (worker.display) }
                                @if let Some(repo) = &worker.repo {
                                    span { (repo) @if let Some(path) = worker.paths.first() { ", in " code { (path) } } }
                                }
                            }
                        }
                    }
                }
            }
            @if chrome.admin {
                h4 { "Operator" }
                div class="nav" {
                    a class={ "item" (on("people")) } aria-current=[here("people")] href="/people" { (ic("user", "")) span { "People" } }
                    a class={ "item" (on("teams")) } aria-current=[here("teams")] href="/teams" { (ic("agents", "")) span { "Teams" } }
                    a class={ "item" (on("reports")) } aria-current=[here("reports")] href="/reports" { (ic("alert", "")) span { "Reports" } }
                    a class={ "item" (on("log")) } aria-current=[here("log")] href="/log" { (ic("activity", "")) span { "Forge log" } }
                }
            }
            @match viewer {
                Some(viewer) => {
                    details class="me" {
                        summary {
                            (avatar(viewer.0.as_str(), &chrome.display, false, false))
                            span class="t" {
                                b { (chrome.display) }
                                span { (viewer.0.as_str()) @if chrome.admin { " · operator" } }
                            }
                            (ic("chev", "sm"))
                        }
                        div class="pop" {
                            a class=[only("you")] href="/you" { (ic("changes", "sm")) "Your changes" @if chrome.yours > 0 { span class="n" { (chrome.yours) } } }
                            a href="/you/settings#tokens" { (ic("key", "sm")) "Tokens" }
                            a href="/you/settings#sessions" { (ic("globe", "sm")) "Sessions" }
                            a class=[only("settings")] href="/you/settings" { (ic("settings", "sm")) "Settings" }
                            div class="lab" { "Theme" }
                            form class="seg" method="post" action="/theme" {
                                button class=[chosen(Theme::Light)] type="submit" name="to" value="light" { (ic("sun", "sm")) "Light" }
                                button class=[chosen(Theme::Dark)] type="submit" name="to" value="dark" { (ic("moon", "sm")) "Dark" }
                                button class=[chosen(Theme::System)] type="submit" name="to" value="system" { "Auto" }
                            }
                            form method="post" action="/logout" {
                                button class="danger" type="submit" { (ic("logout", "sm")) "Sign out" }
                            }
                        }
                    }
                }
                None => {
                    div class="me anon" {
                        a class="btn2 sm" href="/login" { "Sign in" }
                        form class="seg" method="post" action="/theme" {
                            button class=[chosen(Theme::Light)] type="submit" name="to" value="light" aria-label="Light" { (ic("sun", "sm")) }
                            button class=[chosen(Theme::Dark)] type="submit" name="to" value="dark" aria-label="Dark" { (ic("moon", "sm")) }
                            button class=[chosen(Theme::System)] type="submit" name="to" value="system" { "Auto" }
                        }
                    }
                }
            }
        }
    }
}

/// The row at the top of every page: where you are, and the one thing
/// you can do from here.
fn headrow(
    viewer: Option<&Viewer>,
    repo: Option<&str>,
    section: Option<&str>,
    title: &str,
    head: &Head,
) -> Markup {
    html! {
        div class="head" {
            @match (repo, section) {
                (Some(repo), _) => {
                    div class="where" {
                        (ic("repo", ""))
                        @match repo.split_once('/') {
                            Some((owner, name)) => {
                                a href={ "/" (owner) } { (owner) }
                                span class="sep" { "/" }
                                b { (name) }
                            }
                            None => { b { (repo) } }
                        }
                    }
                }
                (None, Some(section)) => { h1 { (section_label(section)) } }
                (None, None) => { h1 { (title) } }
            }
            @if let Some(count) = &head.count { span class="n" { (count) } }
            div class="acts" {
                a class="quiet menu" href="#nav" { (ic("menu", "sm")) " Menu" }
                @if let Some(acts) = &head.acts { (acts) }
                @if viewer.is_some() && section != Some("new") && head.acts.is_none() {
                    a class="btn2 sm" href="/new" { (ic("plus", "sm")) "New repository" }
                }
            }
        }
    }
}

/// What a sidebar section is called on its own page.
fn section_label(section: &str) -> &str {
    match section {
        "inbox" => "Inbox",
        "you" => "Your changes",
        "tasks" => "Tasks",
        "agents" => "Agents",
        "settings" => "Settings",
        "people" => "People",
        "teams" => "Teams",
        "reports" => "Reports",
        "log" => "Forge log",
        "search" => "Search",
        "new" => "New repository",
        "task" => "Task",
        other => other,
    }
}

/// Inside a repository, its tabs head the page. The tabs are the
/// places a repository has: its code, its changes, its tasks, what
/// needs review, how much of it is verified, what happened.
fn repohead(repo: &str, active: Option<Tab>, viewer: Option<&Viewer>, chrome: &Chrome) -> Markup {
    let open = chrome
        .repos
        .iter()
        .find(|r| r.name == repo)
        .map(|r| r.open)
        .unwrap_or(0);
    html! {
        div class="repohead" {
            div class="tabs" {
                (tab(repo, "", "code", "Code", 0, active == Some(Tab::Code)))
                (tab(repo, "/changes", "changes", "Changes", open, active == Some(Tab::Changes)))
                @if viewer.is_some() {
                    (tab_to(&format!("/tasks?repo={repo}"), "tasks", "Tasks", 0, active == Some(Tab::Tasks)))
                }
                (tab(repo, "/community", "message", "Community", 0, active == Some(Tab::Community)))
                (tab(repo, "/review", "review", "Review", 0, active == Some(Tab::Review)))
                (tab(repo, "/coverage", "coverage", "Coverage", 0, active == Some(Tab::Coverage)))
                (tab(repo, "/activity", "activity", "Activity", 0, active == Some(Tab::Activity)))
                @if let Some(viewer) = viewer
                    && (viewer.1.admin || viewer.1.owned.iter().any(|r| r == repo)) {
                    (tab(repo, "/settings", "settings", "Settings", 0, active == Some(Tab::Settings)))
                }
            }
        }
    }
}

fn tab(repo: &str, path: &str, icon: &str, label: &str, count: usize, active: bool) -> Markup {
    tab_to(&format!("/{repo}{path}"), icon, label, count, active)
}

fn tab_to(href: &str, icon: &str, label: &str, count: usize, active: bool) -> Markup {
    html! {
        a class={ "tab" @if active { " on" } } href=(href) aria-current=[active.then_some("page")] {
            (ic(icon, ""))
            (label)
            @if count > 0 { span class="n" { (count) } }
        }
    }
}

fn short(oid: &str) -> &str {
    &oid[..oid.len().min(7)]
}

/// The public page: what this is, and a way to be told when it is
/// ready. Signed-out visitors get this instead of a sign-in form,
/// because a form asks for something they do not have and tells them
/// nothing about why they would want it.
/// Making an account: the form when this forge lets strangers in, and
/// otherwise the plain fact that it does not, with the way to ask.
pub fn signup(theme: Theme, state: super::Signup, error: Option<&str>) -> Markup {
    let open = state == super::Signup::Open;
    layout(
        theme,
        None,
        None,
        None,
        "Make an account",
        html! {
            div class="auth" {
                div class="card" {
                    div class="brand" { (mark()) "ambolt" }
                    @if let Some(error) = error { p class="error" { (error) } }
                    @if open {
                        form method="post" action="/signup" {
                            div class="field" {
                                label for="name" { "Username" }
                                input id="name" name="name" type="text" autocomplete="username"
                                    autocapitalize="none" autofocus required pattern="[a-z0-9-]{2,64}";
                                p class="hint" { "Lowercase letters, digits and hyphens. Your repositories live under it, and it cannot be changed later, so choose it with care." }
                            }
                            div class="field" {
                                label for="display" { "Shown as" }
                                input id="display" name="display" type="text" autocomplete="name";
                            }
                            div class="field" {
                                label for="email" { "Email" }
                                input id="email" name="email" type="email" autocomplete="email";
                                p class="hint" { "A confirmation link goes there; sign-in links and invitations use it." }
                            }
                            div class="field" {
                                label for="password" { "Password" }
                                input id="password" name="password" type="password"
                                    autocomplete="new-password" required;
                            }
                            button class="btn wide" type="submit" { "Make the account" }
                        }
                        p class="hint" { "Already here? " a href="/login" { "Sign in" } "." }
                    } @else if state == super::Signup::Full {
                        p class="plain" { "This forge is full: as many people have made accounts as it takes." }
                        p class="hint" { "Ask on the " a href="/" { "front page" } " and whoever runs it will make room or say where else to go; or sign in if you have an account: " a href="/login" { "Sign in" } "." }
                    } @else {
                        p class="plain" { "This forge takes people by invitation." }
                        p class="hint" { "Ask for one from the " a href="/" { "front page" } ", or sign in if you have been invited: " a href="/login" { "Sign in" } "." }
                    }
                }
            }
        },
    )
}

/// What the front page can say in numbers, all counted from this forge
/// so none of them is a claim. A number that cannot be computed is left
/// out rather than made up.
#[derive(Default)]
pub struct FrontNumbers {
    pub landed: Option<i64>,
    pub repos: Option<i64>,
    pub agents: Option<i64>,
}

/// One rule row inside the front page's drawn change page. The three
/// that settle in the story carry a class the stylesheet animates.
fn story_req(story: Option<&str>, met: bool, rule: &str, evidence: Option<&str>) -> Markup {
    html! {
        div class={ "req" @if met { " met" } @else { " unmet" } @if let Some(s) = story { " " (s) } } {
            span class="st" {
                @if met { (ic("check", "")) } @else { svg class="ic no" { r#use href="#i-x" {} } svg class="ic ok" { r#use href="#i-check" {} } }
            }
            div {
                b { (rule) }
                @if let Some(evidence) = evidence { div class="why" { (evidence) } }
            }
        }
    }
}

/// The front page: what this is, shown rather than described, and a way
/// to be told when it opens up. Everything animated has a resting state
/// the page shows without a script.
pub fn welcome(theme: Theme, joined: bool, error: Option<&str>, numbers: &FrontNumbers) -> Markup {
    layout(
        theme,
        None,
        None,
        None,
        "ambolt",
        html! {
            div class="landing" {
                header class="topnav" id="topnav" { div class="wrap" {
                    a class="brand" href="/" { (mark()) "ambolt" }
                    nav { a href="#how" { "How it works" } a href="#why" { "Why Ambolt" } a href="#git" { "Git" } a href="https://github.com/mandipadk/ambolt" { "Source" } }
                    div class="right" { a href="/login" { "Sign in" } a class="btn sm" href="#join" { "Join the waitlist" } }
                } }

                section class="hero" {
                    div class="wrap" {
                        span class="eyebrow" { i { (ic("check", "")) } "Alpha · self-hosted and free · hosted by invitation" }
                        h1 { "Git hosting for code that " em { "agents" } " write." }
                        p class="lede" { "Every change carries its proof: what was claimed, who re-ran it, who approved it, and why it was allowed to land." }
                        div class="cta" { a class="btn" href="#join" { "Join the waitlist" } a class="btn2" href="#git" { (ic("terminal", "")) "Run it yourself" } }
                        p class="fine" { "Ordinary git on the wire. One binary. AGPL-3.0." }
                    }
                    div class="frame" { div class="screen" { div class="ui story" {
                        aside class="sb" {
                            div class="org" { i { (mark()) } "Ambolt" }
                            a href="#" { (ic("home", "")) "Home" }
                            a href="#" { (ic("inbox", "")) "Inbox" span class="badge" { "5" } }
                            a href="#" { (ic("tasks", "")) "Tasks" }
                            a href="#" { (ic("agents", "")) "Agents" }
                            h5 { "Repositories" }
                            a class="on" href="#" { (ic("repo", "")) "ambolt / ambolt" }
                            a href="#" { (ic("repo", "")) "ambolt / console" }
                            h5 { "Working" }
                            div class="who" { (avatar("quill", "Quill", true, true)) "Quill · web/mod.rs" }
                            div class="who" { (avatar("scout", "Scout", true, true)) "Scout · web/" }
                        }
                        div class="main" {
                            div class="head" { span { "ambolt / ambolt" } span { "/" } span { "Changes" } span { "/" } b { "#2" } span class="btn land" { "Land on main" } }
                            div class="cols" {
                                div {
                                    h4 { "Bound the blame page by file size" }
                                    div class="meta" { span class="chip acc" { "Open" } span class="chip" { "2 attempts" } (avatar("quill", "Quill", true, false)) span { "Quill · opened 2 h ago · into main" } }
                                    div class="diff" {
                                        header { (ic("file", "sm")) code { "crates/server/src/web/diff.rs" } span class="pm" { span class="plus" { "+3" } " " span class="minus" { "−0" } } }
                                        div class="hunk" {
                                            div class="ln ctx" { span class="no" { "150" } span class="no" { "150" } span class="sign" {} code class="cd" { "    }" } }
                                            div class="ln ctx" { span class="no" { "151" } span class="no" { "151" } span class="sign" {} code class="cd" { "}" } }
                                            div class="ln add" { span class="no" {} span class="no" { "152" } span class="sign" { "+" } code class="cd" {} }
                                            div class="ln add" { span class="no" {} span class="no" { "153" } span class="sign" { "+" } code class="cd" { span class="hl-comment" { "/// Files past this many bytes are cut with a note at the top." } } }
                                            div class="ln add" { span class="no" {} span class="no" { "154" } span class="sign" { "+" } code class="cd" { span class="hl-keyword" { "pub const" } " BLAME_LIMIT: " span class="hl-keyword" { "usize" } " = 2 << 20;" } }
                                        }
                                        div class="thread" {
                                            div class="h" { (avatar("ada", "Ada", false, false)) b { "Ada" } span class="sec2" { "raised a concern on revision 1" } span class="when" { "2 h ago" } }
                                            p class="body" { "Where is the note rendered? The constant is here but nothing reads it yet." }
                                            div class="reply" { (avatar("scout", "Scout", true, false)) span { b { "Scout" } p { "Revision 2 renders it at the top of the file view." } } }
                                        }
                                    }
                                }
                                div {
                                    div class="panel readiness" {
                                        header class="ttl" { h2 class="title-not" { "Not ready to land" } h2 class="title-ready" { "Ready to land" } span class="n" { "7 rules" } }
                                        div class="pad" {
                                            div class="progress" { i class="on" {} i class="on" {} i class="on" {} i class="on" {} i class="bad p1" {} i class="bad p2" {} i class="bad p3" {} }
                                            div class="reqs" {
                                                (story_req(Some("s1"), false, "Someone other than the author approves it", Some("No approval yet")))
                                                (story_req(Some("s2"), false, "Tests pass on revision 2", Some("No claim on revision 2 yet")))
                                                (story_req(Some("s3"), false, "Every concern is resolved", Some("Ada's concern on line 152 is open")))
                                                (story_req(None, true, "Runner agrees with every claim", None))
                                                (story_req(None, true, "Nobody has blocked it", None))
                                                (story_req(None, true, "One attempt is chosen", None))
                                            }
                                            span class="btn wide land" { "Land on main" }
                                        }
                                    }
                                    div class="panel" {
                                        header { h2 { "Claims" } span class="n" { "revision 2" } }
                                        div class="ev-row" { (avatar("scout", "Scout", true, false)) div { div class="h" { b { "Tests pass" } span class="sec3" { "Scout" } } span class="cmd" { "cargo test -p ambolt-server" } div class="sub good" { (ic("rerun", "sm")) "Runner re-ran it · 61 passed" } } }
                                    }
                                }
                            }
                        }
                    } } }
                }

                section class="works" { div class="wrap" {
                    p { "Works with what you already use" }
                    div class="row" {
                        span { (ic("branch", "")) "Any git client" }
                        span { (ic("agents", "")) "Claude Code" }
                        span { (ic("agents", "")) "Cursor" }
                        span { (ic("globe", "")) "Anything that speaks MCP" }
                        span { (ic("rerun", "")) "Your CI as a runner" }
                    }
                } }

                section class="lsec" id="how" { div class="wrap" {
                    div class="top" { span class="kicker" { i {} "How it works" } h2 { "Push. Prove. Land." } p class="lede" { "Three steps, all recorded. Nothing lands on trust alone." } }
                    div class="steps" {
                        div class="step s1" {
                            div class="ico" { (ic("terminal", "")) }
                            h3 { "Push" }
                            p { "A normal git push opens a change." }
                            div class="mini" { span class="p" { "$" } " " span class="c" { "git push origin HEAD:refs/for/main" } br; span class="p" { "remote:" } " change #2 opened · revision 1" br; span class="p" { "remote:" } " not ready · 3 of 7 rules met" }
                        }
                        div class="step s2" {
                            div class="ico" { (ic("rerun", "")) }
                            h3 { "Prove" }
                            p { "Claims are commands. A runner re-runs them." }
                            div class="mini" { b { "claim" } " tests pass · Scout" br; span class="p" { "$" } " " span class="c" { "cargo test --workspace" } div class="sub" { span class="a" { i class="spin" {} "Runner re-running…" } span class="b" { (ic("check", "sm")) "Reproduced · 61 passed" } } }
                        }
                        div class="step s3" {
                            div class="ico" { (ic("receipt", "")) }
                            h3 { "Land" }
                            p { "The rules decide. The receipt says why." }
                            div class="mini sealed" { span class="seal" { (ic("check", "")) "Verified" } b { "#2" } " landed on main" br; "judged: revision 2 · Scout" br; "approved: Ada · Runner" br; span class="p" { "signed 9a59 2453 77e7 3fca" } }
                        }
                    }
                } }

                section class="lsec tight" id="why" { div class="wrap" {
                    div class="top" { span class="kicker" { i {} "Why Ambolt" } h2 { "Built for the moment most of your code isn't typed by a person." } }
                    div class="feats" {
                        div class="feat" {
                            div class="cap" { h3 { "Rules you can see" } p { "Every repository says what must be true before anything lands. Every change shows how far it is." } }
                            div class="stage" { div class="panel readiness" {
                                header { h2 { "Not ready to land" } span class="n" { "4 of 7" } }
                                div class="pad" {
                                    div class="progress" { i class="on" {} i class="on" {} i class="on" {} i class="on" {} i class="bad" {} i class="bad" {} i class="bad" {} }
                                    div class="reqs" {
                                        (story_req(None, false, "Someone other than the author approves it", Some("No approval yet")))
                                        (story_req(None, false, "Every concern is resolved", Some("Ada's concern on line 152 is open")))
                                        (story_req(None, true, "Runner agrees with every claim", None))
                                        (story_req(None, true, "Nobody has blocked it", None))
                                    }
                                }
                            } }
                        }
                        div class="feat flip" {
                            div class="stage" { div class="panel" {
                                div class="ev-row" { (avatar("quill", "Quill", true, false)) div { div class="h" { b { "Tests pass" } span class="chip bad" { (ic("alert", "")) "disputed" } } span class="cmd" { "cargo test -p ambolt-server" } div class="sub bad" { (ic("rerun", "sm")) "Runner saw 1 failure: blame::cuts_large_files" } } }
                                div class="ev-row" { (avatar("scout", "Scout", true, false)) div { div class="h" { b { "Tests pass" } span class="chip good" { (ic("check", "")) "reproduced" } } span class="cmd" { "cargo test -p ambolt-server" } div class="sub good" { (ic("check", "sm")) "Runner re-ran it · 61 passed" } } }
                            } }
                            div class="cap" { h3 { "Claims are commands, not comments" } p { "\"Tests pass\" carries the command that produced it. A runner re-runs it. A dispute blocks the change." } }
                        }
                        div class="feat" {
                            div class="cap" { h3 { "Attention, ranked" } p { "Your time goes where judgment is needed, and a share of unreviewed work is sampled anyway." } }
                            div class="stage" { div class="panel" {
                                div class="row need" { span class="chip bad" { "Disputed" } span class="tt" { span class="t" { "Bound the blame page by file size" } span class="s" { "Runner disagreed with Quill · two attempts" } } span class="avs" {} span class="age" { "2 h" } }
                                div class="row need" { span class="chip" { "Unreviewed" } span class="tt" { span class="t" { "Write the agent quickstart" } span class="s" { "Scout stopped and left a lesson" } } span class="avs" {} span class="age" { "2 h" } }
                                div class="row need" { span class="chip acc" { "Spot check" } span class="tt" { span class="t" { "Rename the client crate" } span class="s" { "picked · nobody has looked" } } span class="avs" {} span class="age" { "4 h" } }
                            } }
                        }
                        div class="feat flip" {
                            div class="stage" { div class="panel" {
                                div class="ev-row" { (avatar("scout", "Scout", true, false)) div { div class="h" { b { "Scout" } span class="sec3" { "claude-fable-5-1 · yours" } } div class="grants" { span class="chip g1" { "task" } span class="chip g2" { "push" } span class="chip g3" { "review" } span class="chip off" { "merge" } span class="chip off" { "verify" } } div class="sub" { "12 of 12 claims reproduced · 9 changes landed · 90 days" } } }
                                div class="ev-row" { (avatar("quill", "Quill", true, false)) div { div class="h" { b { "Quill" } span class="sec3" { "gpt-5 · Ada's" } } div class="grants" { span class="chip" { "task" } span class="chip" { "push" } span class="chip off" { "review" } } div class="sub" { span class="bad-t" { "7 of 8" } " claims reproduced · 5 landed" } } }
                            } }
                            div class="cap" { h3 { "Agents with permissions" } p { "Grant exactly what a job needs, everywhere or on one repository. Revoke in one click. Every claim goes on the record." } }
                        }
                        div class="feat" {
                            div class="cap" { h3 { "Signed receipts" } p { "Why it landed, attached to the commit, checkable offline, forever." } }
                            div class="stage" { div class="receipt" { span class="seal" { (ic("check", "")) "Verified" } b { "#1" } " landed on main as 6f1c9a2" br; "judged: revision 1 · Scout" br; "approved: Ada · correctness" br; "reproduced: Runner · 61 passed, 0 failed" br; "rules: 7 of 7 met" br; span class="sec3" { "signed by this forge · key 9a59 2453 77e7 3fca" } } }
                        }
                        div class="feat flip cov" {
                            div class="stage" { div { div class="big" { "34" small { "% verified" } } div class="cbars" { i class="g" {} i class="r" {} } div class="clegend" { span { i class="s-reproduced" {} "reproduced" } span { i class="s-gap" {} "declared gap" } span { i class="s-imported" {} "imported" } } } }
                            div class="cap" { h3 { "Coverage that climbs" } p { "Lines backed by a re-run claim, counted. Imported history is debt, paid down task by task." } }
                        }
                    }
                } }

                section class="lsec" id="git" { div class="wrap gitsec" {
                    div {
                        span class="kicker" { i {} "Ordinary git" }
                        h2 { "Nothing new to install for the people." }
                        div class="pts" {
                            div { i { "01" } span { b { "Clone and push" } " with the client you already have" } }
                            div { i { "02" } span { b { "One binary," } " one database file, one directory" } }
                            div { i { "03" } span { b { "Agents connect over MCP," } " built in" } }
                            div { i { "04" } span { b { "Private by default." } " Passkeys, tokens, scopes" } }
                        }
                    }
                    div class="bigterm" {
                        header { i {} i {} i {} }
                        // The whole transcript is on the page; the script
                        // replays it as typing, and without one it is read.
                        pre id="term" {
                            span class="p" { "$ " } span class="c" { "git push origin HEAD:refs/for/main\n" }
                            span class="a" { "remote: change #2 opened · revision 1\n" }
                            span class="a" { "remote: not ready · 3 of 7 rules met\n" }
                            span class="p" { "$ " } span class="c" { "ambolt claim --change 2 --test \"cargo test --workspace\"\n" }
                            span class="a" { "claim recorded · Runner will re-run it\n" }
                            span class="g" { "runner reproduced it · 61 passed · 4 of 7 rules met\n" }
                            span class="p" { "$ " } span class="c" { "ambolt land 2\n" }
                            span class="g" { "#2 landed on main · receipt signed 9a59…3fca\n" }
                        }
                    }
                } }

                section class="lsec tight" { div class="wrap" {
                    div class="nums" {
                        div class="numt" { div class="v" { "1" } div class="k" { "binary to run a whole forge" } }
                        @if let Some(landed) = numbers.landed.filter(|n| *n > 0) {
                            div class="numt" { div class="v" data-count=(landed) { span { (landed) } } div class="k" { "changes landed on this forge, each with a signed receipt" } }
                        }
                        @if let Some(repos) = numbers.repos.filter(|n| *n > 0) {
                            div class="numt" { div class="v" data-count=(repos) { span { (repos) } } div class="k" { @if repos == 1 { "repository hosted here, including this one" } @else { "repositories hosted here, including this one" } } }
                        }
                        @if let Some(agents) = numbers.agents.filter(|n| *n > 0) {
                            div class="numt" { div class="v" data-count=(agents) { span { (agents) } } div class="k" { @if agents == 1 { "agent on the record" } @else { "agents on the record" } } }
                        }
                        div class="numt" { div class="v" { "AGPL" } div class="k" { "free to run, change and fork" } }
                    }
                } }

                section class="lsec tight" id="join" { div class="wrap" {
                    div class="cta-block" {
                        h2 { "Put your code on the anvil." }
                        p { "Hosted forges open by invitation, in the order people asked." }
                        @if joined {
                            p class="joined" { (ic("check", "")) "You are on the list. We will be in touch." }
                        } @else {
                            form class="join" method="post" action="/waitlist" {
                                input name="email" type="email" required autocomplete="email" placeholder="you@example.com" aria-label="Email";
                                input name="company" type="text" autocomplete="organization" placeholder="for my company (optional)" aria-label="Company, if this is for one";
                                button class="btn" type="submit" { "Join the waitlist" }
                            }
                            @if let Some(error) = error { p class="error" { (error) } }
                            p class="alt" { "Name a company and we make it a forge of its own instead, with its people and organisations as owners." }
                            p class="alt" { "One address, kept so we can tell you when this opens up. Ask and it is deleted: it is deliberately not written to the log, because a log that cannot forget is the wrong place for a person's details." }
                        }
                        p class="alt" { "Or run it now: " code { "cargo install --git https://ambolt.sh/git/ambolt/ambolt ambolt" } }
                    }
                } }

                footer { div class="wrap" {
                    a class="brand" href="/" { (mark()) "ambolt" }
                    a href="https://github.com/mandipadk/ambolt" { "Source" }
                    a href="/report" { "Report a problem" }
                    a href="/login" { "Sign in" }
                    span class="right" { "© 2026 Ambolt · AGPL-3.0 forge · Apache-2.0 client" }
                } }
            }
        },
    )
}

/// A page for somebody who is not signed in: one card on a quiet page,
/// the mark above it, the title on it.
fn outside(theme: Theme, title: &str, body: Markup) -> Markup {
    layout(
        theme,
        None,
        None,
        None,
        title,
        html! {
            div class="auth" {
                div class="card" {
                    div class="brand" { (mark()) "ambolt" }
                    h1 { (title) }
                    (body)
                }
            }
        },
    )
}

pub fn forgot(theme: Theme, can_mail: bool, sent: bool, error: Option<&str>) -> Markup {
    outside(
        theme,
        "Reset your password",
        html! {
            @if let Some(error) = error { div class="notice bad" { (ic("alert", "")) span { (error) } } }
            @if sent {
                @if can_mail {
                    p { "If that account has an email address on record, a link is on its way. It works once, for thirty minutes." }
                    p class="hint" { "No address on record? The people who run this forge have been told, and can send you a new sign-in link." }
                } @else {
                    p { "The people who run this forge have been told, and can send you a new sign-in link." }
                }
                p class="hint" { a href="/login" { "Back to sign in" } }
            } @else {
                form class="form" method="post" action="/forgot" {
                    div class="field" {
                        label for="who" { "Username or email" }
                        input id="who" name="who" type="text" autocomplete="username" autofocus required;
                    }
                    button class="btn wide" type="submit" { @if can_mail { "Send a reset link" } @else { "Ask for a new link" } }
                    p class="hint" { a href="/login" { "Back to sign in" } }
                }
            }
        },
    )
}

pub fn reset(theme: Theme, token: &str, error: Option<&str>) -> Markup {
    outside(
        theme,
        "Choose a new password",
        html! {
            @if let Some(error) = error { div class="notice bad" { (ic("alert", "")) span { (error) } } }
            form class="form" method="post" action="/reset" {
                input type="hidden" name="token" value=(token);
                div class="field" {
                    label for="password" { "New password" }
                    input id="password" name="password" type="password" autocomplete="new-password" minlength="12" autofocus required;
                }
                div class="field" {
                    label for="confirm" { "Again" }
                    input id="confirm" name="confirm" type="password" autocomplete="new-password" minlength="12" required;
                    span class="hint" { "At least 12 characters." }
                }
                button class="btn wide" type="submit" { "Set password" }
            }
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub fn login(
    theme: Theme,
    dev: bool,
    can_mail: bool,
    can_passkey: bool,
    provider: Option<&str>,
    sent: bool,
    done: Option<&str>,
    error: Option<&str>,
) -> Markup {
    outside(
        theme,
        "Sign in",
        html! {
            @if let Some(error) = error { div class="notice bad" { (ic("alert", "")) span { (error) } } }
            @if let Some(done) = done { div class="notice" { (ic("check", "")) span { (done) } } }
            @if sent {
                div class="notice" { (ic("send", "")) span { "If that account has a confirmed address, a sign-in link is on its way. It works once, for fifteen minutes." } }
            }
            form class="form" method="post" action="/login" {
                div class="field" {
                    label for="principal" { "Username" }
                    input id="principal" name="principal" type="text"
                        autocomplete="username webauthn" autocapitalize="none" autofocus required;
                }
                div class="field" {
                    label for="password" { "Password" }
                    input id="password" name="password" type="password" autocomplete="current-password";
                }
                button class="btn wide" type="submit" { "Sign in" }
                p class="hint" { a href="/forgot" { "Forgot your password?" } }
            }
            @if provider.is_some() || can_passkey || can_mail {
                div class="or" { "or" }
            }
            @if let Some(provider) = provider {
                a class="btn2 wide" href="/login/oidc" { "Continue with " (provider) }
            }
            @if can_passkey {
                button class="btn2 wide" type="button" data-passkey="login" data-say="passkey-note" { (ic("key", "sm")) "Sign in with a passkey" }
                p class="hint" id="passkey-note" {}
            }
            @if can_mail && !sent {
                form method="post" action="/login/link" {
                    details class="alt" {
                        summary { "Email me a sign-in link" }
                        div class="field" {
                            label for="who" { "Username or email" }
                            input id="who" name="who" type="text" autocomplete="username" required;
                        }
                        button class="btn2 wide" type="submit" { "Send the link" }
                    }
                }
            }
            form method="post" action="/login" {
                details class="alt" {
                    summary { "Sign in with a token" }
                    div class="field" {
                        label for="token" { "API token" }
                        input id="token" name="token" type="password" autocomplete="off";
                        span class="hint" { "From your settings, once signed in." }
                    }
                    button class="btn2 wide" type="submit" { "Sign in with the token" }
                }
            }
            @if dev {
                p class="hint" { "Dev mode: a name alone is accepted as asserted identity." }
            }
        },
    )
}

pub fn home(theme: Theme, viewer: &Viewer, data: &super::HomeData) -> Markup {
    let people = &data.people;
    let working = &viewer.1.working;
    let disputed = data
        .needs_you
        .iter()
        .filter(|e| attention_chip(&e.item).1 == "Disputed")
        .count();
    let unlooked = data.needs_you.len() - disputed;
    layout(
        theme,
        Some(viewer),
        None,
        None,
        "Home",
        html! {
            div class="stats" {
                div class="stat" {
                    span class="k" { (ic("review", "sm")) "Needs you" }
                    span class="v" { (data.needs_you.len()) }
                    span class="d" {
                        @if data.needs_you.is_empty() { "nothing waits on your judgment" }
                        @else {
                            @if disputed > 0 { (disputed) " disputed" }
                            @if disputed > 0 && unlooked > 0 { ", " }
                            @if unlooked > 0 { (unlooked) " to look at" }
                        }
                    }
                }
                div class="stat" {
                    span class="k" { (ic("agents", "sm")) "At work now" }
                    span class="v" { (working.len()) small { @if working.len() == 1 { "agent" } @else { "agents" } } }
                    span class="d" {
                        @if working.is_empty() { "nobody is holding a path" }
                        @else { (names(working.iter().map(|w| w.display.as_str()))) }
                    }
                }
                div class="stat" {
                    span class="k" { (ic("check", "sm")) "Landed today" }
                    span class="v" { (data.landed_today) }
                    span class="d" {
                        @match &data.latest_landed {
                            Some((repo, number, title)) => { a href={ "/" (repo) "/changes/" (number) } { "#" (number) " " (title) } }
                            None => { "nothing landed in the last day" }
                        }
                    }
                }
                div class="stat" {
                    span class="k" { (ic("clock", "sm")) "In the queue" }
                    span class="v" { (data.lanes.iter().map(|l| l.queued).sum::<usize>()) }
                    span class="d" {
                        @if data.lanes.is_empty() { "nothing waiting to land" }
                        @else { (names(data.lanes.iter().map(|l| l.repo.as_str()))) }
                    }
                }
            }

            div class="sec" {
                div class="sh" { h2 { "Needs you" } span class="n" { (data.needs_you.len()) } }
                div class="panel" {
                    @if data.needs_you.is_empty() {
                        div class="empty" { b { "Nothing needs you right now." } "Changes that want a person's judgment appear here, ranked by what that judgment is worth." }
                    }
                    @for entry in &data.needs_you {
                        @let item = &entry.item;
                        @let (chip, label) = attention_chip(item);
                        @let (display, agent) = people.name(&item.change.owner);
                        a class="row need" href={ "/" (entry.repo) "/changes/" (item.change.number) } title=(attention_evidence(item)) {
                            span class=(chip) { (label) }
                            span class="tt" {
                                span class="t" { (item.change.title) }
                                span class="s tagline" {
                                    span class="tag" { (entry.repo) " #" (item.change.number) }
                                    @if let Some(draw) = &item.drawn { span class="tag" { "picked " (draw.day) } }
                                    @for signal in item.signals.iter().filter(|s| s.kind != ambolt_core::SignalKind::Drawn).take(2) {
                                        span class="tag" { (signal.description) }
                                    }
                                }
                            }
                            span class="avs" { (avatar(item.change.owner.as_str(), display, agent, false)) }
                            span class="age" title=(item.change.updated_at) { (ago(&item.change.updated_at)) }
                        }
                    }
                }
            }

            @if !data.mine.is_empty() {
                div class="sec" {
                    div class="sh" { h2 { "Yours, open" } span class="n" { (data.mine.len()) } }
                    div class="panel" {
                        @for (repo, change) in &data.mine {
                            a class="row need" href={ "/" (repo) "/changes/" (change.number) } {
                                span class="chip acc" { (ic("changes", "")) "Revision " (change.latest_revision) }
                                span class="tt" {
                                    span class="t" { (change.title) }
                                    span class="s" { (repo) " #" (change.number) " · into " (change.target) }
                                }
                                span class="avs" {}
                                span class="age" title=(change.updated_at) { (ago(&change.updated_at)) }
                            }
                        }
                    }
                }
            }

                div class="sec" {
                    div class="sh" { h2 { "Across your repositories" } }
                    div class="panel" {
                        @if data.recent.is_empty() { div class="empty" { "Nothing has happened across your repositories yet." } }
                        @for line in &data.recent {
                            @let (display, agent) = people.name(&line.actor);
                            div class="row feed" {
                                (avatar(line.actor.as_str(), display, agent, false))
                                span class="s wrap" {
                                    b { (display) } " " (line.what)
                                    @if let Some((href, label)) = &line.object { " " a href=(href) { b { (label) } } }
                                    @if !line.tail.is_empty() { " " (line.tail) }
                                }
                                span class="age" title=(line.ts) { (ago(&line.ts)) }
                            }
                        }
                    }
                }



            @if !data.lessons.is_empty() {
                div class="sec" {
                    div class="sh" { h2 { "Stopped, with a lesson" } span class="n" { (data.lessons.len()) } }
                    div class="panel" {
                        @for lesson in &data.lessons {
                            @let (display, agent) = people.name(&lesson.agent);
                            div class="row ls" {
                                (avatar(lesson.agent.as_str(), display, agent, false))
                                span class="tt" {
                                    span class="t" { (lesson.task_title) }
                                    span class="s wrap" { (lesson.outcome) }
                                }
                                span class="age" { (display) }
                            }
                        }
                    }
                }
            }
        },
    )
}

/// "Quill and Scout", "Quill, Scout and Ada": a short list, spoken.
fn names<'a>(items: impl Iterator<Item = &'a str>) -> String {
    let all: Vec<&str> = items.collect();
    match all.len() {
        0 => String::new(),
        1 => all[0].to_owned(),
        2 => format!("{} and {}", all[0], all[1]),
        n if n <= 4 => format!("{} and {}", all[..n - 1].join(", "), all[n - 1]),
        n => format!("{}, {} and {} more", all[0], all[1], n - 2),
    }
}

/// The chip a change wanting attention wears: what its lead signal is.
fn attention_chip(item: &ambolt_core::AttentionItem) -> (&'static str, &'static str) {
    use ambolt_core::SignalKind;
    let lead = item
        .signals
        .iter()
        .filter(|s| s.kind != SignalKind::Drawn)
        .max_by_key(|s| s.weight)
        .map(|s| s.kind);
    match lead {
        Some(SignalKind::DisputedClaim | SignalKind::RunnersDisagree) => ("chip bad", "Disputed"),
        Some(SignalKind::ReviewersDisagree) => ("chip bad", "Disagreement"),
        Some(SignalKind::Blocked) => ("chip bad", "Blocked"),
        Some(SignalKind::NoExecutedCheck) => ("chip", "No check run"),
        Some(SignalKind::SpotCheck) => ("chip acc", "Spot check"),
        _ if item.drawn.is_some() => ("chip acc", "Picked"),
        _ => ("chip", "Unreviewed"),
    }
}

/// A forge with nothing in it yet. The first thing anyone sees, so it
/// teaches the model rather than reporting an absence.
pub fn first_run(theme: Theme, viewer: &Viewer) -> Markup {
    layout(
        theme,
        Some(viewer),
        None,
        None,
        "Home",
        html! {
            div class="first" {
                h2 { "Nothing here yet" }
                p class="sec2" {
                    "ambolt records how software actually came to exist: who claimed what, \
                     who re-ran it, and why anything was allowed to land. It starts \
                     recording from the first push."
                }
                div class="panel" {
                    a class="row need" href="/new" {
                        span class="chip" { (ic("repo", "")) "Repository" }
                        span class="tt" { span class="t" { "Create a repository" } span class="s" { "Empty, ready for a first push." } }
                        span class="avs" {}
                        span class="age" { (ic("chev", "sm")) }
                    }
                    a class="row need" href="/new" {
                        span class="chip" { (ic("download", "")) "Import" }
                        span class="tt" { span class="t" { "Import from GitHub" } span class="s" { "Recorded as imported, never as reviewed." } }
                        span class="avs" {}
                        span class="age" { (ic("chev", "sm")) }
                    }
                    a class="row need" href="/agents" {
                        span class="chip" { (ic("agents", "")) "Agent" }
                        span class="tt" { span class="t" { "Add an agent" } span class="s" { "A token and a narrow capability grant." } }
                        span class="avs" {}
                        span class="age" { (ic("chev", "sm")) }
                    }
                }
            }
        },
    )
}

pub fn search(
    theme: Theme,
    viewer: &Viewer,
    query: &str,
    kind: Option<HitKind>,
    hits: &[super::Hit],
) -> Markup {
    // The filter row rewrites the query rather than adding a control:
    // what it does is visible in the box afterwards, and copyable.
    let without_kind: String = query
        .split_whitespace()
        .filter(|w| !w.to_lowercase().starts_with("kind:"))
        .collect::<Vec<_>>()
        .join(" ");
    let with = |k: Option<HitKind>| -> String {
        let q = match k {
            Some(k) => format!("{} kind:{}", without_kind, k.as_str()),
            None => without_kind.clone(),
        };
        format!("/search?q={}", super::urlencode(q.trim()))
    };
    let kind_words = |k: HitKind| match k {
        HitKind::Change => "Changes",
        HitKind::Repository => "Repositories",
        HitKind::Person => "People",
        HitKind::Task => "Tasks",
        HitKind::Lesson => "Lessons",
    };
    layout(
        theme,
        Some(viewer),
        None,
        None,
        "Search",
        html! {
            div class="sec top" {
                form class="search-in big" method="get" action="/search" {
                    (ic("search", ""))
                    input class="input" name="q" type="search" value=(query) autofocus
                          placeholder="Repositories, changes, people, or #12, repo:demo, by:scout" aria-label="Search";
                    button class="btn2" type="submit" { "Search" }
                }
                @if !query.trim().is_empty() {
                    div class="filters" {
                        a class=[kind.is_none().then_some("on")] href=(with(None)) { "All" }
                        @for k in [HitKind::Change, HitKind::Repository, HitKind::Person, HitKind::Task, HitKind::Lesson] {
                            a class=[(kind == Some(k)).then_some("on")] href=(with(Some(k))) { (kind_words(k)) }
                        }
                        span class="gap" {}
                        span class="sec3" { (hits.len()) @if hits.len() == 1 { " result" } @else { " results" } }
                    }
                }
                div class="panel" {
                    @if query.trim().is_empty() {
                        div class="empty" {
                            b { "Type to search repositories, changes, tasks, lessons and people." }
                            "Narrow with " code { "repo:" } ", " code { "state:open" } ", " code { "by:" } " or " code { "kind:" } "; "
                            code { "#12" } " opens a change by number."
                        }
                    } @else if hits.is_empty() {
                        div class="empty" { "Nothing matches " b { (query) } "." }
                    }
                    @for hit in hits {
                        a class="row need" href=(hit.href) {
                            span class="chip" { (hit.kind) }
                            span class="tt" {
                                span class="t" { (hit.label) }
                                span class="s" { (hit.detail) }
                            }
                            span class="avs" {}
                            span class="age" { (ic("chev", "sm")) }
                        }
                    }
                }
            }
        },
    )
}

/// `owners` is who the viewer may create under: themselves first, then
/// every organisation they belong to. With one choice there is nothing
/// to ask; the name simply lands under it.
pub fn new_repo(
    may_import: bool,
    theme: Theme,
    viewer: &Viewer,
    owners: &[String],
    chosen: Option<&str>,
    error: Option<&str>,
) -> Markup {
    let chosen = chosen.unwrap_or(viewer.0.as_str());
    layout(
        theme,
        Some(viewer),
        None,
        None,
        "New repository",
        html! {
            @if let Some(error) = error { div class="notice bad" { (ic("alert", "")) span { (error) } } }
            div class="sec top" {
                div class="page-form" {
                    p class="lede" { "A repository is empty until its first push. What lands in it is what its rules allow." }
                    form class="form" method="post" action="/new" {
                        @if owners.len() > 1 {
                            div class="field" {
                                label for="owner" { "Owner" }
                                select id="owner" name="owner" {
                                    @for owner in owners {
                                        option value=(owner) selected[owner == chosen] { (owner) }
                                    }
                                }
                                span class="hint" { "Yours, or an organisation you belong to. The address is the owner's name, then this one." }
                            }
                        } @else {
                            input type="hidden" name="owner" value=(chosen);
                        }
                        div class="field" {
                            label for="name" { "Name" }
                            input id="name" name="name" type="text" autofocus autocomplete="off" placeholder="lowercase, digits and hyphens" required;
                            span class="hint" { "It will live at /" (chosen) "/…" }
                        }
                        div class="field" {
                            label for="default_branch" { "Default branch" }
                            input id="default_branch" name="default_branch" type="text" autocomplete="off" placeholder="main";
                        }
                        // The forge fetching from an address a caller typed is
                        // the operator's to allow; the field is not shown to
                        // somebody it would refuse.
                        @if may_import {
                            div class="field" {
                                label for="source" { "Import from" }
                                input id="source" name="source" type="text" autocomplete="off" placeholder="https://github.com/you/project.git (optional)";
                                span class="hint" { "History brought in this way is recorded as imported. Nothing here was reviewed under this repository's policy, and the log says so rather than implying otherwise." }
                            }
                        }
                        div class="acts" { button class="btn" type="submit" { "Create" } }
                    }
                }
            }
        },
    )
}

pub fn you(theme: Theme, viewer: &Viewer, mine: &[(String, Change)]) -> Markup {
    layout_section_head(
        theme,
        viewer,
        "you",
        "Your changes",
        Head {
            count: Some(format!("{} open", mine.len())),
            acts: None,
        },
        html! {
            div class="sec top" {
                div class="panel" {
                    @if mine.is_empty() {
                        div class="empty" { b { "Nothing of yours is open." } "Push to a repository's " code { "refs/for/main" } " and the change appears here." }
                    }
                    @for (repo, change) in mine {
                        a class="row need" href={ "/" (repo) "/changes/" (change.number) } {
                            span class="chip acc" { (ic("changes", "")) "Revision " (change.latest_revision) }
                            span class="tt" {
                                span class="t" { (change.title) }
                                span class="s" { (repo) " #" (change.number) " · into " (change.target) }
                            }
                            span class="avs" {}
                            span class="age" title=(change.updated_at) { (ago(&change.updated_at)) }
                        }
                    }
                }
            }
        },
    )
}

/// What is addressed to the viewer, newest first, grouped by day. An
/// unread row carries a dot and full weight; a read one recedes. Every
/// row is a link to the thing itself, because a notice that cannot be
/// acted on from where it is read is a to-do list somebody has to copy.
pub fn inbox(
    theme: Theme,
    viewer: &Viewer,
    notices: &[Notice],
    unread: usize,
    people: &People,
) -> Markup {
    // Notices arrive newest first; the page groups them by day.
    let mut days: Vec<(String, Vec<&Notice>)> = Vec::new();
    for notice in notices {
        let day = day_of(&notice.ts);
        match days.last_mut() {
            Some((last, group)) if *last == day => group.push(notice),
            _ => days.push((day, vec![notice])),
        }
    }
    layout_section_head(
        theme,
        viewer,
        "inbox",
        "Inbox",
        Head {
            count: Some(format!("{unread} unread")),
            acts: (unread > 0).then(|| {
                html! {
                    form method="post" action="/inbox/read" {
                        input type="hidden" name="all" value="1";
                        button class="btn2 sm" type="submit" { (ic("check", "sm")) "Mark all read" }
                    }
                }
            }),
        },
        html! {
            div class="sec top" {
                @if notices.is_empty() {
                    div class="panel" { div class="empty" { b { "Nothing is waiting on you." } "Replies, reviews and landings of your changes arrive here." } }
                }
            }
            @for (day, group) in &days {
                div class="sec" {
                    div class="sh" { h2 { (day_label(day)) } }
                    div class="panel" {
                        @for notice in group {
                            @let (display, agent) = people.name(&notice.actor);
                            a class={ "row ib" @if notice.read { " read" } } href=(notice_href(notice)) {
                                i class={ "dot" @if !notice.read { " acc" } } {}
                                (avatar(notice.actor.as_str(), display, agent, false))
                                span class="tt" {
                                    span class="t" {
                                        @match notice.what.strip_prefix(notice.actor.as_str()).filter(|rest| rest.starts_with(' ')) {
                                            Some(rest) => { b { (display) } (rest) }
                                            None => { (notice.what) }
                                        }
                                    }
                                    span class="s" {
                                        @if let Some(repo) = &notice.repo { (repo) }
                                        @if let Some(number) = notice.number { " #" (number) }
                                    }
                                }
                                span class="chip" { (notice_kind_words(&notice.kind)) }
                                span class="age" title=(notice.ts) { (ago(&notice.ts)) }
                            }
                        }
                    }
                }
            }
        },
    )
}

/// Small facts with their names on them: `Model claude-…`, `Harness laptop`.
/// One line, wrapping, each pair a unit, so a reader is never guessing
/// what a bare word means.
fn kv(pairs: &[(&str, Markup)]) -> Markup {
    html! {
        span class="kv" {
            @for (k, v) in pairs {
                span class="pair" { span class="k" { (k) } span class="v" { (v) } }
            }
        }
    }
}

/// Prose with each `@name` a link to that person's page. The text is
/// escaped as ever; only the names become anchors.
fn with_mentions(text: &str) -> Markup {
    let names = ambolt_core::mentions::in_text(text);
    if names.is_empty() {
        return html! { (text) };
    }
    let mut out = Vec::new();
    let mut rest = text;
    for name in names {
        let at = format!("@{name}");
        let Some(i) = rest.find(&at) else { break };
        out.push(html! { (rest[..i]) a class="mention" href={ "/" (name) } { (at) } });
        rest = &rest[i + at.len()..];
    }
    out.push(html! { (rest) });
    html! { @for piece in out { (piece) } }
}

/// What kind of thing a notice is, as a chip.
fn notice_kind_words(kind: &str) -> &str {
    match kind {
        "mentioned" => "Mention",
        "opened" => "Change",
        "landed" => "Landed",
        "dequeued" => "Queue",
        "reply" | "concern" | "question" | "note" | "resolved" => "Discussion",
        "verdict" | "blocked" => "Review",
        "claim" | "disputed" => "Claim",
        "compared" => "Attempts",
        "drawn" => "Your look",
        "transfer" => "Transfer",
        "granted" => "Grant",
        "reset-request" => "People",
        other => other,
    }
}

/// Where a notice points: the change if it names one, else the
/// repository, else the viewer's own pages.
fn notice_href(notice: &Notice) -> String {
    match (&notice.repo, notice.number) {
        (Some(repo), Some(number)) => format!("/{repo}/changes/{number}"),
        (Some(repo), None) if notice.kind == "transfer" => format!("/{repo}/transfer"),
        (Some(repo), None) => format!("/{repo}"),
        (None, _) => match notice.kind.as_str() {
            "reset-request" => "/people".to_owned(),
            _ => "/you".to_owned(),
        },
    }
}

fn day_of(ts: &str) -> String {
    ts.get(..10).unwrap_or(ts).to_owned()
}

/// "5 Sep" for a day this year, "5 Sep 2025" otherwise: what a person
/// scanning a list wants to read.
fn short_day(ts: &str) -> String {
    let Some(date) = ts.get(..10) else {
        return ts.to_owned();
    };
    let mut parts = date.split('-');
    let (Some(year), Some(month), Some(day)) = (parts.next(), parts.next(), parts.next()) else {
        return date.to_owned();
    };
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let month = month
        .parse::<usize>()
        .ok()
        .and_then(|m| MONTHS.get(m.wrapping_sub(1)))
        .copied()
        .unwrap_or(month);
    let day = day.trim_start_matches('0');
    let this_year = jiff::Timestamp::now().to_string();
    if this_year.starts_with(year) {
        format!("{day} {month}")
    } else {
        format!("{day} {month} {year}")
    }
}

/// The body of a commit message: after the title line, before the
/// trailers nobody reads twice.
fn message_body(message: &str) -> Option<String> {
    let (_, rest) = message.split_once('\n')?;
    let lines: Vec<&str> = rest.trim().lines().collect();
    let mut end = lines.len();
    while end > 0 {
        let line = lines[end - 1].trim();
        let is_trailer = line.is_empty()
            || line.split_once(": ").is_some_and(|(key, _)| {
                !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            });
        if !is_trailer {
            break;
        }
        end -= 1;
    }
    let body = lines[..end].join("\n").trim().to_owned();
    (!body.is_empty()).then_some(body)
}

fn clock_of(ts: &str) -> String {
    ts.get(11..16).unwrap_or("").to_owned()
}

/// "Today", "Yesterday", or the date. The comparison is in UTC, which
/// is what the log records; a viewer's own midnight is not something
/// the server knows.
fn day_label(day: &str) -> String {
    let today = jiff::Timestamp::now().to_string();
    let today = today.get(..10).unwrap_or("");
    let yesterday = (jiff::Timestamp::now() - jiff::SignedDuration::from_hours(24)).to_string();
    let yesterday = yesterday.get(..10).unwrap_or("");
    if day == today {
        "Today".to_owned()
    } else if day == yesterday {
        "Yesterday".to_owned()
    } else {
        day.to_owned()
    }
}

/// Every task the viewer may see, newest first, filtered by state and,
/// when asked, by repository; the lessons attempts left behind sit
/// under the list, because they are read before the next attempt.
pub fn tasks(
    theme: Theme,
    viewer: &Viewer,
    tasks: &[Task],
    filter: Option<TaskState>,
    repo: Option<&str>,
    lessons: &[ambolt_core::Lesson],
    people: &People,
) -> Markup {
    let href = |state: Option<TaskState>, repo: Option<&str>| {
        let mut parts = Vec::new();
        if let Some(state) = state {
            parts.push(format!("state={}", state.as_str()));
        }
        if let Some(repo) = repo {
            parts.push(format!("repo={repo}"));
        }
        if parts.is_empty() {
            "/tasks".to_owned()
        } else {
            format!("/tasks?{}", parts.join("&"))
        }
    };
    layout_section_head(
        theme,
        viewer,
        "tasks",
        "Tasks",
        Head {
            count: Some(tasks.len().to_string()),
            acts: None,
        },
        html! {
            div class="sec top" {
                div class="filters" {
                    a class=[filter.is_none().then_some("on")] href=(href(None, repo)) { "All" }
                    @for state in [TaskState::Open, TaskState::Claimed, TaskState::Landed, TaskState::Abandoned] {
                        a class=[(filter == Some(state)).then_some("on")] href=(href(Some(state), repo)) { (task_state_words(state)) }
                    }
                    @if !viewer.1.repos.is_empty() {
                        span class="gap" {}
                        a class=[repo.is_none().then_some("on")] href=(href(filter, None)) { (ic("repo", "sm")) "Every repository" }
                        @for known in &viewer.1.repos {
                            a class=[(repo == Some(known.name.as_str())).then_some("on")] href=(href(filter, Some(&known.name))) { (known.name) }
                        }
                    }
                }
                div class="panel" {
                    @if tasks.is_empty() {
                        div class="empty" { b { "No tasks yet." } "An agent or a person opens one: a durable statement of what should be done and why." }
                    }
                    @for task in tasks {
                        a class="row tk" href={ "/tasks/" (task.id.as_str()) } {
                            (task_chip(task.state))
                            span class="tt" {
                                span class="t" { (task.title) }
                                (kv(&[
                                    ("Repository", html! { @match &task.repo { Some(repo) => { (repo) } None => { span class="none" { "any" } } } }),
                                    if task.attempts > 1 {
                                        ("Attempts", html! { (task.claimants.len()) " of " (task.attempts) " taken" })
                                    } else {
                                        ("Held by", html! { @match &task.claimed_by { Some(who) => { (people.name(who).0) } None => { span class="none" { "nobody yet" } } } })
                                    },
                                ]))
                            }
                            span class="avs" {
                                @for who in task.claimants.iter().chain(task.claimed_by.iter()).take(3) {
                                    @let (display, agent) = people.name(who);
                                    (avatar(who.as_str(), display, agent, false))
                                }
                            }
                        }
                    }
                }
            }
            @if !lessons.is_empty() {
                div class="sec" {
                    div class="sh" {
                        h2 { "Stopped, with a lesson" }
                        span class="n" { (lessons.len()) }
                        @if let Some(repo) = repo {
                            div class="right" { a href={ "/" (repo) "/lessons" } { "Search the lessons" } }
                        }
                    }
                    div class="panel" {
                        @for lesson in lessons {
                            @let (display, agent) = people.name(&lesson.agent);
                            div class="row ls" {
                                (avatar(lesson.agent.as_str(), display, agent, false))
                                span class="tt" {
                                    span class="t" { (lesson.task_title) }
                                    span class="s wrap" { (lesson.outcome) }
                                }
                                span class="age" { (display) }
                            }
                        }
                        div class="foot" { (ic("sparkle", "sm")) "When an agent stops, it says why. Those reasons are searched before the next attempt." }
                    }
                }
            }
        },
    )
}

fn domain_words(domain: ReviewDomain) -> &'static str {
    match domain {
        ReviewDomain::Correctness => "Correctness",
        ReviewDomain::Security => "Security",
        ReviewDomain::Design => "Design",
        ReviewDomain::Style => "Style",
    }
}

fn task_state_words(state: TaskState) -> &'static str {
    match state {
        TaskState::Open => "Open",
        TaskState::Claimed => "Taken",
        TaskState::Landed => "Landed",
        TaskState::Abandoned => "Abandoned",
    }
}

fn task_chip(state: TaskState) -> Markup {
    html! {
        @match state {
            TaskState::Open => { span class="chip" { (ic("tasks", "")) "Open" } }
            TaskState::Claimed => { span class="chip acc" { (ic("agents", "")) "Taken" } }
            TaskState::Landed => { span class="chip good" { (ic("check", "")) "Landed" } }
            TaskState::Abandoned => { span class="chip" { (ic("x", "")) "Abandoned" } }
        }
    }
}

/// One task: its intent, who holds it, the runs against it and what they
/// learned, and the changes that came of it.
/// The task's change and everything the attempts put on it.
pub struct TaskFocus {
    pub change: Change,
    pub revisions: Vec<Revision>,
    pub claims: Vec<Claim>,
    pub verifications: Vec<Verification>,
    /// Readiness of the change while it is open.
    pub trace: Option<PolicyTrace>,
    pub preference: Option<ambolt_core::Preference>,
}

pub struct TaskPage<'a> {
    pub theme: Theme,
    pub viewer: &'a Viewer,
    pub task: &'a Task,
    pub sessions: &'a [Session],
    pub changes: &'a [Change],
    pub focus: Option<&'a TaskFocus>,
    pub can_close: bool,
    pub people: &'a People,
}

/// One attempt at a task, as the page reads it: an author, their
/// sessions, and their revisions of the task's change.
struct Attempt<'a> {
    by: &'a PrincipalId,
    sessions: Vec<&'a Session>,
    revisions: Vec<&'a Revision>,
}

fn attempts<'a>(sessions: &'a [Session], revisions: &'a [Revision]) -> Vec<Attempt<'a>> {
    let mut out: Vec<Attempt<'a>> = Vec::new();
    for revision in revisions.iter().filter(|r| !r.by.as_str().is_empty()) {
        match out.iter_mut().find(|a| a.by == &revision.by) {
            Some(attempt) => attempt.revisions.push(revision),
            None => out.push(Attempt {
                by: &revision.by,
                sessions: sessions.iter().filter(|s| s.agent == revision.by).collect(),
                revisions: vec![revision],
            }),
        }
    }
    out
}

pub fn task(page: TaskPage<'_>) -> Markup {
    let TaskPage {
        theme,
        viewer,
        task,
        sessions,
        changes,
        focus,
        can_close,
        people,
    } = page;
    let attempts: Vec<Attempt<'_>> = focus
        .map(|f| attempts(sessions, &f.revisions))
        .unwrap_or_default();
    let competing = focus.is_some_and(|f| f.change.competing);
    let closable = can_close && matches!(task.state, TaskState::Open | TaskState::Claimed);
    layout_section(
        theme,
        viewer,
        "tasks",
        &task.title,
        html! {
            div class="pagehead" {
                div {
                    h1 { (task.title) }
                    div class="meta" {
                        (task_chip(task.state))
                        span class="by" {
                            @let (display, agent) = people.name(&task.created_by);
                            (avatar(task.created_by.as_str(), display, agent, false))
                            b { (display) }
                        }
                        @if task.attempts > 1 {
                            span { "claimed by " (task.claimants.len()) " of " (task.attempts) }
                        } @else if let Some(who) = &task.claimed_by {
                            span { "held by " (people.name(who).0) }
                        }
                        @if let Some(repo) = &task.repo { span { "in " a href={ "/" (repo) } { (repo) } } }
                        @if let Some(f) = focus {
                            span { "change " a href={ "/" (f.change.repo) "/changes/" (f.change.number) } { "#" (f.change.number) } ", " (f.change.state.as_str()) }
                        }
                        @if let Some(parent) = &task.parent { span { a href={ "/tasks/" (parent.as_str()) } { "part of a larger task" } } }
                    }
                }
                @if closable {
                    div class="acts" {
                        form method="post" action={ "/tasks/" (task.id.as_str()) } {
                            button class="btn2 sm" type="submit" name="state" value="landed" { (ic("check", "sm")) "Mark landed" }
                            button class="btn2 sm danger" type="submit" name="state" value="abandoned" { (ic("x", "sm")) "Abandon" }
                        }
                    }
                }
            }
            div class="chg-body" {
                pre class="msg" { (task.spec) }

                @if let Some(f) = focus {
                    @if !attempts.is_empty() && (competing || task.attempts > 1) {
                        div class="sec" {
                            div class="sh" { h2 { "Attempts" } span class="n" { (attempts.len()) " · revisions of #" (f.change.number) } }
                            div class="tries" {
                                @for attempt in &attempts {
                                    @let latest = attempt.revisions.last().expect("an attempt has a revision");
                                    @let chosen = f.change.preferred_revision == Some(latest.number);
                                    @let (display, agent) = people.name(attempt.by);
                                    div class={ "try" @if chosen { " chosen" } } {
                                        div class="h" {
                                            (avatar(attempt.by.as_str(), display, agent, false))
                                            b { (display) }
                                            @if chosen { span class="chip acc" { (ic("check", "")) "chosen" } }
                                            @for session in &attempt.sessions {
                                                span class="chip" { "session · " (session.state.as_str()) }
                                            }
                                        }
                                        div class="tagline" {
                                            @for revision in &attempt.revisions {
                                                span class="tag" {
                                                    "rev " (revision.number) " " code { (short(&revision.commit_oid)) }
                                                    @if !revision.paths.is_empty() { ", " (revision.paths.len()) " files" }
                                                }
                                            }
                                        }
                                        @let claims: Vec<&Claim> = f.claims.iter().filter(|c| c.revision == latest.number).collect();
                                        @if claims.is_empty() { span class="s" { "No claims on r" (latest.number) " yet." } }
                                        @for claim in claims { (claim_row(claim, &f.verifications, people)) }
                                        @for session in attempt.sessions.iter().filter(|s| s.outcome.is_some()) {
                                            q class="s" { (session.outcome.as_deref().unwrap_or("")) }
                                        }
                                    }
                                }
                            }
                            @if let Some(preference) = &f.preference {
                                @if f.change.preferred_revision.is_some() {
                                    @let (display, agent) = people.name(&preference.by);
                                    div class="panel" {
                                        div class="ev-row" {
                                            (avatar(preference.by.as_str(), display, agent, false))
                                            div {
                                                div class="h" {
                                                    b { (display) }
                                                    span class="sec2" { "preferred revision " (preference.revision) " over " @for (i, n) in preference.over.iter().enumerate() { @if i > 0 { ", " } "revision " (n) } }
                                                    span class="sec3" { (short_day(&preference.at)) }
                                                }
                                                q { (preference.rationale) }
                                            }
                                        }
                                    }
                                }
                            }
                            @if competing && f.change.preferred_revision.is_none() && f.change.state == ChangeState::Open {
                                form class="review" method="post" action={ "/" (f.change.repo) "/changes/" (f.change.number) "/prefer" } {
                                    span class="lab" { "Compare" }
                                    select class="input sm" name="revision" aria-label="Revision" {
                                        @for attempt in &attempts {
                                            @let latest = attempt.revisions.last().expect("an attempt has a revision");
                                            option value=(latest.number) { "Revision " (latest.number) " by " (attempt.by) }
                                        }
                                    }
                                    input class="input sm" type="text" name="rationale" placeholder="Why this one and not the others" aria-label="Why this one and not the others" required;
                                    button class="btn2 sm" type="submit" { "Prefer" }
                                }
                            }
                        }
                    }
                    @if let Some(trace) = &f.trace {
                        div class="sec" {
                            div class="sh" { h2 { "Before #" (f.change.number) " lands" } span class="n" { "revision " (f.change.judged_revision()) } }
                            div class="panel" {
                                div class="pad reqs" {
                                    @for requirement in trace.requirements.iter().filter(|r| !r.satisfied) { (requirement_row(requirement)) }
                                    @for requirement in trace.requirements.iter().filter(|r| r.satisfied) { (requirement_row(requirement)) }
                                }
                            }
                        }
                    }
                }

                div class="sec" {
                    div class="sh" { h2 { "Sessions" } span class="n" { (sessions.len()) } }
                    div class="panel" {
                        @if sessions.is_empty() { div class="empty" { "Nobody has run against this yet." } }
                        @for session in sessions {
                            @let (display, agent) = people.name(&session.agent);
                            div class="row ls" {
                                (avatar(session.agent.as_str(), display, agent, session.state == SessionState::Active))
                                span class="tt" {
                                    span class="t" { (display) " " span class="sec3" { (session.state.as_str()) } }
                                    @if let Some(outcome) = &session.outcome { span class="s wrap" { (outcome) } }
                                }
                                span class="age" title=(session.id.as_str()) { (session.state.as_str()) }
                            }
                        }
                    }
                }
                div class="sec" {
                    div class="sh" { h2 { "Changes" } span class="n" { (changes.len()) } }
                    div class="panel" {
                        @if changes.is_empty() { div class="empty" { "No change names this task yet." } }
                        @for change in changes {
                            @let (display, agent) = people.name(&change.owner);
                            a class="row need" href={ "/" (change.repo) "/changes/" (change.number) } {
                                @match change.state {
                                    ChangeState::Open => { span class="chip acc" { (ic("changes", "")) "Open" } }
                                    ChangeState::Merged => { span class="chip good" { (ic("check", "")) "Landed" } }
                                    ChangeState::Abandoned => { span class="chip" { (ic("x", "")) @if change.discarded { "Discarded" } @else { "Abandoned" } } }
                                }
                                @if change.proposal { span class="chip" { "Proposal" } }
                                span class="tt" {
                                    span class="t" { (change.title) }
                                    span class="s" { (change.repo) " #" (change.number) }
                                }
                                span class="avs" { (avatar(change.owner.as_str(), display, agent, false)) }
                                span class="age" title=(change.updated_at) { (ago(&change.updated_at)) }
                            }
                        }
                    }
                }
            }
        },
    )
}

/// Everything that happened, forge-wide, for whoever runs it.
pub fn forge_log(
    theme: Theme,
    viewer: &Viewer,
    numbers: &HashMap<String, (i64, String)>,
    events: &[Envelope],
    scopes: &HashMap<i64, Option<String>>,
    after: i64,
    people: &People,
) -> Markup {
    let refs: Refs = numbers
        .iter()
        .map(|(id, (number, title))| (id.as_str(), (*number, title.as_str())))
        .collect();
    layout_section_head(
        theme,
        viewer,
        "log",
        "Forge log",
        Head {
            count: Some("everything, newest last".to_owned()),
            acts: None,
        },
        html! {
            div class="sec top" {
                @if events.is_empty() { div class="panel" { div class="empty" { "Nothing more recent." } } }
            }
            (event_days(&refs, events, people, Some(scopes)))
            @if let Some(last) = events.last() {
                @if after > 0 || events.len() >= 200 {
                    div class="sec" { a class="btn2 sm" href={ "/log?after=" (last.seq.0) } { "Later events" } }
                }
            }
        },
    )
}

/// A repository's settings: one panel per thing an owner decides, a
/// sub-navigation beside them, the landing policy with its preview and
/// its simulation as panels of their own.
#[allow(clippy::too_many_arguments)] // one page, one set of facts about the repository
pub fn repo_settings(
    theme: Theme,
    viewer: &Viewer,
    repo: &Repo,
    // Every live grant on this repository, and whether the viewer may
    // change them: the owner, or an organisation's owners.
    access: &[ambolt_core::Grant],
    may_grant: bool,
    error: Option<&str>,
    done: bool,
    preview: Option<&[(Change, PolicyTrace)]>,
    simulation: Option<&ambolt_core::Simulation>,
) -> Markup {
    let policy = &repo.policy;
    let ninety_days_ago = (jiff::Timestamp::now() - jiff::SignedDuration::from_hours(24 * 90))
        .strftime("%Y-%m-%d")
        .to_string();
    let base = format!("/{}/settings", repo.name);
    layout(
        theme,
        Some(viewer),
        Some(&repo.name),
        Some(Tab::Settings),
        "Settings",
        html! {
            @if let Some(error) = error { div class="notice bad" { (ic("alert", "")) span { (error) } } }
            @if done { div class="notice" { (ic("check", "")) span { "Saved." } } }
            div class="settings" {
                nav class="subnav" {
                    a href="#visibility" { "Visibility" }
                    a href="#ownership" { "Ownership" }
                    a href="#access" { "Access" }
                    a href="#policy" { "Landing policy" }
                    a class="sub" href="#approval" { "Approval" }
                    a class="sub" href="#looks" { "Human looks" }
                    a class="sub" href="#trust" { "Earned trust" }
                    a class="sub" href="#packs" { "Packs" }
                    @if viewer.1.admin { a href="#mirror" { "Mirror" } }
                    a href="#description" { "Description" }
                    a href="#name" { "Name" }
                    a href="#archive" { "Archive" }
                    a href="#delete" { "Delete" }
                }
                div class="panels" {
                    div class="panel" id="visibility" {
                        form class="pref" method="post" action={ (base) "/visibility" } {
                            h3 { "Visibility" }
                            div class="seg radio" {
                                label { input type="radio" name="visibility" value="private" checked[repo.visibility == Visibility::Private]; span { (ic("lock", "sm")) "Private" } }
                                label { input type="radio" name="visibility" value="public" checked[repo.visibility == Visibility::Public]; span { (ic("globe", "sm")) "Public" } }
                            }
                            p class="what" { "Private: only you, and whoever you grant something on it. Public: anyone can read and clone it; writing still needs authority." }
                            div class="acts" { button class="btn2" type="submit" { "Save" } }
                        }
                    }
                    div class="panel" id="ownership" {
                        div class="pref" {
                            h3 { "Ownership" span class="n" { (repo.owner.as_str()) } }
                            @if let Some(pending) = &repo.pending_owner {
                                p class="what" { "Offered to " b { (pending.as_str()) } ". Nothing changes until they accept." }
                                form method="post" action={ (base) "/transfer" } {
                                    input type="hidden" name="action" value="withdraw";
                                    button class="btn2 danger" type="submit" { "Withdraw the offer" }
                                }
                            } @else {
                                p class="what" { "Owning a repository carries every capability on it. Offer it to a person; it moves when they accept." }
                                form class="line" method="post" action={ (base) "/transfer" } {
                                    input type="hidden" name="action" value="offer";
                                    input class="input" id="to" name="to" type="text" autocomplete="off" placeholder="Their username" required aria-label="Offer to";
                                    button class="btn2" type="submit" { "Offer ownership" }
                                }
                            }
                        }
                    }
                    div class="panel" id="access" {
                        div class="pref" {
                            h3 { "Access" span class="n" { (access.len()) } }
                            p class="what" { "Who holds what on this repository, beyond its owner. A grant to a person or an agent, or to a team inside the organisation, named " code { "org/team" } "." }
                            @if access.is_empty() { p class="s" { "Nobody holds anything here yet." } }
                            @for grant in access {
                                div class="line" {
                                    b { (grant.grantee.as_str()) }
                                    @for action in &grant.actions { span class="chip" { (action.as_str()) } }
                                    span class="s" { "from " (grant.grantor.as_str()) @if let Some(until) = &grant.until { " until " (until) } }
                                    @if may_grant {
                                        form method="post" action={ (base) "/access" } {
                                            input type="hidden" name="action" value="revoke";
                                            input type="hidden" name="grant" value=(grant.id.as_str());
                                            button class="ghost sm danger" type="submit" { "Revoke" }
                                        }
                                    }
                                }
                            }
                            @if may_grant {
                                form class="line" method="post" action={ (base) "/access" } {
                                    input type="hidden" name="action" value="grant";
                                    input class="input" name="grantee" type="text" autocomplete="off" placeholder="Person, agent, or org/team" required aria-label="Grantee";
                                    label class="chk" { input type="checkbox" name="task" value="1"; "task" }
                                    label class="chk" { input type="checkbox" name="push" value="1"; "push" }
                                    label class="chk" { input type="checkbox" name="review" value="1"; "review" }
                                    label class="chk" { input type="checkbox" name="merge" value="1"; "merge" }
                                    label class="chk" { input type="checkbox" name="verify" value="1"; "verify" }
                                    button class="btn2" type="submit" { "Grant" }
                                }
                            }
                        }
                    }
                    form class="policy" method="post" action={ (base) "/policy" } {
                        div class="panel" id="policy" {
                            div class="pref" {
                                h3 { "Landing policy" }
                                p class="what" { "What must be true of a change before it lands on " b { (repo.default_branch) } ". Every rule shows on the change page, met or not, so nobody has to guess." }
                                h4 { "Checks" }
                                div class="opts" {
                                    label class="opt" {
                                        span class="t" { "Tests must pass" }
                                        span class="d" { "Someone ran the tests on the landing revision and recorded the command, so a runner can run them again." }
                                        input class="sw" type="checkbox" name="require_executed_check" checked[policy.require_executed_check];
                                    }
                                    label class="opt" {
                                        span class="t" { "A runner must reproduce a claim" }
                                        span class="d" { "A runner re-runs the claim and gets the same answer before the change lands." }
                                        input class="sw" type="checkbox" name="require_runner_verification" checked[policy.require_runner_verification];
                                        span class="sub" {
                                            label for="runner_quorum" { "Runners of distinct provenance that must agree" }
                                            input class="input sm num" id="runner_quorum" name="runner_quorum" type="number" min="1" max="9" value=(policy.runner_quorum.to_string());
                                        }
                                    }
                                    label class="opt" {
                                        span class="t" { "Every concern must be resolved" }
                                        span class="d" { "A concern raised in discussion holds the change until whoever raised it, or a later revision, resolves it." }
                                        input class="sw" type="checkbox" name="require_concerns_resolved" checked[policy.require_concerns_resolved];
                                    }
                                    label class="opt" {
                                        span class="t" { "Agents act only inside sessions" }
                                        span class="d" { "An agent's standing token cannot push, review or land on its own. It opens a session first, so everything it does has one on record." }
                                        input class="sw" type="checkbox" name="agents_act_in_sessions" checked[policy.agents_act_in_sessions];
                                    }
                                    label class="opt" {
                                        span class="t" { "Anyone may propose a change" }
                                        span class="d" { "While the repository is public, anyone signed in can push a change here as a proposal: theirs to revise and abandon, yours to review and land. It counts against their allowance, not yours." }
                                        input class="sw" type="checkbox" name="proposals" checked[policy.proposals];
                                    }
                                    label class="opt" {
                                        span class="t" { "Anyone may report a bug or ask for something" }
                                        span class="d" { "While the repository is public, anyone signed in can file a bug, a request or a question here. A bug carrying the command that shows it gets re-run; one without waits for somebody to add one." }
                                        input class="sw" type="checkbox" name="community" checked[policy.community];
                                    }
                                }
                            }
                        }
                        div class="panel" id="approval" {
                            div class="pref" {
                                h3 { "Approval" }
                                div class="field" {
                                    label for="independence" { "Who must approve" }
                                    select id="independence" name="independence" {
                                        @for choice in [Independence::HumanOrTwoModels, Independence::HumanOnly, Independence::Anyone, Independence::None] {
                                            option value=(choice.as_str()) selected[policy.independence == choice] {
                                                @match choice {
                                                    Independence::HumanOrTwoModels => "Someone other than the author: one person, or two agents of different models",
                                                    Independence::HumanOnly => "A person, and only a person",
                                                    Independence::Anyone => "Anyone but the owner",
                                                    Independence::None => "Nobody (a scratch repository)",
                                                }
                                            }
                                        }
                                    }
                                    span class="hint" { "A change waits for this approval however many checks pass." }
                                }
                                div class="field" {
                                    label { "Reviewed for" }
                                    div class="checks" {
                                        @for domain in [ReviewDomain::Correctness, ReviewDomain::Security, ReviewDomain::Design, ReviewDomain::Style] {
                                            label class="check" { input type="checkbox" name="domains" value=(domain.as_str()) checked[policy.required_domains.contains(&domain)]; span { (domain_words(domain)) } }
                                        }
                                    }
                                    span class="hint" { "An approval counts only when it says it looked at each of these. Leave them all off to accept any approval." }
                                }
                            }
                        }
                        div class="panel" id="looks" {
                            div class="pref" {
                                h3 { "Human looks" }
                                p class="what" { "Even when every rule is met, some changes deserve a person's eyes. The rules pick the ones whose judgment is worth the most, and a picked change waits for a person before it lands." }
                                div class="field" {
                                    label for="attention_budget" { "Changes picked per day" }
                                    input class="input sm num" id="attention_budget" name="attention_budget" type="number" min="0" max="100"
                                          value=(policy.attention_budget.map(|n| n.to_string()).unwrap_or_default());
                                    span class="hint" { "Empty for none." }
                                }
                            }
                        }
                        div class="panel" id="trust" {
                            div class="pref" {
                                @let trust = policy.trust.as_ref();
                                h3 { "Earned trust" }
                                p class="what" { "An owner whose claims a runner has kept reproducing may stand in for some of the rules above. What their record may stand in for:" }
                                div class="opts" {
                                    label class="opt" {
                                        span class="t" { "Their claim, in place of a runner" }
                                        span class="d" { "The owner's own test claim counts as reproduced." }
                                        input class="sw" type="checkbox" name="trust_waives" value="runner_verification" checked[trust.is_some_and(|t| t.waives.contains(&Waiver::RunnerVerification))];
                                    }
                                    label class="opt" {
                                        span class="t" { "Their claim, in place of an approval" }
                                        span class="d" { "The owner's own claim counts as the independent approval." }
                                        input class="sw" type="checkbox" name="trust_waives" value="independent_approval" checked[trust.is_some_and(|t| t.waives.contains(&Waiver::IndependentApproval))];
                                    }
                                }
                                h4 { "When their record counts" }
                                div class="sentence" {
                                    span { "At least" }
                                    input class="input sm num" id="trust_percent" name="trust_percent" type="number" min="50" max="100" aria-label="Percent reproduced" value=(trust.map(|t| t.min_reproduced_percent.to_string()).unwrap_or_else(|| "98".to_owned()));
                                    span { "% of at least" }
                                    input class="input sm num" id="trust_claims" name="trust_claims" type="number" min="1" max="10000" aria-label="Claims" value=(trust.map(|t| t.min_claims.to_string()).unwrap_or_else(|| "20".to_owned()));
                                    span { "claims reproduced in the last" }
                                    input class="input sm num" id="trust_days" name="trust_days" type="number" min="1" max="3650" aria-label="Days" value=(trust.map(|t| t.window_days.to_string()).unwrap_or_else(|| "90".to_owned()));
                                    span { "days" }
                                }
                                div class="field" {
                                    label for="trust_paths" { "Only for changes touching" }
                                    input class="input" id="trust_paths" name="trust_paths" type="text" autocomplete="off" placeholder="docs/, *.md" value=(trust.map(|t| t.paths.join(", ")).unwrap_or_default());
                                    span class="hint" { "Paths and patterns, comma-separated. Empty means any change." }
                                }
                            }
                        }
                        div class="panel" id="packs" {
                            div class="pref" {
                                h3 { "Policy packs" }
                                p class="what" { "A pack is a whole policy at once. Choosing one replaces everything above when you save." }
                                div class="field" {
                                    label for="pack" { "Apply a pack" }
                                    select id="pack" name="pack" {
                                        option value="" { "None, keep the rules above" }
                                        @for pack in ambolt_core::packs() { option value=(pack.name) { (pack.name) " · " (pack.description) } }
                                    }
                                }
                                div class="field" {
                                    label for="pack_json" { "Or paste one exported by another repository" }
                                    textarea id="pack_json" name="pack_json" rows="3" placeholder="{ \"pack\": 1, \"name\": … }" {}
                                    span class="hint" { a href={ "/api/repos/" (repo.name) "/policy/pack" } { "Export this policy as a pack" } }
                                }
                            }
                        }
                        div class="panel" id="save" {
                            div class="pref" {
                                h3 { "Save, or try it first" }
                                p class="what" { "A preview shows which open changes the rules would hold; a simulation replays past landings against them. Neither changes anything." }
                                div class="savebar" {
                                    button class="btn" type="submit" name="action" value="save" { "Save policy" }
                                    button class="btn2" type="submit" name="action" value="preview" { "Preview against open changes" }
                                }
                                div class="line" {
                                    span class="lbl" { "Simulate against landings since" }
                                    input class="input sm" id="since" name="since" type="date" value=(ninety_days_ago) aria-label="Since";
                                    button class="btn2" type="submit" name="action" value="simulate" { "Simulate" }
                                }
                            }
                        }
                    }
                    @if let Some(simulation) = simulation {
                        div class="panel" {
                            header { (ic("rerun", "")) h2 { "Simulated" } span class="n" { "landings since " (short_day(&simulation.since)) } }
                            div class="pad" {
                                p class="what" {
                                    "Against " (simulation.landings) " landing(s) since " (short_day(&simulation.since)) ", this policy would have held " (simulation.held) "."
                                    @if simulation.landings > 0 && simulation.held == 0 { " A requirement nothing failed is one that was being met, which is the reason to keep it." }
                                }
                            }
                            @for (description, n) in &simulation.by_requirement {
                                div class="row" { span class="s" { (n) " held by: " (description) } }
                            }
                            @for entry in simulation.entries.iter().filter(|e| e.held) {
                                a class="row need" href={ "/" (repo.name) "/changes/" (entry.number) } {
                                    span class="chip bad" { "held" }
                                    span class="tt" { span class="t" { "#" (entry.number) " " (entry.title) } span class="s" { (entry.unmet.join("; ")) } }
                                    span class="avs" {} span class="age" {}
                                }
                            }
                        }
                    }
                    @if let Some(previewed) = preview {
                        @let blocked: Vec<&(Change, PolicyTrace)> = previewed.iter().filter(|(_, trace)| !trace.satisfied).collect();
                        div class="panel" {
                            header { (ic("review", "")) h2 { "Previewed" } span class="n" { "against " (previewed.len()) " open change(s)" } }
                            div class="pad" { p class="what" { "This policy would hold " (blocked.len()) " of them." } }
                            @for (change, trace) in blocked {
                                a class="row need" href={ "/" (repo.name) "/changes/" (change.number) } {
                                    span class="chip bad" { "held" }
                                    span class="tt" { span class="t" { "#" (change.number) " " (change.title) } span class="s" { (trace.unmet_summary()) } }
                                    span class="avs" {} span class="age" {}
                                }
                            }
                        }
                    }
                    @if viewer.1.admin {
                        div class="panel" id="mirror" {
                            form class="pref" method="post" action={ (base) "/mirror" } {
                                h3 { "Mirror" span class="n" { @if let Some(m) = &repo.mirror { (m.url) } @else { "none" } } }
                                p class="what" { "Landed branches are copied to this address after every landing. The credential lives with whoever runs the forge, never here." }
                                div class="field" {
                                    label for="mirror-url" { "Push URL" }
                                    input id="mirror-url" name="url" type="text" autocomplete="off" placeholder="https://… (empty to stop mirroring)" value=(repo.mirror.as_ref().map(|m| m.url.as_str()).unwrap_or(""));
                                }
                                label class="check" { input type="checkbox" name="enabled" checked[repo.mirror.as_ref().is_some_and(|m| m.enabled)]; span { "Pushes attempted" } }
                                div class="acts" { button class="btn2" type="submit" { "Save mirror" } }
                            }
                        }
                    }
                    div class="panel" id="description" {
                        form class="pref" method="post" action={ (base) "/description" } {
                            h3 { "Description" }
                            div class="field" {
                                label for="description" { "What this repository is for" }
                                input id="description" name="description" type="text" autocomplete="off" maxlength="300" value=(repo.description);
                            }
                            div class="acts" { button class="btn2" type="submit" { "Save" } }
                        }
                    }
                    div class="panel" id="topics" {
                        form class="pref" method="post" action={ (base) "/topics" } {
                            h3 { "Topics" }
                            div class="field" {
                                label for="topics" { "A few words it is filed under" }
                                input id="topics" name="topics" type="text" autocomplete="off" maxlength="300" value=(repo.topics.join(" ")) placeholder="rust forge agents";
                                p class="hint" { "Lowercase, hyphens, eight at most. Searchable, and how Explore groups public repositories." }
                            }
                            div class="acts" { button class="btn2" type="submit" { "Save" } }
                        }
                    }
                    div class="panel" id="name" {
                        form class="pref" method="post" action={ (base) "/rename" } {
                            h3 { "Name" span class="n" { (repo.name) } }
                            p class="what" { "Everything follows the new name; the old one answers not found." }
                            div class="field" {
                                label for="rename-to" { "New name" }
                                input id="rename-to" name="to" type="text" autocomplete="off" autocapitalize="none" placeholder="lowercase, digits, hyphens" required;
                            }
                            div class="acts" { button class="btn2" type="submit" { "Rename" } }
                        }
                    }
                    div class="panel" id="archive" {
                        form class="pref" method="post" action={ (base) "/archive" } {
                            h3 { "Archive" span class="n" { @if repo.archived { "archived" } @else { "active" } } }
                            @if repo.archived {
                                p class="what" { "Read-only: nothing new lands here until it is unarchived." }
                                input type="hidden" name="archived" value="no";
                                div class="acts" { button class="btn2" type="submit" { "Unarchive" } }
                            } @else {
                                p class="what" { "An archived repository stays readable and clonable; pushes, new changes and new tasks are refused." }
                                input type="hidden" name="archived" value="yes";
                                div class="acts" { button class="btn2" type="submit" { "Archive" } }
                            }
                        }
                    }
                    div class="panel" id="delete" {
                        form class="pref" method="post" action={ (base) "/delete" } {
                            h3 class="bad-t" { "Delete" }
                            p class="what" { "Its changes, claims, verdicts and discussion go with it; the log keeps what happened. Tasks and lessons stay. Type its name to confirm." }
                            div class="field" {
                                label for="confirm" { "Repository name" }
                                input id="confirm" name="confirm" type="text" autocomplete="off" autocapitalize="none" placeholder=(repo.name) required;
                            }
                            div class="acts" { button class="btn2 danger" type="submit" { "Delete this repository" } }
                        }
                    }
                }
            }
        },
    )
}

/// What the person an offer was made to sees: the offer, and a yes or no.
pub fn transfer_offer(theme: Theme, viewer: &Viewer, repo: &Repo, error: Option<&str>) -> Markup {
    layout(
        theme,
        Some(viewer),
        None,
        None,
        "Ownership offered",
        html! {
            div class="sec top" {
                div class="panel narrow" {
                    header { (ic("repo", "")) h2 { "Ownership offered" } span class="n" { (repo.name) } }
                    div class="pad" {
                        @if let Some(error) = error { div class="notice bad" { (ic("alert", "")) span { (error) } } }
                        p class="what" {
                            b { (repo.owner.as_str()) } " has offered you " b { (repo.name) } ". "
                            "If you accept, you hold every capability on it from then on, and they hold none unless you grant it."
                        }
                        form class="acts" method="post" action={ "/" (repo.name) "/transfer" } {
                            button class="btn" type="submit" name="action" value="accept" { "Accept" }
                            button class="btn2" type="submit" name="action" value="decline" { "Decline" }
                        }
                    }
                }
            }
        },
    )
}

/// What the settings page has to say about the last thing that happened.
#[derive(Default)]
pub struct SettingsNote<'a> {
    pub error: Option<&'a str>,
    pub done: bool,
    pub sent: bool,
    pub first: bool,
}

/// A browser, roughly, from a user agent string. Enough to tell your
/// laptop from your phone; nothing here is trusted for anything else.
fn browser_family(agent: Option<&str>) -> &'static str {
    let Some(agent) = agent else {
        return "unknown browser";
    };
    let a = agent.to_ascii_lowercase();
    let device = if a.contains("iphone") || a.contains("ipad") {
        " on iOS"
    } else if a.contains("android") {
        " on Android"
    } else if a.contains("macintosh") || a.contains("mac os") {
        " on macOS"
    } else if a.contains("windows") {
        " on Windows"
    } else if a.contains("linux") {
        " on Linux"
    } else {
        ""
    };
    match (
        a.contains("edg/"),
        a.contains("chrome/") || a.contains("crios/"),
        a.contains("firefox/") || a.contains("fxios/"),
        a.contains("safari/"),
        a.starts_with("curl/"),
    ) {
        (true, ..) => match device {
            " on macOS" => "Edge on macOS",
            " on Windows" => "Edge on Windows",
            _ => "Edge",
        },
        (_, true, ..) => match device {
            " on macOS" => "Chrome on macOS",
            " on Windows" => "Chrome on Windows",
            " on Linux" => "Chrome on Linux",
            " on Android" => "Chrome on Android",
            " on iOS" => "Chrome on iOS",
            _ => "Chrome",
        },
        (_, _, true, ..) => match device {
            " on macOS" => "Firefox on macOS",
            " on Windows" => "Firefox on Windows",
            " on Linux" => "Firefox on Linux",
            _ => "Firefox",
        },
        (_, _, _, true, _) => match device {
            " on iOS" => "Safari on iOS",
            _ => "Safari on macOS",
        },
        (_, _, _, _, true) => "curl",
        _ => "another browser",
    }
}

/// Everything about the account, one panel per section behind a
/// sub-navigation: address, how you sign in, tokens, sessions, how the
/// pages look. Tokens and sessions used to be pages of their own.
pub struct SettingsPage<'a> {
    pub theme: Theme,
    pub viewer: &'a Viewer,
    pub contact: &'a Contact,
    pub can_mail: bool,
    pub passkeys: Option<&'a [PasskeyRecord]>,
    pub identities: Option<(&'a str, &'a [ambolt_core::IdentityLink])>,
    pub tokens: &'a [ambolt_core::TokenInfo],
    pub sessions: &'a [BrowserSession],
    /// A token just minted, shown this once.
    pub fresh: Option<&'a str>,
    pub note: SettingsNote<'a>,
}

pub fn settings(page: SettingsPage<'_>) -> Markup {
    let SettingsPage {
        theme,
        viewer,
        contact,
        can_mail,
        passkeys,
        identities,
        tokens,
        sessions,
        fresh,
        note,
    } = page;
    let SettingsNote {
        error,
        done,
        sent,
        first,
    } = note;
    let now = jiff::Timestamp::now().to_string();
    let live = tokens
        .iter()
        .filter(|t| !t.revoked && t.until.as_deref().is_none_or(|u| u > now.as_str()))
        .count();
    let others = sessions.iter().filter(|s| !s.current).count();
    layout_section(
        theme,
        viewer,
        "settings",
        "Settings",
        html! {
            @if let Some(error) = error { div class="notice bad" { (ic("alert", "")) span { (error) } } }
            @if done { div class="notice" { (ic("check", "")) span { "Saved." } } }
            @if sent { div class="notice" { (ic("send", "")) span { "A confirmation link is on its way." } } }
            @if first {
                div class="notice" { (ic("key", "")) span { "You are signed in from an invitation, which worked once. Set a password, or add a passkey, to sign in next time." } }
            }
            @if let Some(secret) = fresh {
                div class="once" {
                    p { b { "Copy this now." } " It is stored only as a hash, so this is the one time it can be shown." }
                    code class="secret" { (secret) }
                }
            }
            div class="settings" {
                nav class="subnav" {
                    a href="#account" { "Account" }
                    a href="#email" { "Email" }
                    a href="#security" { "Security" }
                    a href="#tokens" { "Tokens" }
                    a href="#sessions" { "Sessions" }
                    a href="#appearance" { "Appearance" }
                }
                div class="panels" {
                    div class="panel" id="account" {
                        div class="pref" {
                            h3 { "Account" }
                            p class="what" { "Your username is how the forge names you everywhere: pages, git, receipts, the log. It is one of a kind on this forge and cannot be changed." }
                            div class="keyrow" {
                                span class="k" { "Username" }
                                code { (viewer.0.as_str()) }
                            }
                            div class="keyrow" {
                                span class="k" { "Display name" }
                                span { (viewer.1.display) }
                            }
                        }
                    }
                    div class="panel" id="email" {
                        div class="pref" {
                            h3 { "Email" }
                            p class="what" {
                                @match (&contact.email, &contact.pending) {
                                    (Some(email), None) => { (email) ", confirmed." }
                                    (Some(email), Some(pending)) => { (email) ", confirmed. " (pending) " is awaiting confirmation." }
                                    (None, Some(pending)) => { (pending) " is awaiting confirmation; follow the link we sent." }
                                    (None, None) => { "No address on record. One is needed for password resets and sign-in links." }
                                }
                            }
                            @if can_mail {
                                form class="line" method="post" action="/you/settings/email" {
                                    input class="input" name="email" type="email" autocomplete="email" required
                                          placeholder=(if contact.email.is_some() { "new address" } else { "you@example.org" })
                                          aria-label="Email address";
                                    button class="btn2" type="submit" { "Send a confirmation" }
                                }
                                span class="hint" { "Kept beside your credentials, not in the log; shown to nobody; trusted only once you have followed the link." }
                            } @else {
                                span class="hint" { "This forge does not send mail, so an address cannot be confirmed here." }
                            }
                        }
                    }
                    div class="panel" id="security" {
                        div class="pref" {
                            h3 { "Security" }
                            @if let Some(passkeys) = passkeys {
                                div class="field" {
                                    label { "Passkeys" }
                                    @if passkeys.is_empty() { span class="what" { "None yet. A passkey signs you in with the device in your hand, no password needed." } }
                                    @for key in passkeys {
                                        div class="keyrow" {
                                            span { (ic("key", "sm")) " " (key.label) }
                                            span class="sec3" {
                                                @match &key.last_used {
                                                    Some(used) => { "last used " (day_of(used)) }
                                                    None => { "added " (day_of(&key.created)) }
                                                }
                                            }
                                            form method="post" action="/you/passkeys/remove" {
                                                input type="hidden" name="cred_id" value=(key.cred_id);
                                                button class="ghost sm danger" type="submit" { "Remove" }
                                            }
                                        }
                                    }
                                    div class="line" {
                                        input class="input sm" id="passkey-label" type="text" placeholder="A name for this device" autocomplete="off" aria-label="Passkey name";
                                        button class="btn2 sm" type="button" data-passkey="register" data-say="passkey-note" { "Add a passkey" }
                                    }
                                    span class="hint" id="passkey-note" { "Your device asks you to confirm; nothing leaves it but a public key." }
                                }
                            }
                            @if let Some((provider, links)) = identities {
                                div class="field" {
                                    label { "Sign in with " (provider) }
                                    @if links.is_empty() { span class="what" { "Not linked. Link your " (provider) " account and it signs you in here; nothing links itself." } }
                                    @for link in links {
                                        div class="keyrow" {
                                            span { (link.email.as_deref().unwrap_or(&link.subject)) }
                                            span class="sec3" { "linked " (day_of(&link.linked_at)) }
                                            form method="post" action="/you/settings/oidc/unlink" {
                                                input type="hidden" name="subject" value=(link.subject);
                                                button class="ghost sm danger" type="submit" { "Unlink" }
                                            }
                                        }
                                    }
                                    @if links.is_empty() {
                                        div class="line" { a class="btn2 sm" href="/you/settings/oidc/link" { "Link " (provider) } }
                                    }
                                }
                            }
                            form class="form" method="post" action="/you/settings" {
                                div class="field" {
                                    label for="password" { "New password" }
                                    input id="password" name="password" type="password" autocomplete="new-password" minlength="12" required;
                                }
                                div class="field" {
                                    label for="confirm" { "Again" }
                                    input id="confirm" name="confirm" type="password" autocomplete="new-password" minlength="12" required;
                                    span class="hint" { "At least 12 characters. Changing it signs you out everywhere, including here." }
                                }
                                div class="acts" { button class="btn2" type="submit" { "Change password" } }
                            }
                        }
                    }
                    div class="panel" id="tokens" {
                        div class="pref" {
                            h3 { "Tokens" span class="n" { (live) " live" } }
                            p class="what" { "A token is a password for git and the API. Give one to each machine and revoke it here when the machine goes." }
                            @if tokens.is_empty() { span class="what" { "None yet." } }
                            @for token in tokens {
                                div class="keyrow" {
                                    span {
                                        (token.label.as_deref().unwrap_or("unlabelled"))
                                        " " code class="sec3" { (token.id.0) }
                                    }
                                    span class="sec3" {
                                        @if token.revoked { "revoked" }
                                        @else {
                                            @match &token.until {
                                                Some(until) if until.as_str() <= now.as_str() => { "expired" }
                                                Some(until) => { "until " (day_of(until)) }
                                                None => { "until revoked" }
                                            }
                                        }
                                    }
                                    @if !token.revoked {
                                        form method="post" action="/you/tokens" {
                                            input type="hidden" name="action" value="revoke";
                                            input type="hidden" name="token" value=(token.id.0);
                                            button class="ghost sm danger" type="submit" { "Revoke" }
                                        }
                                    } @else { span {} }
                                }
                            }
                            form class="line" method="post" action="/you/tokens" {
                                input type="hidden" name="action" value="mint";
                                input class="input" name="label" type="text" placeholder="What it is for" autocomplete="off" aria-label="Label";
                                select class="input" name="days" aria-label="Expires" {
                                    option value="30" { "Expires in 30 days" }
                                    option value="90" selected { "Expires in 90 days" }
                                    option value="365" { "Expires in a year" }
                                    option value="never" { "Never expires" }
                                }
                                button class="btn2" type="submit" { "Mint a token" }
                            }
                        }
                    }
                    div class="panel" id="sessions" {
                        div class="pref" {
                            h3 { "Where you are signed in" span class="n" { (sessions.len()) } }
                            @for session in sessions {
                                div class="keyrow" {
                                    span {
                                        (browser_family(session.agent.as_deref()))
                                        span class="sec3" { " · signed in " (day_of(&session.created)) }
                                        @if session.current { span class="sec3" { " · this session" } }
                                    }
                                    span class="sec3" {
                                        @match &session.last_seen {
                                            Some(seen) => { "seen " (day_of(seen)) " " (clock_of(seen)) }
                                            None => { "" }
                                        }
                                    }
                                    @if session.current {
                                        span class="chip good" { "current" }
                                    } @else {
                                        form method="post" action="/you/sessions" {
                                            input type="hidden" name="id" value=(session.id);
                                            button class="ghost sm danger" type="submit" { "Sign out" }
                                        }
                                    }
                                }
                            }
                            @if others > 0 {
                                form class="line" method="post" action="/you/sessions" {
                                    input type="hidden" name="others" value="1";
                                    button class="btn2 sm" type="submit" { "Sign out everywhere else" }
                                }
                            }
                            span class="hint" { "Changing your password ends every session, this one included. Ending one here ends only that one." }
                        }
                    }
                    div class="panel" id="appearance" {
                        div class="pref" {
                            h3 { "Appearance" }
                            div class="field" {
                                label { "Theme" }
                                form class="seg" method="post" action="/theme" {
                                    input type="hidden" name="back" value="/you/settings#appearance";
                                    button class=[(theme == Theme::Light).then_some("on")] type="submit" name="to" value="light" { (ic("sun", "sm")) "Light" }
                                    button class=[(theme == Theme::Dark).then_some("on")] type="submit" name="to" value="dark" { (ic("moon", "sm")) "Dark" }
                                    button class=[(theme == Theme::System).then_some("on")] type="submit" name="to" value="system" { "Match system" }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

/// Teams: authority held in one place and carried by whoever is on it.
pub fn teams(
    theme: Theme,
    viewer: &Viewer,
    teams: &[super::TeamRow],
    repos: &[String],
    error: Option<&str>,
    people: &People,
) -> Markup {
    layout_section_head(
        theme,
        viewer,
        "teams",
        "Teams",
        Head {
            count: Some(teams.len().to_string()),
            acts: Some(html! { a class="btn sm" href="#add" { (ic("plus", "sm")) "Add a team" } }),
        },
        html! {
            @if let Some(error) = error { div class="notice bad" { (ic("alert", "")) span { (error) } } }
            div class="sec top" {
                p class="lede" { "A team holds authority; whoever is on it carries that authority, and loses it on leaving. An organisation is a team that owns repositories." }
                @if teams.is_empty() { div class="panel" { div class="empty" { b { "None yet." } "Make one, add people, and grant it what its members should all hold." } } }
                div class="grid2" {
                    @for row in teams {
                        @let id = row.principal.id.as_str();
                        div class="panel agent" {
                            header {
                                (avatar(id, &row.principal.display, false, false))
                                div class="tt" {
                                    h2 { (row.principal.display) }
                                    span class="s" { code { (id) } }
                                }
                            }
                            div class="pad ag" {
                                div {
                                    span class="lbl" { "Members" }
                                    @if row.members.is_empty() { div class="s" { "Nobody yet." } }
                                    div class="line" {
                                        @for member in &row.members {
                                            @let (display, agent) = people.name(member);
                                            span class="chip" {
                                                (avatar(member.as_str(), display, agent, false))
                                                a href={ "/" (member.as_str()) } { (display) }
                                                @if row.owners.contains(member) { span class="s" { "owner" } }
                                                form method="post" action="/teams" {
                                                    input type="hidden" name="action" value="remove";
                                                    input type="hidden" name="team" value=(id);
                                                    input type="hidden" name="member" value=(member.as_str());
                                                    button class="x" type="submit" aria-label={ "Remove " (member.as_str()) } title={ "Remove " (member.as_str()) } { (ic("x", "sm")) }
                                                }
                                            }
                                        }
                                    }
                                    form class="line" method="post" action="/teams" {
                                        input type="hidden" name="action" value="add";
                                        input type="hidden" name="team" value=(id);
                                        input class="input sm" name="member" type="text" placeholder="Add a person or agent" autocomplete="off" aria-label="Member";
                                        button class="btn2 sm" type="submit" { "Add" }
                                    }
                                }
                                div {
                                    span class="lbl" { "Holds" }
                                    @if row.grants.is_empty() { div class="s" { "Nothing yet." } }
                                    @for grant in &row.grants {
                                        div class="line" {
                                            @for action in &grant.actions { span class="chip" { (action.as_str()) } }
                                            span class="s" { "on " (grant.repo.as_deref().unwrap_or("everything")) }
                                        }
                                    }
                                    @if viewer.1.admin {
                                        details class="fold" {
                                            summary class="ghost sm" { (ic("plus", "sm")) "Grant something" }
                                            form class="form" method="post" action="/teams" {
                                                input type="hidden" name="action" value="grant";
                                                input type="hidden" name="team" value=(id);
                                                div class="line" {
                                                    @for capability in ["task", "push", "review", "merge", "verify", "admin"] {
                                                        label class="check" { input type="checkbox" name=(capability) value="on"; span { (capability) } }
                                                    }
                                                }
                                                div class="line" {
                                                    select class="input sm" name="repo" aria-label="Where" {
                                                        option value="" { "Everywhere" }
                                                        @for repo in repos { option value=(repo) { (repo) } }
                                                    }
                                                    button class="btn2 sm" type="submit" { "Grant" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div class="panel" id="add" {
                        header { (ic("plus", "")) h2 { "Add a team" } }
                        form class="pad form" method="post" action="/teams" {
                            input type="hidden" name="action" value="create";
                            div class="field" {
                                label for="id" { "Name" }
                                input id="id" name="id" type="text" autocomplete="off" placeholder="lowercase, digits and hyphens" required;
                            }
                            div class="field" {
                                label for="display" { "Display name" }
                                input id="display" name="display" type="text" autocomplete="off";
                            }
                            div class="field" {
                                label for="owner" { "Owner" }
                                input id="owner" name="owner" type="text" autocomplete="off" placeholder="who runs it; put on it and named its owner";
                            }
                            div class="acts" { button class="btn" type="submit" { "Add a team" } }
                        }
                    }
                }
            }
        },
    )
}

/// One line of "what you are using, out of what you may".
fn allowance(what: &str, used: String, limit: Option<String>) -> Markup {
    html! {
        div class="row allow" {
            span class="t" { (what) }
            span class="s" { (used) }
            span class="age" {
                @match limit {
                    Some(limit) => { "of " (limit) }
                    None => { "no limit" }
                }
            }
        }
    }
}

/// An owner's page: who they are, and every repository of theirs the
/// reader may see. A person or an organisation; for an organisation,
/// its members too.
#[allow(clippy::too_many_arguments)] // one page, one set of facts about its owner
pub fn owner(
    theme: Theme,
    who: Reading<'_>,
    owner: &ambolt_core::Principal,
    repos: &[ambolt_core::Repo],
    members: &[(ambolt_core::PrincipalId, ambolt_core::TeamRole)],
    // Members invited who have not arrived yet.
    invited: &[ambolt_core::PrincipalId],
    // What being on the organisation means on its repositories; none
    // for a person.
    members_act: Option<ambolt_core::MembersAct>,
    // The teams inside the organisation, with how many are on each.
    teams: &[(String, usize)],
    // What the viewer is to this organisation, if anything.
    viewer_role: Option<ambolt_core::TeamRole>,
    may_create: bool,
    // Whether the viewer may change who is on the organisation: one of
    // its owners, or whoever runs the forge.
    may_manage: bool,
    // What this owner is taking up and what they may, shown only to
    // them and to whoever runs the forge: how full somebody's account
    // is is their business.
    allowances: Option<(ambolt_core::Usage, ambolt_core::Quota)>,
    // The record: what the log says this person or agent did lately.
    // None for an organisation.
    record: Option<&ambolt_core::Record>,
    error: Option<&str>,
    // An owner just asked for more; say it was heard.
    asked: bool,
    // An invitation link the forge could not mail, shown this once; or
    // where it was mailed.
    fresh: Option<&str>,
    mailed: Option<&str>,
    people: &People,
) -> Markup {
    let organisation = owner.kind == ambolt_core::PrincipalKind::Team;
    let agent = owner.kind == ambolt_core::PrincipalKind::Agent;
    layout_reading(
        theme,
        who,
        None,
        None,
        owner.id.as_str(),
        html! {
            div class="pagehead" {
                div class="who" {
                    span class="av lg" { (avatar_of(owner.id.as_str(), &owner.display, owner.kind, false)) }
                    div {
                        h1 { (owner.display) }
                        div class="meta" {
                            code { (owner.id.as_str()) }
                            @if organisation { span class="chip" { (ic("agents", "")) "Organisation" } }
                            @else if agent { span class="chip" { (ic("agents", "")) "Agent" } }
                            @else { span class="chip" { (ic("user", "")) "Person" } }
                            @if !owner.active { span class="chip bad" { "Deactivated" } }
                        }
                    }
                }
                div class="acts" {
                    @if may_create {
                        a class="btn sm" href={ "/new?owner=" (owner.id.as_str()) } { (ic("plus", "sm")) "New repository" }
                    }
                    @if who.viewer().is_none_or(|v| v.0 != owner.id) {
                        a class="ghost sm" href={ "/report?kind=abuse&place=%2F" (owner.id.as_str()) } title="Report this account to whoever runs the forge" { "Report" }
                    }
                }
            }
            @if let Some(error) = error { div class="notice bad" { (ic("alert", "")) span { (error) } } }
            @if let Some(link) = fresh {
                div class="once" {
                    p { b { "Hand them this link." } " It signs them in once and is shown only now; this forge could not mail it." }
                    code class="secret" { (link) }
                }
            }
            @if let Some(to) = mailed {
                div class="notice" { (ic("inbox", "")) span { "The invitation went to " (to) "." } }
            }
            @if asked {
                div class="notice" { (ic("check", "")) span { "Asked. Whoever runs the forge sees it with the reports, and answers from there." } }
            }
            @if let Some(record) = record {
                div class="sec" {
                    div class="sh" { h2 { "Record" } span class="n" { "the last " (record.window_days) " days, counted from the log" } }
                    div class="stats record" {
                        div class="stat" { div class="v" { (record.landed) } div class="k" { "landed" } }
                        div class="stat" { div class="v" { (record.abandoned) } div class="k" { "abandoned" } }
                        div class="stat" {
                            div class="v" {
                                @match record.reproduced_percent {
                                    Some(percent) => { (percent) small { "%" } }
                                    None => { "–" }
                                }
                            }
                            div class="k" { "of " (record.judged) " re-run claim" @if record.judged != 1 { "s" } " reproduced" }
                        }
                        div class="stat" { div class="v" { (record.claims) } div class="k" { "claim" @if record.claims != 1 { "s" } " with a command" } }
                        div class="stat" {
                            div class="v" { (record.audits_passed) small { " of " (record.audits) } }
                            div class="k" { "human looks passed" @if record.blocks > 0 { ", " (record.blocks) " blocked" } }
                        }
                        div class="stat" { div class="v" { (record.gaps_declared) } div class="k" { "gap" @if record.gaps_declared != 1 { "s" } " declared on claims" } }
                    }
                    p class="hint" { "Nothing here is declared. Claims a runner re-ran, verdicts by other people, and what landed or was abandoned, each with " (owner.display) "'s name on it in the log." }
                }
            }
            div class="sec" {
                div class="sh" { h2 { "Repositories" } span class="n" { (repos.len()) } }
                div class="panel" {
                    @if repos.is_empty() { div class="empty" { "Nothing here yet." } }
                    @for repo in repos {
                        @let short = ambolt_core::split_repo_name(&repo.name).map(|(_, s)| s).unwrap_or(&repo.name);
                        a class="row need" href={ "/" (repo.name) } {
                            @if repo.visibility == ambolt_core::Visibility::Public { span class="chip" { (ic("globe", "")) "Public" } } @else { span class="chip" { (ic("lock", "")) "Private" } }
                            span class="tt" {
                                span class="t" { (short) }
                                span class="s" { @if repo.description.is_empty() { (repo.name) } @else { (repo.description) } }
                            }
                            span class="avs" {}
                            span class="age" { @if repo.archived { "archived" } }
                        }
                    }
                }
            }
            @if organisation && viewer_role.is_some() || (organisation && who.viewer().is_some_and(|v| v.1.admin)) {
                div class="sec" {
                    div class="sh" { h2 { "Teams" } span class="n" { (teams.len()) } }
                    div class="panel" {
                        @if teams.is_empty() { div class="empty" { "None yet." } }
                        @for (name, count) in teams {
                            div class="row need" {
                                a class="t" href={ "/" (owner.id.as_str()) "/teams/" (name) } { (owner.id.as_str()) "/" (name) }
                                span class="s" { (count) @if *count == 1 { " member" } @else { " members" } }
                                span class="acts" {
                                    @if may_manage {
                                        form method="post" action={ "/" (owner.id.as_str()) "/members" } {
                                            input type="hidden" name="action" value="team-remove";
                                            input type="hidden" name="name" value=(name);
                                            button class="ghost sm danger" type="submit" { "Remove" }
                                        }
                                    }
                                }
                            }
                        }
                        @if may_manage {
                            form class="foot" method="post" action={ "/" (owner.id.as_str()) "/members" } {
                                input type="hidden" name="action" value="team-make";
                                input class="input sm" type="text" name="name" placeholder="Team name" pattern="[a-z0-9-]{2,64}" required aria-label="Team name";
                                button class="btn2 sm" type="submit" { "Make a team" }
                                span class="hint" { "Give it access from a repository's settings." }
                            }
                        }
                    }
                }
                @if let Some(mode) = members_act {
                    div class="sec" {
                        div class="sh" { h2 { "Access" } span class="n" { "what being a member means" } }
                        div class="panel" {
                            form class="pref" method="post" action={ "/" (owner.id.as_str()) "/members" } {
                                input type="hidden" name="action" value="access";
                                div class="seg radio" {
                                    label { input type="radio" name="mode" value="owners" checked[mode == ambolt_core::MembersAct::Owners] disabled[!may_manage]; span { "Members act as owners" } }
                                    label { input type="radio" name="mode" value="readers" checked[mode == ambolt_core::MembersAct::Readers] disabled[!may_manage]; span { "Members read; access is granted" } }
                                }
                                p class="what" { "Owners: every member can do everything on every repository. Readers: members read and create; the rest is granted." }
                                @if may_manage { div class="acts" { button class="btn2 sm" type="submit" { "Save" } } }
                            }
                        }
                    }
                }
            }
            @if let Some((usage, quota)) = allowances {
                div class="sec" {
                    div class="sh" { h2 { "Allowance" } span class="n" { "what this account uses, of what it may" } }
                    div class="panel" {
                        (allowance("Repositories", usage.repos.to_string(), quota.repos.map(|n| n.to_string())))
                        (allowance("Disk", crate::in_bytes(usage.disk), quota.disk.map(crate::in_bytes)))
                        (allowance("Agents", usage.agents.to_string(), quota.agents.map(|n| n.to_string())))
                        (allowance("Open tasks", usage.open_tasks.to_string(), quota.open_tasks.map(|n| n.to_string())))
                        (allowance("Open changes", usage.open_changes.to_string(), quota.open_changes.map(|n| n.to_string())))
                        (allowance("Tokens", usage.tokens.to_string(), quota.tokens.map(|n| n.to_string())))
                        @if organisation { (allowance("Members", usage.members.to_string(), quota.members.map(|n| n.to_string()))) }
                        @if organisation && may_manage {
                            form class="foot" method="post" action={ "/" (owner.id.as_str()) "/members" } {
                                input type="hidden" name="action" value="ask";
                                select class="input sm" name="ask" aria-label="Ask for" {
                                    option value="allowance" { "More allowance" }
                                    option value="forge" { "A forge of our own" }
                                }
                                input class="input sm" type="text" name="note" placeholder="What, and why" aria-label="Note";
                                button class="btn2 sm" type="submit" { "Ask" }
                                span class="hint" { "Whoever runs the forge answers." }
                            }
                        }
                    }
                }
            }
            @if organisation {
                div class="sec" {
                    div class="sh" { h2 { "Members" } span class="n" { (members.len()) } }
                    div class="panel" {
                        @if members.is_empty() { div class="empty" { "Nobody yet." } }
                        @for (member, role) in members {
                            @let (display, agent) = people.name(member);
                            @let is_owner = *role == ambolt_core::TeamRole::Owner;
                            div class="row member" {
                                (avatar(member.as_str(), display, agent, false))
                                span class="tt" {
                                    a class="t" href={ "/" (member.as_str()) } { (display) }
                                    span class="s" { (member.as_str()) }
                                }
                                span class="tags" {
                                    @if is_owner { span class="chip acc" { "Owner" } } @else { span class="chip" { "Member" } }
                                    @if invited.contains(member) { span class="chip" { "Invited" } }
                                }
                                span class="acts" {
                                    @if may_manage {
                                        form method="post" action={ "/" (owner.id.as_str()) "/members" } {
                                            input type="hidden" name="action" value=(if is_owner { "member" } else { "owner" });
                                            input type="hidden" name="member" value=(member.as_str());
                                            button class="ghost sm" type="submit" { @if is_owner { "Make member" } @else { "Make owner" } }
                                        }
                                        form method="post" action={ "/" (owner.id.as_str()) "/members" } {
                                            input type="hidden" name="action" value="remove";
                                            input type="hidden" name="member" value=(member.as_str());
                                            button class="ghost sm danger" type="submit" { "Remove" }
                                        }
                                    } @else if viewer_role.is_some() && who.viewer().is_some_and(|v| v.0 == *member) {
                                        form method="post" action={ "/" (owner.id.as_str()) "/members" } {
                                            input type="hidden" name="action" value="remove";
                                            input type="hidden" name="member" value=(member.as_str());
                                            button class="ghost sm danger" type="submit" { "Leave" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    @if may_manage {
                        div class="grid2 tight" {
                            div class="panel" {
                                form class="pref" method="post" action={ "/" (owner.id.as_str()) "/members" } {
                                    input type="hidden" name="action" value="add";
                                    h3 { "Add a member" }
                                    div class="field" {
                                        label for="add-member" { "Username" }
                                        input id="add-member" class="input" type="text" name="member" placeholder="Already on the forge" pattern="[a-z0-9-]{2,64}" required;
                                    }
                                    div class="acts" { button class="btn2" type="submit" { "Add member" } }
                                }
                            }
                            div class="panel" {
                                form class="pref" method="post" action={ "/" (owner.id.as_str()) "/members" } {
                                    input type="hidden" name="action" value="invite";
                                    h3 { "Invite someone new" }
                                    div class="field" {
                                        label for="invite-member" { "Username" }
                                        input id="invite-member" class="input" type="text" name="member" placeholder="jane" pattern="[a-z0-9-]{2,64}" required;
                                    }
                                    div class="field" {
                                        label for="invite-display" { "Name" }
                                        input id="invite-display" class="input" type="text" name="display" placeholder="Jane Okoro";
                                    }
                                    div class="field" {
                                        label for="invite-email" { "Email" }
                                        input id="invite-email" class="input" type="email" name="email" placeholder="jane@example.org" required;
                                        p class="hint" { "A sign-in link goes there. The account and the membership are theirs." }
                                    }
                                    div class="acts" { button class="btn2" type="submit" { "Invite" } }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

/// Where anyone says what broke. Works signed out, since the person most
/// likely to have hit something is the one who could not get in.
pub fn report(
    theme: Theme,
    viewer: Option<&Viewer>,
    filed: Option<i64>,
    error: Option<&str>,
    // Brought here by the link on a repository or a person's page.
    abuse: bool,
    place: &str,
) -> Markup {
    let title = if abuse {
        "Report this"
    } else {
        "Say what broke"
    };
    let body = html! {
        @if abuse {
            p class="hint" { "Say what is wrong with it, in a sentence or two. Whoever runs this forge reads every report and can hide a repository or stop an account; nothing is done by a machine." }
        } @else {
            p class="hint" { "What you did, what you expected, and what happened instead. The version of this forge is recorded with it. An address is optional; leave one if you want to hear back." }
        }
        @if let Some(id) = filed {
            div class="notice" { (ic("check", "")) span { "Recorded as report " (id) ". Thank you." } }
        }
        @if let Some(error) = error { div class="notice bad" { (ic("alert", "")) span { (error) } } }
        form class="form" method="post" action="/report" {
            input type="hidden" name="kind" value=(if abuse { "abuse" } else { "bug" });
            div class="field" {
                label for="what" { @if abuse { "What is wrong" } @else { "What happened" } }
                textarea id="what" name="what" rows="7" required minlength="10" placeholder=(if abuse { "This repository's name is …" } else { "I pushed a change with … and the page said …" }) {}
            }
            div class="field" {
                label for="place" { @if abuse { "What it is about" } @else { "Where" } }
                input id="place" name="place" type="text" maxlength="300" value=(place) placeholder="A page, a command, a change number";
            }
            div class="field" {
                label for="contact" { "How to reach you" }
                input id="contact" name="contact" type="email" autocomplete="email" placeholder="Optional";
            }
            button class="btn wide" type="submit" { "Send the report" }
        }
        p class="hint" { "Reports are kept beside the waitlist and not in the log, so one can be removed when you ask." }
    };
    match viewer {
        Some(viewer) => layout(
            theme,
            Some(viewer),
            None,
            None,
            "Report",
            html! { div class="sec top" { div class="panel narrow" { header { (ic("alert", "")) h2 { (title) } } div class="pad" { (body) } } } },
        ),
        None => outside(theme, title, body),
    }
}

/// What the Explore page is given.
pub struct ExplorePage<'a> {
    /// What the search and the topic narrowed it to.
    pub entries: &'a [ambolt_core::ExploreEntry],
    /// Everything public, for the topic bar.
    pub all: &'a [ambolt_core::ExploreEntry],
    pub topic: Option<&'a str>,
    pub q: Option<&'a str>,
    pub sort: ambolt_core::ExploreSort,
    pub people: &'a People,
}

/// Every public repository, as cards: who owns it, what it is for, what it
/// is filed under, and three numbers with their names. Where a stranger
/// starts, and it needs nobody signed in.
pub fn explore(theme: Theme, who: Reading<'_>, page: ExplorePage<'_>) -> Markup {
    let ExplorePage {
        entries,
        all,
        topic,
        q,
        sort,
        people,
    } = page;
    let mut topics: Vec<(&str, usize)> = Vec::new();
    for entry in all {
        for t in &entry.topics {
            match topics.iter_mut().find(|(name, _)| *name == t.as_str()) {
                Some((_, n)) => *n += 1,
                None => topics.push((t.as_str(), 1)),
            }
        }
    }
    topics.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    // A link to this page with one parameter changed and the rest kept.
    let href = |topic: Option<&str>, sort: ambolt_core::ExploreSort| {
        let mut parts: Vec<String> = Vec::new();
        if let Some(q) = q {
            parts.push(format!("q={}", super::urlencode(q)));
        }
        if let Some(topic) = topic {
            parts.push(format!("topic={}", super::urlencode(topic)));
        }
        if sort != ambolt_core::ExploreSort::Busiest {
            parts.push(format!("sort={}", sort.as_str()));
        }
        if parts.is_empty() {
            "/explore".to_owned()
        } else {
            format!("/explore?{}", parts.join("&"))
        }
    };
    let sorts = [
        (ambolt_core::ExploreSort::Busiest, "Busiest"),
        (ambolt_core::ExploreSort::Newest, "Newest"),
        (ambolt_core::ExploreSort::Reproduced, "Most reproduced"),
        (ambolt_core::ExploreSort::Name, "Name"),
    ];
    layout_reading(
        theme,
        who,
        None,
        None,
        "Explore",
        html! {
            form class="xbar" method="get" action="/explore" {
                @if let Some(topic) = topic { input type="hidden" name="topic" value=(topic); }
                @if sort != ambolt_core::ExploreSort::Busiest { input type="hidden" name="sort" value=(sort.as_str()); }
                label class="xsearch" {
                    (ic("search", "sm"))
                    input type="search" name="q" value=[q] placeholder="Search public repositories" aria-label="Search public repositories" autocomplete="off";
                }
                div class="seg" role="group" aria-label="Order" {
                    @for (key, words) in sorts {
                        a class={ @if sort == key { "on" } } href=(href(topic, key)) { (words) }
                    }
                }
            }
            @if !topics.is_empty() {
                div class="xtopics" {
                    a class={ "chip" @if topic.is_none() { " on" } } href=(href(None, sort)) { "All" span class="n" { (all.len()) } }
                    @for (name, n) in &topics {
                        a class={ "chip" @if topic == Some(*name) { " on" } } href=(href(Some(name), sort)) { "#" (name) span class="n" { (n) } }
                    }
                }
            }
            @if entries.is_empty() {
                div class="panel" {
                    div class="empty" {
                        @if q.is_some() { b { "Nothing matches." } "Try fewer words." }
                        @else if topic.is_some() { b { "Nothing is filed under that." } "Pick another topic." }
                        @else { b { "No public repository yet." } "A repository goes public from its settings, and shows up here." }
                    }
                }
            } @else {
                div class="xgrid" {
                    @for entry in entries {
                        @let (owner, short) = entry.name.split_once('/').unwrap_or(("", entry.name.as_str()));
                        a class="xcard" href={ "/" (entry.name) } {
                            div class="xhead" {
                                (avatar_of(entry.owner.as_str(), people.name(&entry.owner).0, entry.owner_kind, false))
                                span class="xname" { span class="o" { (owner) "/" } (short) }
                                @if entry.archived { span class="chip" { (ic("archive", "")) "Archived" } }
                            }
                            @if entry.description.is_empty() {
                                p class="xdesc none" { "No description yet." }
                            } @else {
                                p class="xdesc" { (entry.description) }
                            }
                            @if !entry.topics.is_empty() {
                                div class="xtags" { @for t in &entry.topics { span class="tag" { "#" (t) } } }
                            }
                            div class="xstats" {
                                div class="xs" { b { (entry.landed_week) } span { "landed this week" } }
                                div class="xs" { b { (entry.open) } span { "open" } }
                                div class="xs" {
                                    @match entry.coverage_percent {
                                        Some(p) => { b { (p) "%" } span { "reproduced" } }
                                        None => { b class="none" { "–" } span { "not measured" } }
                                    }
                                }
                                @if let Some(at) = &entry.updated_at {
                                    span class="xage" title=(at) { (ic("clock", "sm")) (ago(at)) }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

/// What was reported and not yet dealt with, newest first, for whoever
/// runs the forge.
pub fn reports(
    theme: Theme,
    viewer: &Viewer,
    reports: &[ambolt_core::Report],
    error: Option<&str>,
) -> Markup {
    layout_section_head(
        theme,
        viewer,
        "reports",
        "Reports",
        Head {
            count: Some(reports.len().to_string()),
            acts: None,
        },
        html! {
            @if let Some(error) = error { div class="notice bad" { (ic("alert", "")) span { (error) } } }
            div class="sec top" {
                div class="panel" {
                    @if reports.is_empty() {
                        div class="empty" { b { "Nothing reported." } "The form is at " a href="/report" { "/report" } "." }
                    }
                    @for report in reports {
                        @let place_path = report.place.as_deref().filter(|p| p.starts_with('/') && p.len() > 1);
                        @let about_repo = place_path.is_some_and(|p| p.trim_start_matches('/').contains('/'));
                        div class="row ls" {
                            span class="chip" { "#" (report.id) }
                            @if report.kind == "abuse" { span class="chip bad" { "Abuse" } }
                            span class="tt" {
                                (kv(&[
                                    ("Filed", html! { (report.filed.get(..16).unwrap_or(&report.filed).replace('T', " ")) }),
                                    ("Version", html! { (report.version) }),
                                    ("About", html! {
                                        @match &report.place {
                                            Some(place) => { @if let Some(path) = place_path { a href=(path) { (place) } } @else { (place) } }
                                            None => { span class="none" { "nowhere in particular" } }
                                        }
                                    }),
                                ]))
                                span class="s wrap" { (report.what) }
                                span class="s" {
                                    @if let Some(contact) = &report.contact {
                                        a href={ "mailto:" (contact) } { (contact) }
                                        @if let Some(by) = &report.by { ", signed in as " b { (by) } }
                                    } @else if let Some(by) = &report.by {
                                        "signed in as " b { (by) }
                                    } @else {
                                        "no address left"
                                    }
                                }
                            }
                            span class="acts" {
                                @if let Some(path) = place_path.filter(|_| report.kind == "abuse") {
                                    @if about_repo {
                                        form method="post" action="/reports" {
                                            input type="hidden" name="id" value=(report.id);
                                            input type="hidden" name="action" value="hide";
                                            input type="hidden" name="target" value=(path);
                                            button class="ghost sm" type="submit" title="Set the repository private" { "Hide" }
                                        }
                                    } @else {
                                        form method="post" action="/reports" {
                                            input type="hidden" name="id" value=(report.id);
                                            input type="hidden" name="action" value="stop";
                                            input type="hidden" name="target" value=(path);
                                            button class="ghost sm danger" type="submit" title="Deactivate the account" { "Stop" }
                                        }
                                    }
                                }
                                form method="post" action="/reports" {
                                    input type="hidden" name="id" value=(report.id);
                                    input type="hidden" name="action" value="dismiss";
                                    button class="ghost sm" type="submit" { "Dismiss" }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

/// Everyone with an account, for whoever runs the forge: who can sign
/// in, who is invited, who is gone, and a way to add someone.
pub fn people(
    theme: Theme,
    viewer: &Viewer,
    people: &[super::PersonRow],
    can_mail: bool,
    join_link: Option<&str>,
    mailed: Option<&str>,
    error: Option<&str>,
) -> Markup {
    layout_section_head(
        theme,
        viewer,
        "people",
        "People",
        Head {
            count: Some(people.len().to_string()),
            acts: Some(
                html! { a class="btn sm" href="#add" { (ic("plus", "sm")) "Add a person" } },
            ),
        },
        html! {
            @if let Some(error) = error { div class="notice bad" { (ic("alert", "")) span { (error) } } }
            @if let Some(link) = join_link {
                div class="once" {
                    @if let Some(to) = mailed {
                        p { b { "Sent to " (to) "." } " The same link is here in case it does not arrive; it signs them in once, then it is spent." }
                    } @else {
                        p { b { "Send them this link." } " It signs them in once, then it is spent; this is the only time it can be shown." }
                    }
                    code class="secret" { (link) }
                }
            }
            div class="sec top" {
                div class="panel" {
                    @for row in people {
                        @let id = row.principal.id.as_str();
                        div class="row person" {
                            (avatar(id, &row.principal.display, false, false))
                            span class="tt" {
                                span class="t" { (row.principal.display) }
                                span class="s" {
                                    (id)
                                    @if !row.principal.active { " · deactivated" }
                                    @else if row.admin { " · runs the forge" }
                                    @else if row.has_password { " · can sign in" }
                                    @else { " · no password yet" }
                                    @match (&row.contact.email, &row.contact.pending) {
                                        (Some(_), _) => { " · email confirmed" }
                                        (None, Some(_)) => { " · email pending" }
                                        (None, None) => { " · no email" }
                                    }
                                    @if let Some(invite) = &row.invitation {
                                        " · invited"
                                        @if let Some(until) = &invite.until { ", link good until " (day_of(until)) }
                                    }
                                }
                            }
                            span class="acts" {
                                @if row.principal.active {
                                    form method="post" action="/people" {
                                        input type="hidden" name="action" value="relink";
                                        input type="hidden" name="id" value=(id);
                                        button class="ghost sm" type="submit" {
                                            (ic("send", "sm"))
                                            @if can_mail && (row.contact.email.is_some() || row.contact.pending.is_some()) { "Send a new sign-in link" } @else { "Make a sign-in link" }
                                        }
                                    }
                                }
                                @if row.principal.id != viewer.0 {
                                    form method="post" action="/people" {
                                        input type="hidden" name="action" value={ @if row.principal.active { "deactivate" } @else { "reactivate" } };
                                        input type="hidden" name="id" value=(id);
                                        // What goes down with them is said on the
                                        // control, and that it does not come back
                                        // with them: retired agents are brought back
                                        // one at a time, by whoever holds them.
                                        @if row.principal.active {
                                            button class="ghost sm danger" type="submit" {
                                                @match row.agents {
                                                    0 => { "Deactivate" }
                                                    1 => { "Deactivate, and their agent" }
                                                    n => { "Deactivate, and their " (n) " agents" }
                                                }
                                            }
                                        } @else {
                                            button class="ghost sm" type="submit" title="Their retired agents stay retired until brought back one by one" { "Reactivate" }
                                        }
                                    }
                                }
                                @if row.invitation.is_some() {
                                    form method="post" action="/people" {
                                        input type="hidden" name="action" value="cancel";
                                        input type="hidden" name="id" value=(id);
                                        button class="ghost sm danger" type="submit" { "Cancel invitation" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div class="sec" {
                div class="panel narrow" id="add" {
                    header { (ic("plus", "")) h2 { "Add a person" } }
                    form class="pad form" method="post" action="/people" {
                        input type="hidden" name="action" value="register";
                        div class="field" {
                            label for="id" { "Username" }
                            input id="id" name="id" type="text" autocomplete="off" placeholder="lowercase, digits and hyphens" required;
                        }
                        div class="field" {
                            label for="display" { "Display name" }
                            input id="display" name="display" type="text" autocomplete="off";
                        }
                        div class="field" {
                            label for="email" { "Email" }
                            input id="email" name="email" type="email" autocomplete="off"
                                  placeholder=(if can_mail { "The invitation goes here" } else { "Optional; kept for password resets" });
                        }
                        div class="acts" {
                            button class="btn" type="submit" {
                                @if can_mail { "Add and send an invitation" } @else { "Add and make a link" }
                            }
                        }
                    }
                }
            }
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub fn agents(
    theme: Theme,
    viewer: &Viewer,
    agents: &[super::AgentRow],
    repos: &[String],
    // Who a new agent may belong to: the viewer, and every organisation
    // they are in.
    owners: &[String],
    fresh: Option<&str>,
    error: Option<&str>,
    people: &People,
) -> Markup {
    layout_section_head(
        theme,
        viewer,
        "agents",
        "Agents",
        Head {
            count: Some(agents.len().to_string()),
            acts: Some(
                html! { a class="btn sm" href="#add" { (ic("plus", "sm")) "Add an agent" } },
            ),
        },
        html! {
            @if let Some(error) = error { div class="notice bad" { (ic("alert", "")) span { (error) } } }
            @if let Some(secret) = fresh {
                div class="once" {
                    p { b { "Copy this now." } " It is the agent's only credential, and it is stored as a hash." }
                    code class="secret" { (secret) }
                }
            }
            @if agents.is_empty() {
                div class="panel" { div class="empty" { b { "None yet." } "An agent needs a name, a token, and a grant narrow enough to be worth trusting." } }
            }
            div class="grid2" {
                @for row in agents {
                    @let id = row.principal.id.as_str();
                    @let live = viewer.1.working.iter().any(|w| w.who == id);
                    @let mine = viewer.1.admin || row.principal.owner.as_ref().is_some_and(|o| owners.iter().any(|mine| mine == o.as_str()));
                    div class="panel agent" {
                        header {
                            (avatar(id, &row.principal.display, true, live))
                            div class="tt" {
                                h2 { (row.principal.display) @if !row.principal.active { " " span class="chip" { "Retired" } } }
                                span class="s" { code { (id) } }
                            }
                            @if row.principal.active && mine {
                                details class="more" {
                                    summary class="ghost sm" aria-label="More" { (ic("more", "sm")) }
                                    div class="pop" {
                                        form method="post" action="/agents" {
                                            input type="hidden" name="action" value="mint";
                                            input type="hidden" name="grantee" value=(id);
                                            button type="submit" { (ic("key", "sm")) "New token" }
                                        }
                                        form method="post" action="/agents" {
                                            input type="hidden" name="action" value="retire";
                                            input type="hidden" name="grantee" value=(id);
                                            button class="danger" type="submit" { (ic("archive", "sm")) "Retire" }
                                        }
                                    }
                                }
                            }
                        }
                        div class="pad ag" {
                            (kv(&[
                                ("Model", html! { @match &row.principal.model { Some(m) => { (m) } None => { span class="none" { "not said" } } } }),
                                ("Harness", html! { @match &row.principal.harness { Some(h) => { (h) } None => { span class="none" { "not said" } } } }),
                                ("Held by", html! { @match &row.principal.owner { Some(o) => { a href={ "/" (o.as_str()) } { (people.name(o).0) } } None => { span class="none" { "nobody" } } } }),
                            ]))
                            div {
                                span class="lbl" { "Can" }
                                @if row.principal.active && row.grants.iter().all(|g| g.revoked) {
                                    div class="s" { "Nothing yet: no live grant." }
                                }
                                @for grant in row.grants.iter().filter(|g| !g.revoked) {
                                    div class="line" {
                                        @for action in &grant.actions { span class="chip" { (action.as_str()) } }
                                        span class="s" {
                                            @match &grant.repo {
                                                // A repository the viewer cannot read is not
                                                // named on their page, even on an agent of theirs:
                                                // the operator granted it, and the name is the
                                                // operator's to share.
                                                Some(repo) if viewer.1.admin || viewer.1.repos.iter().any(|r| &r.name == repo) => { "on " (repo) }
                                                Some(_) => { "on a repository not yours to see" }
                                                None => { "everywhere" }
                                            }
                                        }
                                        // Only a control that will work: revoking is the
                                        // grantor's or the grantee's, or the forge's.
                                        @if viewer.1.admin || grant.grantor == viewer.0 {
                                            form method="post" action="/agents" {
                                                input type="hidden" name="action" value="revoke";
                                                input type="hidden" name="grant" value=(grant.id.0);
                                                button class="ghost sm danger" type="submit" { "Revoke" }
                                            }
                                        }
                                    }
                                }
                            }
                            div {
                                span class="lbl" { "Record, " (row.record.window_days) " days" }
                                div class={ "s" @if row.record.disputed > 0 || row.record.blocks > 0 { " bad-t" } } { (record_words(&row.record)) }
                            }
                            div {
                                span class="lbl" { "Now" }
                                div class="s" {
                                    @match viewer.1.working.iter().find(|w| w.who == id) {
                                        Some(work) => {
                                            "Working"
                                            @if let Some(repo) = &work.repo { " on " (repo) }
                                            @if let Some(path) = work.paths.first() { ", in " code { (path) } }
                                        }
                                        None => { @if row.principal.active { "Idle" } @else { "Retired" } }
                                    }
                                }
                            }
                            // A retired agent takes no grant; the control that
                            // would only refuse is not shown.
                            @if row.principal.active && mine {
                                details class="fold" {
                                    summary class="ghost sm" { (ic("plus", "sm")) "Grant something" }
                                    form class="form" method="post" action="/agents" {
                                        input type="hidden" name="action" value="grant";
                                        input type="hidden" name="grantee" value=(id);
                                        div class="line" {
                                            @for capability in ["task", "push", "review", "merge", "verify"] {
                                                label class="check" {
                                                    input type="checkbox" name=(capability) value="on";
                                                    span { (capability) }
                                                }
                                            }
                                        }
                                        div class="line" {
                                            select class="input sm" name="repo" aria-label="Where" {
                                                // A grant everywhere is running the forge; offering it
                                                // to somebody who cannot make it is a control that
                                                // only ever refuses.
                                                @if viewer.1.admin { option value="" { "Everywhere" } }
                                                @for repo in repos { option value=(repo) { (repo) } }
                                            }
                                            button class="btn2 sm" type="submit" { "Grant" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                }
                div class="panel narrow" id="add" {
                    header { (ic("plus", "")) h2 { "Add an agent" } }
                    form class="pad form" method="post" action="/agents" {
                        input type="hidden" name="action" value="register";
                        div class="field" {
                            label for="id" { "Name" }
                            input id="id" name="id" type="text" autocomplete="off" placeholder="scribe" required;
                            span class="hint" { "Lowercase letters, digits and hyphens. This is how it signs everything it does." }
                        }
                        div class="field" {
                            label for="display" { "Display name" }
                            input id="display" name="display" type="text" autocomplete="off" placeholder="Scribe";
                        }
                        div class="field" {
                            label for="model" { "Model" }
                            input id="model" name="model" type="text" autocomplete="off" placeholder="claude-fable-5-1";
                            span class="hint" { "Two agents of different models can approve a change between them; two of the same model cannot." }
                        }
                        @if owners.len() > 1 {
                            div class="field" {
                                label for="owner" { "Belongs to" }
                                select id="owner" name="owner" {
                                    @for owner in owners { option value=(owner) { (owner) } }
                                }
                            }
                        }
                        div class="acts" {
                            button class="btn" type="submit" { "Add and mint a token" }
                            span class="hint" { "The token is shown once. It grants nothing by itself." }
                        }
                    }
                }
        },
    )
}

/// One sentence, framed like the other pages a stranger can meet.
pub fn plain_note(theme: Theme, text: &str) -> Markup {
    outside(
        theme,
        "Not shown",
        html! { p class="plain" { (text) } p class="hint" { a href="/" { "Home" } } },
    )
}

pub fn error_page(theme: Theme) -> Markup {
    outside(
        theme,
        "Something went wrong",
        html! {
            p class="plain" { "On our side, not yours. The log has the details." }
            p class="hint" { a href="/" { "Home" } }
        },
    )
}

pub fn not_found_page(theme: Theme) -> Markup {
    outside(
        theme,
        "Nothing lives here",
        html! {
            p class="plain" { "The address may be wrong, or this may be something you cannot see." }
            p class="hint" { a href="/" { "Home" } }
        },
    )
}

/// A write that did not come from this site: another site's form, or
/// a page left open past a sign-in. The person is sent back to try
/// from here.
pub fn not_from_here_page(theme: Theme) -> Markup {
    outside(
        theme,
        "Not from here",
        html! {
            p class="plain" { "That was sent from another site, or from a page that has gone stale. Go back to the forge and try again." }
            p class="hint" { a href="/" { "Home" } }
        },
    )
}

pub fn too_many_page(theme: Theme) -> Markup {
    outside(
        theme,
        "Too many requests",
        html! {
            p class="plain" { "You have asked for a lot in a short time. Wait a moment and try again." }
            p class="hint" { a href="/" { "Home" } }
        },
    )
}

#[allow(clippy::too_many_arguments)]
/// The rules a branch lands under, as the sentences the change page
/// uses, so the same words appear wherever a rule is named.
fn rule_words(policy: &ambolt_core::Policy) -> Vec<String> {
    let mut rules = Vec::new();
    if policy.require_executed_check {
        rules.push("Tests pass on the judged revision".to_owned());
    }
    if policy.require_runner_verification {
        rules.push(if policy.runner_quorum > 1 {
            format!("{} runners reproduced the same claim", policy.runner_quorum)
        } else {
            "A runner reproduced a claim".to_owned()
        });
    }
    rules.push(match policy.independence {
        Independence::HumanOrTwoModels => "Someone other than the author approves it".to_owned(),
        Independence::HumanOnly => "A person approves it".to_owned(),
        Independence::Anyone => "Anyone but the owner approves it".to_owned(),
        Independence::None => "Nobody has to approve it".to_owned(),
    });
    if policy.require_concerns_resolved {
        rules.push("Every concern is resolved".to_owned());
    }
    for domain in &policy.required_domains {
        rules.push(format!("Reviewed for {}", domain.as_str()));
    }
    rules
}

pub struct RepoPage<'a> {
    pub theme: Theme,
    pub who: Reading<'a>,
    pub repo: &'a Repo,
    /// Whether the signed-in viewer saved it; none for a stranger.
    pub saved: Option<bool>,
    /// Whether the signed-in viewer watches it; none for a stranger.
    pub watching: Option<bool>,
    /// The viewer may propose here and not push: their push opens a
    /// proposal, and the opener says so.
    pub proposer: bool,
    pub tip: Option<&'a str>,
    pub path: &'a str,
    pub entries: &'a [Entry],
    pub readme: Option<&'a str>,
    pub sidebar: &'a Sidebar,
    pub clone_url: &'a str,
    pub people: &'a People,
}

pub fn repository(page: RepoPage<'_>) -> Markup {
    let RepoPage {
        theme,
        who,
        repo,
        saved,
        watching,
        proposer,
        tip,
        path,
        entries,
        readme,
        sidebar,
        clone_url,
        people,
    } = page;
    let name = repo.name.as_str();
    let branch = repo.default_branch.as_str();
    let short_name = ambolt_core::split_repo_name(name)
        .map(|(_, s)| s)
        .unwrap_or(name);
    let owner = who
        .viewer()
        .is_some_and(|v| v.1.admin || v.1.owned.iter().any(|r| r == name));
    let rail = html! {
        div class="panel" {
            header { h2 { "Rules of " (branch) } }
            div class="pad reqs" {
                @for rule in rule_words(&repo.policy) {
                    div class="req met" { span class="st" { (ic("check", "")) } div { b { (rule) } } }
                }
            }
            @if owner {
                div class="foot" { a href={ "/" (name) "/settings#policy" } { "Change the rules" } }
            }
        }
        div class="panel" {
            header { h2 { "Open changes" } span class="n" { (sidebar.open_changes.len()) } }
            @if sidebar.open_changes.is_empty() { div class="empty" { "None open." } }
            @for change in &sidebar.open_changes {
                @let (display, agent) = people.name(&change.owner);
                a class="ev-row" href={ "/" (name) "/changes/" (change.number) } {
                    (avatar(change.owner.as_str(), display, agent, false))
                    div {
                        div class="h" { b { "#" (change.number) " " (change.title) } }
                        div class="sub" { (kv(&[("By", html! { (display) }), ("Revision", html! { (change.latest_revision) })])) }
                    }
                }
            }
        }
        @if !sidebar.queue.is_empty() {
            div class="panel" {
                header { h2 { "Landing on " (branch) } span class="n" { (sidebar.queue.len()) } }
                @for (index, entry) in sidebar.queue.iter().enumerate() {
                    div class="ev-row" {
                        span class={ "chip" @if index == 0 { " acc" } } { (index + 1) }
                        div { div class="h" { b { (change_short(&sidebar.numbers, entry.change.as_str())) } } }
                    }
                }
            }
        }
        div class="panel" {
            header { h2 { "At work here" } span class="n" { (sidebar.sessions.len()) } }
            @if sidebar.sessions.is_empty() {
                div class="empty" { b { "No agent is working here." } "Grant one push on this repository and it can start." }
            }
            @for session in &sidebar.sessions {
                @let held = sidebar.leases.iter().find(|l| l.session == session.id);
                @let (display, agent) = people.name(&session.agent);
                div class="ev-row" {
                    (avatar(session.agent.as_str(), display, agent, true))
                    div {
                        div class="h" { b { (display) } }
                        @match held {
                            Some(lease) => { span class="cmd" { (lease.paths.join(", ")) } }
                            None => { div class="sub" { "working" } }
                        }
                    }
                }
            }
        }
        @if !sidebar.tags.is_empty() {
            div class="panel" {
                header { h2 { "Tags" } span class="n" { (sidebar.tags.len()) } }
                @for tag in sidebar.tags.iter().take(8) {
                    div class="row tag" {
                        span class="chip" { (ic("tag", "")) (tag.name) }
                        (kv(&[("Commit", html! { code { (short(&tag.commit_oid)) } }), ("By", html! { (people.name(&tag.by).0) })]))
                    }
                }
            }
        }
    };
    layout_reading_with(
        theme,
        who,
        Some(name),
        Some(Tab::Code),
        name,
        html! {
            div class="repometa" {
                div class="about" {
                    @if !repo.description.is_empty() { p class="sub" { (repo.description) } }
                    @else if tip.is_none() { p class="sub" { "Nothing here yet." } }
                    div class="chips" {
                        @if repo.visibility == Visibility::Public { span class="chip" { (ic("globe", "")) "Public" } } @else { span class="chip" { (ic("lock", "")) "Private" } }
                        span class="chip" { (ic("branch", "")) (branch) }
                        @for topic in &repo.topics {
                            a class="chip" href={ "/explore?topic=" (topic) } { "#" (topic) }
                        }
                        @if let Some(tag) = sidebar.tags.first() { span class="chip" { (ic("tag", "")) (tag.name) } }
                        @if let Some(tip) = tip { span class="chip" { code { (short(tip)) } } }
                        @if repo.archived { span class="chip bad" { (ic("archive", "")) "Archived" } }
                    }
                }
                div class="acts" {
                    @if repo.visibility == Visibility::Public {
                        a class="ghost sm" href={ "/report?kind=abuse&place=%2F" (name.replace('/', "%2F")) } title="Report this repository to whoever runs the forge" { "Report" }
                    }
                    @if let Some(watching) = watching {
                        form method="post" action={ "/" (name) "/watch" } {
                            input type="hidden" name="action" value=(if watching { "unwatch" } else { "watch" });
                            button class={ "btn2 sm" @if watching { " on" } } type="submit" title=(if watching { "You hear when something lands here or needs a person; click to stop" } else { "Hear when something lands here, or needs a person" }) {
                                (ic("bell", "sm")) @if watching { "Watching" } @else { "Watch" }
                            }
                        }
                    }
                    @if let Some(saved) = saved {
                        form method="post" action={ "/" (name) "/save" } {
                            input type="hidden" name="action" value=(if saved { "unsave" } else { "save" });
                            button class={ "btn2 sm" @if saved { " on" } } type="submit" title=(if saved { "In your sidebar; click to let it go" } else { "Keep it in your sidebar" }) {
                                (ic("bookmark", "sm")) @if saved { "Saved" } @else { "Save" }
                            }
                        }
                    }
                    details class="clone" {
                        summary class="btn2 sm" { (ic("download", "sm")) "Clone" }
                        div class="pop" {
                            code id="clone-url" { "git clone " (clone_url) }
                            span class="s" { "git asks for a password: paste a token of yours." }
                            button class="ghost sm" type="button" data-copy="clone-url" { (ic("copy", "sm")) "Copy" }
                        }
                    }
                }
            }
            @if tip.is_none() {
                div class="first anvil" {
                    h2 { "Put something on the anvil" }
                    @if proposer {
                        p class="sec2" { "You may propose here. Push from a clone and a proposal opens; someone inside reviews it, asks a runner to run it, and lands it." }
                    } @else {
                        p class="sec2" { "Push from a clone you already have, import history, or let an agent start. The first push opens a change, and the change is what gets reviewed and landed." }
                    }
                    div class="seg" data-tabs="first" {
                        button class="on" type="button" data-pane="first-push" { (ic("terminal", "sm")) @if proposer { "Propose a change" } @else { "Push a change" } }
                        button type="button" data-pane="first-import" { (ic("download", "sm")) "Import history" }
                        button type="button" data-pane="first-agent" { (ic("agents", "sm")) "Let an agent start" }
                    }
                    div class="code" id="first-push" data-pane-of="first" {
                        header { code { "terminal" } div class="right" { button class="ghost sm" type="button" data-copy="first-push-cmd" { (ic("copy", "sm")) "Copy" } } }
                        pre class="lines" { code class="src" id="first-push-cmd" { "git remote add ambolt " (clone_url) "\ngit push ambolt HEAD:refs/for/" (branch) "\n# git asks for a password: paste a token of yours" } }
                    }
                    div class="code" id="first-import" data-pane-of="first" {
                        header { code { "terminal" } }
                        pre class="lines" { code class="src" { "ambolt import https://github.com/you/" (short_name) ".git\n# what arrives is recorded as imported, never as reviewed" } }
                    }
                    div class="code" id="first-agent" data-pane-of="first" {
                        header { code { "terminal" } }
                        pre class="lines" { code class="src" { "ambolt mcp --server " (clone_url.split("/git/").next().unwrap_or("")) " --token $AGENT_TOKEN\n# the agent opens a task, then a change, from its own side" } }
                    }
                }
            } @else {
                @if !path.is_empty() {
                    div class="crumbs" { (breadcrumbs(name, path)) }
                }
                div class="panel files" {
                    @if entries.is_empty() && !path.is_empty() { div class="empty" { "Nothing here." } }
                    @for entry in entries {
                        @let target = if path.is_empty() { entry.name.clone() } else { format!("{path}/{}", entry.name) };
                        div class="row file" {
                            a class="t" href={ "/" (name) "/tree/" (target) } {
                                @if entry.is_dir { (ic("folder", "")) } @else { (ic("file", "")) }
                                (entry.name) @if entry.is_dir { "/" }
                            }
                            @if let Some(change) = &entry.change {
                                a class="s" href={ "/" (name) "/changes/" (change.number) } { "#" (change.number) " " (change.title) }
                                span class="age" { (people.name(&change.owner).0) }
                            } @else {
                                span class="s" { (entry.subject.as_deref().unwrap_or("")) }
                                span class="age" {}
                            }
                        }
                    }
                }
                @if let Some(readme) = readme {
                    div class="panel readme" {
                        header { (ic("file", "")) h2 { "README.md" } }
                        div class="prose" { (markdown(readme)) }
                    }
                }
            }
        },
        Some(rail),
    )
}

fn breadcrumbs(repo: &str, path: &str) -> Markup {
    let mut segments = Vec::new();
    let mut acc = String::new();
    for part in path.split('/') {
        if !acc.is_empty() {
            acc.push('/');
        }
        acc.push_str(part);
        segments.push((part.to_owned(), acc.clone()));
    }
    html! {
        a href={ "/" (repo) } { (repo) }
        @for (part, target) in &segments {
            " / "
            a href={ "/" (repo) "/tree/" (target) } { (part) }
        }
    }
}

/// README markdown with raw HTML stripped: content renders, markup
/// from the file never executes. Fenced code is highlighted like a file.
fn markdown(source: &str) -> Markup {
    use pulldown_cmark::{CodeBlockKind, Event as MdEvent, Parser, Tag, TagEnd, html::push_html};
    let mut events = Vec::new();
    let mut block: Option<(String, String)> = None;
    for event in Parser::new(source) {
        match event {
            MdEvent::Html(_) | MdEvent::InlineHtml(_) => {}
            MdEvent::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                block = Some((lang, String::new()));
            }
            MdEvent::Text(text) if block.is_some() => {
                if let Some((_, code)) = block.as_mut() {
                    code.push_str(&text);
                }
            }
            MdEvent::End(TagEnd::CodeBlock) => {
                if let Some((lang, code)) = block.take() {
                    events.push(MdEvent::Html(code_block(&lang, &code).into()));
                }
            }
            other => events.push(other),
        }
    }
    let mut out = String::new();
    push_html(&mut out, events.into_iter());
    PreEscaped(out)
}

/// A fenced block as the file it would be, by the word after the fence.
fn code_block(lang: &str, code: &str) -> String {
    let word = lang
        .split([' ', ',', '{'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let extension = match word.as_str() {
        "rust" => "rs",
        "shell" | "bash" | "sh" | "console" | "zsh" => "sh",
        "javascript" => "js",
        "typescript" => "ts",
        "python" => "py",
        "markdown" => "md",
        "yaml" | "yml" => "yaml",
        "" => "txt",
        other => other,
    };
    let mut coder = super::highlight::Coder::for_path(&format!("block.{extension}"), code.len());
    let mut html = String::from("<pre><code class=\"src\">");
    let mut lines = code.split('\n').collect::<Vec<_>>();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            html.push('\n');
        }
        html.push_str(&coder.line(line, &[]));
    }
    html.push_str("</code></pre>\n");
    html
}

pub fn file(
    theme: Theme,
    who: Reading<'_>,
    repo: &str,
    path: &str,
    text: &str,
    landed_by: Option<&Change>,
) -> Markup {
    let lines: Vec<&str> = text.split('\n').collect();
    // A trailing newline is a line terminator, not an empty last line.
    let lines = match lines.split_last() {
        Some((last, rest)) if last.is_empty() && !rest.is_empty() => rest,
        _ => &lines[..],
    };
    let binary = text.contains('\u{0}');
    let mut coder = super::highlight::Coder::for_path(path, text.len());
    let language = super::highlight::language(path);
    let plain = language.is_some() && !coder.highlights();
    layout_reading(
        theme,
        who,
        Some(repo),
        Some(Tab::Code),
        path,
        html! {
            div class="crumbs" { (breadcrumbs(repo, path)) }
            div class="code" {
                header {
                    (ic("file", ""))
                    code { (path) }
                    (kv(&[
                        ("Lines", html! { (lines.len()) }),
                        ("Language", html! { @match language { Some(l) => { (l) } None => { span class="none" { "plain" } } } }),
                        if plain { ("Shown", html! { "plain, over " (super::human_bytes(super::highlight::LIMIT as u64)) }) } else { ("", html! {}) },
                    ]))
                    @if let Some(change) = landed_by {
                        span class="sec3" {
                            "landed by "
                            a class="link" href={ "/" (repo) "/changes/" (change.number) } {
                                "#" (change.number) " " (change.title)
                            }
                        }
                    }
                    div class="right" {
                        a class="ghost sm" href={ "/" (repo) "/blame/" (path) } { (ic("review", "sm")) "Blame" }
                    }
                }
                @if binary {
                    p class="empty" { "Binary file — nothing to show." }
                } @else {
                    pre class="source" {
                        @for (index, line) in lines.iter().enumerate() {
                            div class="cline" id={ "L" (index + 1) } {
                                a class="no" href={ "#L" (index + 1) } { (index + 1) }
                                code class="src" { (PreEscaped(coder.line(line, &[]))) }
                            }
                        }
                    }
                }
            }
        },
    )
}

/// One file of a diff, rendered line by line: the new side and the old
/// side each walk their own grammar state, and a deleted line paired
/// with the added line that replaced it carries marks on the words that
/// differ.
fn diff_lines(file: &FileDiff) -> Vec<Vec<String>> {
    let mut old_side = super::highlight::Coder::for_path(&file.path, 0);
    let mut new_side = super::highlight::Coder::for_path(&file.path, 0);
    file.hunks
        .iter()
        .map(|hunk| {
            let lines = &hunk.lines;
            let mut marks: Vec<super::highlight::Marks> = vec![Vec::new(); lines.len()];
            let mut i = 0;
            while i < lines.len() {
                if lines[i].kind != LineKind::Del {
                    i += 1;
                    continue;
                }
                let dels = i + lines[i..]
                    .iter()
                    .take_while(|l| l.kind == LineKind::Del)
                    .count();
                let adds = dels
                    + lines[dels..]
                        .iter()
                        .take_while(|l| l.kind == LineKind::Add)
                        .count();
                for k in 0..(dels - i).min(adds - dels) {
                    let (old, new) =
                        super::highlight::marks(&lines[i + k].text, &lines[dels + k].text);
                    marks[i + k] = old;
                    marks[dels + k] = new;
                }
                i = adds.max(i + 1);
            }
            lines
                .iter()
                .zip(marks)
                .map(|(line, marks)| match line.kind {
                    LineKind::Del => old_side.line(&line.text, &marks),
                    LineKind::Add => new_side.line(&line.text, &marks),
                    LineKind::Context => {
                        old_side.line(&line.text, &[]);
                        new_side.line(&line.text, &marks)
                    }
                })
                .collect()
        })
        .collect()
}

/// "lines 12–40": where a hunk sits in the new file, which is the one a
/// reader has open.
fn hunk_range(hunk: &super::diff::Hunk) -> String {
    let new: Vec<i64> = hunk
        .lines
        .iter()
        .filter(|l| l.kind != LineKind::Del)
        .map(|l| l.number)
        .collect();
    match (new.first(), new.last()) {
        (Some(first), Some(last)) if first != last => format!("lines {first}–{last}"),
        (Some(first), _) => format!("line {first}"),
        _ => hunk.header.clone(),
    }
}

/// The list of a repository's changes: newest first, filtered by state,
/// a page at a time, each row saying who opened it, when, and when it
/// was last touched.
/// What a kind is called where a person reads it.
fn kind_words(kind: OriginKind) -> &'static str {
    match kind {
        OriginKind::Bug => "Bug",
        OriginKind::Request => "Request",
        OriginKind::Question => "Question",
    }
}

fn kind_chip(kind: OriginKind) -> Markup {
    let icon = match kind {
        OriginKind::Bug => "alert",
        OriginKind::Request => "sparkle",
        OriginKind::Question => "message",
    };
    html! { span class="chip" { (ic(icon, "")) (kind_words(kind)) } }
}

/// How a settlement reads, in the words somebody would use about it.
fn settlement_words(how: Settlement) -> &'static str {
    match how {
        Settlement::Answered => "Answered",
        Settlement::Fixed => "Already fixed",
        Settlement::Declined => "Not planned",
        Settlement::Withdrawn => "Withdrawn",
        Settlement::Duplicate => "Said before",
    }
}

/// The three doors, as the query parameter that opens each form.
fn new_form(repo: &str, kind: OriginKind) -> Markup {
    let bug = kind == OriginKind::Bug;
    html! {
        form class="panel report-new" id="new" method="post" action={ "/" (repo) "/community" } {
            input type="hidden" name="kind" value=(kind.as_str());
            div class="pref" {
                h3 { @match kind {
                    OriginKind::Bug => "Report a bug",
                    OriginKind::Request => "Ask for something",
                    OriginKind::Question => "Ask a question",
                } }
                div class="field" {
                    label for="title" { "In one line" }
                    input class="input" id="title" name="title" type="text" required maxlength="300"
                        placeholder=(match kind {
                            OriginKind::Bug => "What goes wrong",
                            OriginKind::Request => "What you cannot do",
                            OriginKind::Question => "What you want to know",
                        });
                }
                div class="field" {
                    label for="body" { @match kind {
                        OriginKind::Bug => "What happened",
                        OriginKind::Request => "What you were trying to do, and what you do instead today",
                        OriginKind::Question => "The question",
                    } }
                    textarea id="body" name="body" rows="5" required {}
                }
                @if bug {
                    div class="grid2 tight" {
                        div class="field" {
                            label for="version" { "Version" }
                            input class="input" id="version" name="version" type="text" placeholder="What you were running";
                        }
                        div class="field" {
                            label for="command" { "Command that shows it" }
                            input class="input" id="command" name="command" type="text" placeholder="Someone else can run this";
                        }
                        div class="field" {
                            label for="observed" { "Exactly what it printed" }
                            input class="input" id="observed" name="observed" type="text";
                        }
                        div class="field" {
                            label for="expected" { "What you expected instead" }
                            input class="input" id="expected" name="expected" type="text";
                        }
                    }
                    p class="hint" {
                        "A bug with a command gets re-run and reaches a maintainer with evidence. \
                         Without one it waits for somebody to add it — send it anyway if you have none."
                    }
                }
                div class="act" {
                    button class="btn" type="submit" { "Send" }
                    a class="btn2" href={ "/" (repo) "/community" } { "Cancel" }
                }
            }
        }
    }
}

/// Everything said about a repository that is not work yet.
#[allow(clippy::too_many_arguments)]
pub fn community(
    theme: Theme,
    who: Reading<'_>,
    repo: &str,
    reports: &[Origin],
    kind: Option<OriginKind>,
    open_only: bool,
    may_report: bool,
    new: Option<OriginKind>,
    people: &People,
    error: Option<&str>,
) -> Markup {
    let href = |kind: Option<OriginKind>, open_only: bool| {
        let mut parts = Vec::new();
        if let Some(kind) = kind {
            parts.push(format!("kind={}", kind.as_str()));
        }
        if !open_only {
            parts.push("state=all".to_owned());
        }
        match parts.is_empty() {
            true => format!("/{repo}/community"),
            false => format!("/{repo}/community?{}", parts.join("&")),
        }
    };
    layout_reading(
        theme,
        who,
        Some(repo),
        Some(Tab::Community),
        "Community",
        html! {
            div class="sec top" {
                div class="sh" {
                    h2 { "Community" }
                    span class="n" { (reports.len()) }
                    @if may_report {
                        span class="grow" {}
                        @for one in [OriginKind::Bug, OriginKind::Request, OriginKind::Question] {
                            a class="btn2 sm" href={ "/" (repo) "/community?new=" (one.as_str()) "#new" } {
                                (ic("plus", "")) (kind_words(one))
                            }
                        }
                    }
                }
                @if let Some(error) = error { p class="error" { (error) } }
                @if let Some(new) = new { (new_form(repo, new)) }
                div class="filters" {
                    a class=[kind.is_none().then_some("on")] href=(href(None, open_only)) { "All" }
                    @for one in [OriginKind::Bug, OriginKind::Request, OriginKind::Question] {
                        a class=[(kind == Some(one)).then_some("on")] href=(href(Some(one), open_only)) { (kind_words(one)) "s" }
                    }
                    span class="grow" {}
                    a class=[open_only.then_some("on")] href=(href(kind, true)) { "Open" }
                    a class=[(!open_only).then_some("on")] href=(href(kind, false)) { "Everything" }
                }
                div class="panel" {
                    @if reports.is_empty() {
                        div class="empty" {
                            b { "Nobody has said anything yet." }
                            @if may_report { "Report a bug, ask for something, or ask a question." }
                            @else { "This repository does not take reports from outside." }
                        }
                    }
                    @for report in reports {
                        @let (display, agent) = people.name(&report.by);
                        a class="row report" href={ "/" (repo) "/community/" (report.number) } {
                            span class="chips" {
                                (kind_chip(report.kind))
                                @match (&report.state, &report.settled) {
                                    (OriginState::Discarded, _) => { span class="chip" { (ic("x", "")) "Discarded" } }
                                    (_, Some(done)) => { span class="chip good" { (ic("check", "")) (settlement_words(done.how)) } }
                                    _ => {}
                                }
                            }
                            span class="tt" {
                                span class="t" { "#" (report.number) " " (report.title) }
                                @let facts = {
                                    let mut facts = vec![
                                        ("Filed by", html! { (display) }),
                                        ("On", html! { span title=(report.at) { (short_day(&report.at)) } }),
                                        ("Replies", html! { (report.replies.len()) }),
                                    ];
                                    // Only a bug has one, and a fact that
                                    // says nothing is worth no room.
                                    if report.kind == OriginKind::Bug {
                                        facts.push((
                                            "Reproduction",
                                            html! {
                                                @if report.repro.is_runnable() { "a command to run" }
                                                @else { span class="none" { "none yet" } }
                                            },
                                        ));
                                    }
                                    facts
                                };
                                (kv(&facts))
                            }
                            span class="avs" { (avatar(report.by.as_str(), display, agent, false)) }
                            span class="age" title=(report.at) { (ago(&report.at)) }
                        }
                    }
                }
            }
        },
    )
}

/// One report, with what was said on it and where it stands.
#[allow(clippy::too_many_arguments)]
pub fn community_report(
    theme: Theme,
    who: Reading<'_>,
    repo: &str,
    report: &Origin,
    may_reply: bool,
    may_answer: bool,
    people: &People,
    error: Option<&str>,
) -> Markup {
    let (display, agent) = people.name(&report.by);
    let discarded = report.state == OriginState::Discarded;
    let action = format!("/{repo}/community/{}", report.number);
    layout_reading(
        theme,
        who,
        Some(repo),
        Some(Tab::Community),
        &format!("#{} {}", report.number, report.title),
        html! {
            div class="sec top" {
                div class="sh" {
                    h2 { "#" (report.number) " " (report.title) }
                    (kind_chip(report.kind))
                    @match (&report.state, &report.settled) {
                        (OriginState::Discarded, _) => { span class="chip" { (ic("x", "")) "Discarded" } }
                        (_, Some(done)) => { span class="chip good" { (ic("check", "")) (settlement_words(done.how)) } }
                        _ => { span class="chip acc" { "Open" } }
                    }
                }
                @if let Some(error) = error { p class="error" { (error) } }
                div class="panel said" {
                    div class="thread" {
                        div class="h" {
                            (avatar(report.by.as_str(), display, agent, false))
                            b { (display) }
                            span class="when" title=(report.at) { (ago(&report.at)) }
                        }
                        p class="body" { (with_mentions(&report.body)) }
                    @if report.kind == OriginKind::Bug && !discarded {
                        div class="repro" {
                            @if report.repro.is_empty() {
                                p class="none" {
                                    "Nothing here can be re-run yet. The command that shows it is what \
                                     would change that — anybody can add one below."
                                }
                            } @else {
                                (kv(&[
                                    ("Version", html! { @match &report.repro.version {
                                        Some(v) => { (v) } None => { span class="none" { "not said" } } } }),
                                ]))
                                @if let Some(command) = &report.repro.command {
                                    div class="field" {
                                        span class="k" { "Command" }
                                        pre { code { (command) } }
                                    }
                                }
                                div class="grid2 tight" {
                                    @if let Some(observed) = &report.repro.observed {
                                        div class="field" { span class="k" { "What happened" } p { (observed) } }
                                    }
                                    @if let Some(expected) = &report.repro.expected {
                                        div class="field" { span class="k" { "What you expected" } p { (expected) } }
                                    }
                                }
                                @if !report.repro.is_runnable() {
                                    p class="none" { "No command yet, so nobody else can check this." }
                                }
                            }
                        }
                    }
                    @for reply in &report.replies {
                        @let (display, agent) = people.name(&reply.by);
                        div class="reply" {
                            (avatar(reply.by.as_str(), display, agent, false))
                            span {
                                b { (display) } span class="when" title=(reply.at) { (ago(&reply.at)) }
                                p { (with_mentions(&reply.body)) }
                            }
                        }
                    }
                    @if let Some(done) = &report.settled {
                        @let (by, _) = people.name(&done.by);
                        p class="closed" {
                            (settlement_words(done.how)) " by " (by)
                            @if let Some(of) = done.duplicate_of {
                                " — see " a href={ "/" (repo) "/community/" (of) } { "#" (of) }
                            }
                            @if !done.note.is_empty() { ": " (done.note) }
                        }
                    }
                    @if may_reply && !discarded && report.settled.is_none() {
                        div class="act" {
                            form method="post" action={ (action) "/reply" } {
                                input class="input sm" type="text" name="body" placeholder="Reply" aria-label="Reply" required autocomplete="off";
                                button class="btn2 sm" type="submit" { "Reply" }
                            }
                        }
                    }
                    }
                }
                @if may_answer && !discarded && report.settled.is_none() {
                    div class="answer" {
                        form class="panel" method="post" action={ (action) "/settle" } {
                            div class="pref" {
                                h3 { "Settle" }
                                div class="field" {
                                    label for="how" { "How" }
                                    select class="input" id="how" name="how" {
                                        option value="answered" { "Answered" }
                                        option value="fixed" { "Already fixed" }
                                        option value="declined" { "Not planned" }
                                        option value="duplicate" { "Said before" }
                                    }
                                }
                                div class="field" {
                                    label for="note" { "Why" }
                                    input class="input" id="note" name="note" type="text" required
                                        placeholder="The next person to ask reads this";
                                }
                                div class="field" {
                                    label for="duplicate_of" { "Repeats which report" }
                                    input class="input" id="duplicate_of" name="duplicate_of" type="number" min="1"
                                        placeholder="Only for “said before”";
                                }
                                div class="act" { button class="btn" type="submit" { "Settle" } }
                            }
                        }
                        form class="discard" method="post" action={ (action) "/discard" } {
                            span class="t" { "Discard" }
                            input class="input sm" name="reason" type="text" required
                                placeholder="Why this should never have arrived" aria-label="Reason";
                            button class="btn2 sm danger" type="submit" { "Discard" }
                            span class="hint" { "The number and the reason stay; the text and its replies do not." }
                        }
                    }
                }
            }
        },
    )
}

pub fn changes(
    theme: Theme,
    who: Reading<'_>,
    repo: &str,
    changes: &[Change],
    filter: Option<ChangeState>,
    older: Option<i64>,
    people: &People,
) -> Markup {
    let filter_href = |state: Option<ChangeState>| match state {
        Some(state) => format!("/{repo}/changes?state={}", state.as_str()),
        None => format!("/{repo}/changes"),
    };
    let state_words = |state: ChangeState| match state {
        ChangeState::Open => "Open",
        ChangeState::Merged => "Landed",
        ChangeState::Abandoned => "Abandoned",
    };
    layout_reading(
        theme,
        who,
        Some(repo),
        Some(Tab::Changes),
        "Changes",
        html! {
            div class="sec top" {
                div class="sh" { h2 { "Changes" } span class="n" { (changes.len()) @if older.is_some() { " shown" } } }
                div class="filters" {
                    a class=[filter.is_none().then_some("on")] href=(filter_href(None)) { "All" }
                    @for state in [ChangeState::Open, ChangeState::Merged, ChangeState::Abandoned] {
                        a class=[(filter == Some(state)).then_some("on")] href=(filter_href(Some(state))) { (state_words(state)) }
                    }
                }
                div class="panel" {
                    @if changes.is_empty() {
                        @match filter {
                            None => { div class="empty" { b { "No changes yet." } "Push to " code { "refs/for/main" } " to open one." } }
                            Some(state) => { div class="empty" { "No " (state_words(state).to_lowercase()) " changes." } }
                        }
                    }
                    @for change in changes {
                        @let (display, agent) = people.name(&change.owner);
                        a class="row need" href={ "/" (repo) "/changes/" (change.number) } {
                            @match change.state {
                                ChangeState::Open => { span class="chip acc" { (ic("changes", "")) "Open" } }
                                ChangeState::Merged => { span class="chip good" { (ic("check", "")) "Landed" } }
                                ChangeState::Abandoned => { span class="chip" { (ic("x", "")) @if change.discarded { "Discarded" } @else { "Abandoned" } } }
                            }
                            @if change.proposal { span class="chip" { "Proposal" } }
                            span class="tt" {
                                span class="t" { "#" (change.number) " " (change.title) }
                                (kv(&[
                                    ("By", html! { (display) }),
                                    ("Revision", html! { (change.latest_revision) }),
                                    ("Opened", html! { span title=(change.opened_at) { (short_day(&change.opened_at)) } }),
                                ]))
                            }
                            span class="avs" { (avatar(change.owner.as_str(), display, agent, false)) }
                            span class="age" title=(change.updated_at) { (ago(&change.updated_at)) }
                        }
                    }
                    @if let Some(before) = older {
                        div class="foot" {
                            a class="btn2 sm" href={ (filter_href(filter)) @if filter.is_some() { "&" } @else { "?" } "before=" (before) } { "Older changes" }
                        }
                    }
                }
            }
        },
    )
}

pub struct ChangePage<'a> {
    pub theme: Theme,
    pub who: Reading<'a>,
    pub repo: &'a str,
    pub change: &'a Change,
    pub task: Option<&'a Task>,
    pub revisions: &'a [Revision],
    pub shown: i64,
    pub files: &'a [FileDiff],
    pub claims: &'a [Claim],
    pub verifications: &'a [Verification],
    pub verdicts: &'a [Verdict],
    pub trace: &'a PolicyTrace,
    pub queued: bool,
    pub threads: &'a [Thread],
    pub composer: Option<ThreadAt>,
    /// The revision the shown one is compared with: an interdiff.
    pub compared: Option<i64>,
    pub error: Option<&'a str>,
    /// Everyone named on the page, by display name.
    pub people: &'a People,
}

/// Where a new thread is being composed, from `?at=`: `new:12:src/x.rs`
/// or `old:3:src/x.rs` for a line, `claim:<id>`, `verdict:<id>`, or
/// `change`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThreadAt {
    Change,
    Line { path: String, side: Side, line: i64 },
    Claim(String),
    Verdict(String),
}

impl ThreadAt {
    pub fn parse(raw: &str) -> Option<Self> {
        if raw == "change" {
            return Some(Self::Change);
        }
        let (head, rest) = raw.split_once(':')?;
        match head {
            "claim" if !rest.is_empty() => Some(Self::Claim(rest.to_owned())),
            "verdict" if !rest.is_empty() => Some(Self::Verdict(rest.to_owned())),
            "old" | "new" => {
                let (line, path) = rest.split_once(':')?;
                let line: i64 = line.parse().ok().filter(|l| *l >= 1)?;
                if path.is_empty() {
                    return None;
                }
                Some(Self::Line {
                    path: path.to_owned(),
                    side: if head == "old" { Side::Old } else { Side::New },
                    line,
                })
            }
            _ => None,
        }
    }

    fn words(&self) -> String {
        match self {
            Self::Change => "on the change".into(),
            Self::Line { path, line, .. } => format!("at {path}:{line}"),
            Self::Claim(_) => "on a claim".into(),
            Self::Verdict(_) => "on a review".into(),
        }
    }
}

pub fn change(page: ChangePage) -> Markup {
    let ChangePage {
        theme,
        who,
        repo,
        change,
        task,
        revisions,
        shown,
        files,
        claims,
        verifications,
        verdicts,
        trace,
        queued,
        threads,
        composer,
        compared,
        error,
        people,
    } = page;
    let message = revisions
        .iter()
        .find(|r| r.number == shown)
        .map(|r| r.message.as_str())
        .unwrap_or("");
    // The title line is the title; what follows, up to the trailers, is
    // the message worth reading.
    let body = message_body(message);
    let title = format!("#{} {}", change.number, change.title);
    let standing = threads
        .iter()
        .filter(|t| t.kind == ThreadKind::Concern && t.resolved.is_none())
        .count();
    // Threads sit under the line they are about, on the revision they
    // were raised on; other revisions list them in the Discussion panel.
    let mut inline: HashMap<(&str, &str, i64), Vec<&Thread>> = HashMap::new();
    for thread in threads.iter().filter(|t| t.revision == shown) {
        if let Anchor::Line { path, side, line } = &thread.anchor {
            inline
                .entry((path.as_str(), side.as_str(), *line))
                .or_default()
                .push(thread);
        }
    }
    let composer_line = match &composer {
        Some(ThreadAt::Line { path, side, line }) if compared.is_none() => {
            Some((path.as_str(), side.as_str(), *line))
        }
        _ => None,
    };
    if compared.is_some() {
        inline.clear();
    }
    let signed = who.viewer().is_some();
    let open = change.state == ChangeState::Open;
    let can_discuss = open && signed;
    let satisfied = trace.requirements.iter().filter(|r| r.satisfied).count();
    let total = trace.requirements.len();
    let base = format!("/{repo}/changes/{}", change.number);
    let unmet_words = || {
        let missing: Vec<String> = trace
            .requirements
            .iter()
            .filter(|r| !r.satisfied)
            .map(|r| requirement_words(&r.description))
            .collect();
        format!("Not ready: {}", missing.join("; "))
    };
    let authors: Vec<&Revision> = revisions
        .iter()
        .filter(|r| !r.by.as_str().is_empty())
        .collect();
    let rail = html! {
        @if open && signed {
            div class="panel readiness" {
                header {
                    h2 { @if trace.satisfied { "Ready to land" } @else { "Not ready to land" } }
                    span class="n" { (satisfied) " of " (total) }
                }
                div class="pad" {
                    div class="progress" {
                        @for _ in 0..satisfied { i class="on" {} }
                        @for _ in satisfied..total { i class="bad" {} }
                    }
                    div class="reqs" {
                        @for requirement in trace.requirements.iter().filter(|r| !r.satisfied) {
                            (requirement_row(requirement))
                        }
                        @for requirement in trace.requirements.iter().filter(|r| r.satisfied) {
                            (requirement_row(requirement))
                        }
                    }
                }
                div class="foot" {
                    @if queued {
                        form class="dequeue" method="post" action={ (base) "/dequeue" } {
                            span { "In the queue; it lands from here." }
                            input class="input sm" type="text" name="reason" placeholder="Why take it out (optional)" aria-label="Why take it out (optional)" autocomplete="off";
                            button class="btn2 sm" type="submit" { "Take it out" }
                        }
                    } @else {
                        form method="post" action={ (base) "/enqueue" } {
                            button class="btn wide" type="submit" disabled[!trace.satisfied] title=[(!trace.satisfied).then(unmet_words)] {
                                (ic("check", "sm")) "Land on " (change.target)
                            }
                        }
                    }
                }
            }
        }
        div class="panel" {
            header {
                h2 { "Claims" }
                span class="n" { "revision " (shown) }
                @if open && signed {
                    div class="right" {
                        button class="ghost sm" type="button" data-toggle="claimform" data-toggle-closed { (ic("plus", "sm")) "Add" }
                    }
                }
            }
            @if claims.is_empty() {
                div class="empty" { b { "None yet." } "Nobody has said what they checked on revision " (shown) "." }
            }
            @for claim in claims {
                (claim_row(claim, verifications, people))
                @if can_discuss {
                    a class="quiet discuss" href={ (base) "?r=" (shown) "&at=claim:" (claim.id.as_str()) "#at" } { "Discuss" }
                }
            }
            @if open && signed {
                form class="pad form" id="claimform" method="post" action={ (base) "/claim" } {
                    input type="hidden" name="revision" value=(shown);
                    select class="input sm" name="kind" aria-label="Kind" {
                        option value="test" { "Tests" }
                        option value="lint" { "Lint" }
                        option value="typecheck" { "Types" }
                        option value="build" { "Build" }
                        option value="manual" { "Looked at it myself" }
                        option value="reasoning" { "Reasoning" }
                    }
                    input class="input sm" type="text" name="command" placeholder="Command that produced it, so a runner can re-run it" aria-label="Command that produced it, so a runner can re-run it" autocomplete="off";
                    input class="input sm" type="text" name="summary" placeholder="What you saw" aria-label="What you saw" required;
                    input class="input sm" type="text" name="unchecked" placeholder="What this did not check, comma-separated" aria-label="What this did not check, comma-separated";
                    div class="line" {
                        button class="btn2 sm" type="submit" name="passed" value="yes" { "Passed" }
                        button class="btn2 sm" type="submit" name="passed" value="no" { "Failed" }
                    }
                }
            }
        }
        div class="panel" {
            header {
                h2 { "Reviews" }
                span class="n" { (verdicts.len()) }
            }
            @if verdicts.is_empty() {
                div class="empty" { b { "None yet." } "Nobody has reviewed revision " (shown) "." }
            }
            @for verdict in verdicts {
                (verdict_row(verdict, people))
                @if can_discuss {
                    a class="quiet discuss" href={ (base) "?r=" (shown) "&at=verdict:" (verdict.id.as_str()) "#at" } { "Discuss" }
                }
            }
        }
        div class="panel" {
            header {
                h2 { "Discussion" }
                span class="n" { (threads.iter().filter(|t| t.resolved.is_none()).count()) " open" }
                @if can_discuss {
                    div class="right" {
                        a class="ghost sm" href={ (base) "?r=" (shown) "&at=change#at" } { (ic("plus", "sm")) "Thread" }
                    }
                }
            }
            @if threads.is_empty() {
                div class="empty" { b { "Nothing yet." } "A line number in the diff starts a thread on that line." }
            }
            @for thread in threads {
                a class="ev-row" href={ (base) "?r=" (thread.revision) "#" (thread.id.as_str()) } {
                    (avatar(thread.by.as_str(), people.name(&thread.by).0, people.name(&thread.by).1, false))
                    div {
                        div class="h" {
                            b { (people.name(&thread.by).0) }
                            span class="sec2" {
                                (thread.kind.as_str())
                                @match &thread.anchor {
                                    Anchor::Change => " on the change",
                                    other => (anchor_words(other)),
                                }
                            }
                        }
                        div class="sub" {
                            (kv(&[
                                ("Revision", html! { (thread.revision) }),
                                ("Status", html! { (closure_words(thread)) }),
                                ("Replies", html! { (thread.replies.len()) }),
                            ]))
                        }
                    }
                }
            }
        }
    };
    layout_reading_with(
        theme,
        who,
        Some(repo),
        Some(Tab::Changes),
        &title,
        html! {
            @if let Some(error) = error {
                div class="notice bad" { (ic("alert", "")) span { (error) } }
            }
            div class="pagehead" {
                div {
                    h1 { (change.title) }
                    div class="meta" {
                        @match change.state {
                            ChangeState::Open => { span class="chip acc" { (ic("changes", "")) "Open" } }
                            ChangeState::Merged => { span class="chip good" { (ic("check", "")) "Landed" } }
                            ChangeState::Abandoned => { span class="chip" { (ic("x", "")) @if change.discarded { "Discarded" } @else { "Abandoned" } } }
                        }
                        @if change.proposal { span class="chip" title="Opened by someone who holds no push here" { "Proposal" } }
                        @if change.competing {
                            @match change.preferred_revision {
                                Some(preferred) => { span class="chip" { (ic("rerun", "")) "Revision " (preferred) " preferred" } }
                                None => { span class="chip" { (ic("rerun", "")) (authors.len()) " attempts" } }
                            }
                        }
                        @if queued { span class="chip acc" { (ic("clock", "")) "In the landing queue" } }
                        @if standing > 0 {
                            span class="chip bad stands" {
                                (ic("alert", ""))
                                (standing) @if standing == 1 { " concern stands" } @else { " concerns stand" }
                            }
                        }
                        span class="by" {
                            (avatar(change.owner.as_str(), people.name(&change.owner).0, people.name(&change.owner).1, false))
                            b { (people.name(&change.owner).0) }
                        }
                        span { "opened " span title=(change.opened_at) { (ago(&change.opened_at)) } }
                        span { "into " b { (change.target) } }
                        @if let Some(task) = task {
                            span { "for the task " a href={ "/tasks/" (task.id.as_str()) } { (task.title) } }
                        }
                        span { "updated " span title=(change.updated_at) { (ago(&change.updated_at)) } }
                        @if change.state == ChangeState::Merged {
                            a href={ "/api/changes/" (change.id) "/receipt" } { "Receipt" }
                        }
                    }
                }
                @if change.proposal && open {
                    div class="notice" {
                        (ic("alert", ""))
                        span {
                            "A proposal: " b { (people.name(&change.owner).0) } " holds no push here. Its claims are their word until a runner reproduces one"
                            @if change.admitted { "; runners have been let at it." } @else { ", and runners leave it alone until someone inside lets them at it." }
                        }
                        @if signed && !change.admitted {
                            form method="post" action={ (base) "/admit" } {
                                button class="btn2 sm" type="submit" title="Runners on a timer will pick it up from here" { "Let the runner at it" }
                            }
                        }
                    }
                }
                @if open && signed {
                    div class="acts" {
                        @if !queued {
                            form method="post" action={ (base) "/enqueue" } {
                                button class="btn sm" type="submit" disabled[!trace.satisfied] title=[(!trace.satisfied).then(unmet_words)] {
                                    (ic("check", "sm")) "Land on " (change.target)
                                }
                            }
                        }
                        details class="more" {
                            summary class="btn2 sm" aria-label="More" { (ic("more", "sm")) }
                            form class="pop" method="post" action={ (base) "/abandon" } {
                                div class="lab" { "Abandon this change" }
                                input class="input sm" type="text" name="reason" placeholder="Why, for whoever reads the log" aria-label="Why, for whoever reads the log" required autocomplete="off";
                                button class="danger" type="submit" { (ic("x", "sm")) "Abandon" }
                                @if change.proposal {
                                form class="pop" method="post" action={ (base) "/discard" } {
                                    div class="lab" { "Discard this proposal" }
                                    p class="hint" { "Abandoned, and its revisions taken out of git. For what should never have arrived; the reason stays in the log." }
                                    input class="input sm" type="text" name="reason" placeholder="Why, for whoever reads the log" aria-label="Why, for whoever reads the log" autocomplete="off" required;
                                    button class="danger" type="submit" { (ic("x", "sm")) "Discard" }
                                }
                            }
                        }
                        }
                    }
                }
            }
            div class="chg-body" {
                @if revisions.len() > 1 {
                    div class="seg revs" {
                        @for revision in revisions {
                            a class=[(revision.number == shown && compared.is_none()).then_some("on")]
                              href={ (base) "?r=" (revision.number) } {
                                "Revision " (revision.number)
                                @if !revision.by.as_str().is_empty() { span class="sec3" { (people.name(&revision.by).0) } }
                            }
                        }
                        @if shown > 1 {
                            @let previous = compared.unwrap_or(shown - 1);
                            a class=[compared.is_some().then_some("on")]
                              href={ (base) "?r=" (shown) "&vs=" (previous) }
                              title="What changed between the two revisions" {
                                "What changed " (previous) " → " (shown)
                            }
                        }
                    }
                }
                @if change.competing && open && authors.len() > 1 {
                    div class="tries" {
                        @for revision in &authors {
                            @let chosen = change.preferred_revision == Some(revision.number);
                            div class={ "try" @if chosen { " chosen" } } {
                                div class="h" {
                                    (avatar(revision.by.as_str(), people.name(&revision.by).0, people.name(&revision.by).1, false))
                                    b { "Revision " (revision.number) } span class="sec3" { "by " (people.name(&revision.by).0) }
                                    @if chosen { span class="chip acc" { (ic("check", "")) "chosen" } }
                                    @else if revision.number == change.latest_revision { span class="chip" { "latest" } }
                                }
                                @if let Some(summary) = message_body(&revision.message) {
                                    span class="s" { (summary.lines().next().unwrap_or("")) }
                                }
                                @if signed && change.preferred_revision.is_none() {
                                    form class="choose" method="post" action={ (base) "/prefer" } {
                                        input type="hidden" name="revision" value=(revision.number);
                                        input class="input sm" type="text" name="rationale" placeholder="Why this one and not the others" aria-label="Why this one and not the others" required;
                                        button class="btn2 sm" type="submit" { "Choose this attempt" }
                                    }
                                }
                            }
                        }
                    }
                }
                @if let Some(body) = body {
                    pre class="msg" { (body) }
                }
                @if let Some(vs) = compared {
                    div class="notice" { (ic("changes", "")) span { "Showing what changed from r" (vs) " to r" (shown) ", not the whole change. Threads sit on the full view of each revision." } }
                }
                (disagreement(verdicts, people))
                @if files.is_empty() {
                    div class="empty" { "No diff to show for this revision." }
                }
                @for file in files {
                    @let rendered = diff_lines(file);
                    @let adds = file.hunks.iter().flat_map(|h| &h.lines).filter(|l| l.kind == LineKind::Add).count();
                    @let dels = file.hunks.iter().flat_map(|h| &h.lines).filter(|l| l.kind == LineKind::Del).count();
                    div class="diff" {
                        header {
                            (ic("file", ""))
                            code { (file.path) }
                            span class="pm" {
                                span class="plus" { "+" (adds) }
                                " "
                                span class="minus" { "−" (dels) }
                            }
                            div class="right" {
                                a class="ghost sm" href={ "/" (repo) "/tree/" (file.path) } { (ic("code", "sm")) "File" }
                                a class="ghost sm" href={ "/" (repo) "/blame/" (file.path) } { (ic("review", "sm")) "Blame" }
                            }
                        }
                        @for (h, hunk) in file.hunks.iter().enumerate() {
                            div class="hunk" {
                                div class="hunk-head" { (hunk_range(hunk)) }
                                @for (i, line) in hunk.lines.iter().enumerate() {
                                    @let (class, sign) = match line.kind {
                                        LineKind::Add => ("ln add", "+"),
                                        LineKind::Del => ("ln del", "−"),
                                        LineKind::Context => ("ln ctx", ""),
                                    };
                                    @let side = if line.kind == LineKind::Del { "old" } else { "new" };
                                    @let (old_no, new_no) = match line.kind {
                                        LineKind::Add => (None, Some(line.number)),
                                        LineKind::Del => (Some(line.number), None),
                                        LineKind::Context => (line.old, Some(line.number)),
                                    };
                                    div class=(class) {
                                        span class="no" { @if let Some(n) = old_no { (n) } }
                                        @if can_discuss {
                                            a class="no" href={ (base) "?r=" (shown) "&at=" (side) ":" (line.number) ":" (query_path(&file.path)) "#at" } { @if let Some(n) = new_no { (n) } @else { (line.number) } }
                                        } @else {
                                            span class="no" { @if let Some(n) = new_no { (n) } }
                                        }
                                        span class="sign" { (sign) }
                                        code class="cd" { (PreEscaped(&rendered[h][i])) }
                                    }
                                    @if let Some(here) = inline.get(&(file.path.as_str(), side, line.number)) {
                                        @for thread in here {
                                            (thread_block(repo, change, shown, thread, people))
                                        }
                                    }
                                    @if composer_line == Some((file.path.as_str(), side, line.number)) {
                                        @if let Some(at) = &composer {
                                            (thread_composer(repo, change, shown, at))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                @let loose: Vec<&Thread> = threads
                    .iter()
                    .filter(|t| !(t.revision == shown && matches!(t.anchor, Anchor::Line { .. })))
                    .collect();
                @let composing_loose = matches!(&composer, Some(ThreadAt::Change | ThreadAt::Claim(_) | ThreadAt::Verdict(_)));
                @if !loose.is_empty() || (can_discuss && composing_loose) {
                    div class="loose" {
                        @for thread in &loose {
                            (thread_block(repo, change, shown, thread, people))
                        }
                        @if can_discuss && composing_loose {
                            @if let Some(at) = &composer {
                                (thread_composer(repo, change, shown, at))
                            }
                        }
                    }
                }
                @if open && signed {
                    form class="review" method="post" action={ (base) "/verdict" } {
                        input type="hidden" name="revision" value=(shown);
                        span class="lab" { "Your review" }
                        select class="input sm" name="domain" aria-label="Domain" {
                            option value="correctness" { "Correctness" }
                            option value="security" { "Security" }
                            option value="design" { "Design" }
                            option value="style" { "Style" }
                        }
                        input class="input sm" type="text" name="rationale" placeholder="Why, for the record" aria-label="Why, for the record" required;
                        button class="btn2 sm" type="submit" name="disposition" value="approve" { (ic("check", "sm")) "Approve" }
                        button class="btn2 sm" type="submit" name="disposition" value="concern" { (ic("alert", "sm")) "Concern" }
                        button class="btn2 sm danger" type="submit" name="disposition" value="block" { (ic("x", "sm")) "Block" }
                    }
                }
            }
        },
        Some(rail),
    )
}

/// One rule of the policy, as a sentence a person can act on, with the
/// evidence under it while it is unmet. The policy's own words stay on
/// the row as its title.
fn requirement_row(requirement: &ambolt_core::Requirement) -> Markup {
    html! {
        div class={ "req" @if requirement.satisfied { " met" } @else { " unmet" } } title=(requirement.description) {
            span class="st" { @if requirement.satisfied { (ic("check", "")) } @else { (ic("x", "")) } }
            div {
                b { (requirement_words(&requirement.description)) }
                @if !requirement.satisfied && !requirement.evidence.is_empty() {
                    div class="why" { (requirement.evidence) }
                }
            }
        }
    }
}

/// The policy describes its rules in the graph's terms; the page says
/// them the way a reviewer would.
fn requirement_words(description: &str) -> String {
    match description {
        "change has at least one revision" => "Has at least one revision".into(),
        "competing revisions have a comparison" => "One attempt is chosen".into(),
        "no concern raised in discussion is left unresolved" => "Every concern is resolved".into(),
        "latest revision carries a passing test claim" => {
            "Tests pass on the latest revision".into()
        }
        "no claim on the latest revision is disputed by a runner" => {
            "Runner agrees with every claim".into()
        }
        "no blocking verdict on the latest revision" => "Nobody has blocked it".into(),
        "a runner reproduced a claim on the latest revision" => {
            "A runner reproduced a claim".into()
        }
        "owner's earned trust" => "The owner has earned trust".into(),
        d if d.starts_with("a human has looked at this change") => {
            "A person has looked at it since it was picked for one".into()
        }
        d if d.starts_with("approved independently") => {
            "Someone other than the author approves it".into()
        }
        d if d.starts_with("approved for ") => {
            format!("Approved for {}", &d["approved for ".len()..])
        }
        d if d.ends_with("reproduced the same claim on the latest revision") => {
            let n = d.split(' ').next().unwrap_or("2");
            format!("{n} runners reproduced the same claim")
        }
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        }
    }
}

/// Who people are on this page: display names and whether each is an
/// agent, looked up once by the route.
#[derive(Default)]
pub struct People(pub HashMap<String, (String, bool)>);

impl People {
    /// The display name and agent flag of a principal, or its id when
    /// the page was not told.
    pub fn name<'a>(&'a self, id: &'a PrincipalId) -> (&'a str, bool) {
        match self.0.get(id.as_str()) {
            Some((display, agent)) => (display.as_str(), *agent),
            None => (id.as_str(), false),
        }
    }
}

/// "2 h ago", "yesterday", "5 Sep": how long since a moment, the way a
/// person says it. The exact time rides on the element's title.
fn ago(ts: &str) -> String {
    let Ok(then) = ts.parse::<jiff::Timestamp>() else {
        return short_day(ts);
    };
    let seconds = jiff::Timestamp::now().duration_since(then).as_secs();
    match seconds {
        s if s < 60 => "just now".into(),
        s if s < 3600 => format!("{} min ago", s / 60),
        s if s < 86_400 => format!("{} h ago", s / 3600),
        s if s < 172_800 => "yesterday".into(),
        s if s < 14 * 86_400 => format!("{} days ago", s / 86_400),
        _ => short_day(ts),
    }
}

/// Where reviewers reached opposite conclusions, put the positions
/// beside each other. This is the one place a human's judgment is
/// provably worth more than another review, so it gets the top of the
/// page rather than a line in a list.
fn disagreement(verdicts: &[Verdict], people: &People) -> Markup {
    let favour: Vec<&Verdict> = verdicts
        .iter()
        .filter(|v| v.disposition == Disposition::Approve)
        .collect();
    let against: Vec<&Verdict> = verdicts
        .iter()
        .filter(|v| v.disposition == Disposition::Block)
        .collect();
    let reserved: Vec<&Verdict> = verdicts
        .iter()
        .filter(|v| v.disposition == Disposition::Concern)
        .collect();
    // Only a genuine conflict qualifies: someone for, someone against.
    if favour.is_empty() || (against.is_empty() && reserved.is_empty()) {
        return html! {};
    }
    html! {
        section class="panel disagree" {
            header {
                h2 { "Reviewers disagree" }
                span class="n" { "your judgment decides this" }
            }
            div class="sides" {
                div class="side" {
                    span class="pos ok" { "In favour" }
                    @for verdict in &favour { (position(verdict, people)) }
                }
                div class="side" {
                    span class="pos bad" {
                        @if against.is_empty() { "Reserved" } @else { "Against" }
                    }
                    @for verdict in against.iter().chain(reserved.iter()) { (position(verdict, people)) }
                }
            }
        }
    }
}

fn position(verdict: &Verdict, people: &People) -> Markup {
    let (display, agent) = people.name(&verdict.by);
    html! {
        div class="stance" {
            div class="who-line" {
                (avatar(verdict.by.as_str(), display, agent, false))
                span class="nm" { (display) }
                span class="sec3" { (verdict.domain.as_str()) }
            }
            q { (verdict.rationale) }
        }
    }
}

/// A path inside a query string: `/` and `:` are legal there and worth
/// keeping readable; anything that could be mistaken for syntax is not.
fn query_path(path: &str) -> String {
    path.bytes()
        .map(|b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                (b as char).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

/// What became of a thread, in a few words.
fn closure_words(thread: &Thread) -> String {
    match &thread.resolved {
        None => match thread.kind {
            ThreadKind::Concern => "stands".into(),
            ThreadKind::Question => "open".into(),
            ThreadKind::Note => "noted".into(),
        },
        Some(done) => match done.how {
            Resolution::Answered => format!("answered by {}", done.by),
            Resolution::Fixed => format!(
                "fixed in revision {} by {}",
                done.revision.unwrap_or_default(),
                done.by
            ),
            Resolution::Withdrawn => "withdrawn".into(),
            Resolution::Overruled => format!("overruled by {}", done.by),
        },
    }
}

/// One thread under the line it is about. Open threads show everything
/// and take replies; resolved ones fold to a line that says how they
/// closed, with the whole exchange a click away.
fn thread_block(
    repo: &str,
    change: &Change,
    shown: i64,
    thread: &Thread,
    people: &People,
) -> Markup {
    let verb = match thread.kind {
        ThreadKind::Question => "asked",
        ThreadKind::Concern => "raised a concern",
        ThreadKind::Note => "noted",
    };
    let id = thread.id.as_str();
    let (display, agent) = people.name(&thread.by);
    let head = html! {
        (avatar(thread.by.as_str(), display, agent, false))
        b { (display) }
        span class="sec2" {
            (verb) " on revision " (thread.revision)
            @if let Anchor::Line { path, line, .. } = &thread.anchor { " at " code { (path) ":" (line) } }
        }
        span class="when" title=(thread.at) { (ago(&thread.at)) }
    };
    let exchange = html! {
        p class="body" { (with_mentions(&thread.body)) }
        @for reply in &thread.replies {
            @let (display, agent) = people.name(&reply.by);
            div class="reply" {
                (avatar(reply.by.as_str(), display, agent, false))
                span {
                    b { (display) } span class="when" title=(reply.at) { (ago(&reply.at)) }
                    p { (with_mentions(&reply.body)) }
                }
            }
        }
    };
    match &thread.resolved {
        None => html! {
            div class="thread" id=(id) {
                div class="h" { (head) }
                (exchange)
                @if change.state == ChangeState::Open {
                    div class="act" {
                        form method="post" action={ "/" (repo) "/changes/" (change.number) "/threads/" (thread.id.as_str()) "/reply" } {
                            input type="hidden" name="revision" value=(shown);
                            input class="input sm" type="text" name="body" placeholder="Reply" aria-label="Reply" required autocomplete="off";
                            button class="btn2 sm" type="submit" { "Reply" }
                        }
                        form class="resolve" method="post" action={ "/" (repo) "/changes/" (change.number) "/threads/" (thread.id.as_str()) "/resolve" } {
                            input type="hidden" name="revision" value=(shown);
                            select class="input sm" name="how" aria-label="Resolve as" {
                                option value="answered" { "Answered" }
                                @for fixed in (thread.revision + 1)..=change.latest_revision {
                                    option value={ "fixed:" (fixed) } { "Fixed in revision " (fixed) }
                                }
                                option value="withdrawn" { "Withdrawn" }
                                option value="overruled" { "Overruled" }
                            }
                            input class="input sm" type="text" name="note" placeholder="Why (optional)" aria-label="Why (optional)" autocomplete="off";
                            button class="btn2 sm" type="submit" { "Resolve" }
                        }
                    }
                }
            }
        },
        Some(done) => html! {
            details class="thread folded" id=(id) {
                summary {
                    span class="h" { (head) }
                    span class="chip" { (closure_words(thread)) }
                }
                (exchange)
                p class="closed" {
                    (closure_words(thread))
                    @if !done.note.is_empty() { ": " (done.note) }
                }
            }
        },
    }
}

/// The form a line number opens beneath itself, or the Discussion panel
/// opens for a claim, a verdict or the change. No script: `?at=` says
/// where, and the page renders the form there.
fn thread_composer(repo: &str, change: &Change, shown: i64, at: &ThreadAt) -> Markup {
    html! {
        form class="thread new" id="at" method="post" action={ "/" (repo) "/changes/" (change.number) "/threads" } {
            input type="hidden" name="revision" value=(shown);
            @match at {
                ThreadAt::Change => { input type="hidden" name="on" value="change"; }
                ThreadAt::Line { path, side, line } => {
                    input type="hidden" name="on" value="line";
                    input type="hidden" name="path" value=(path);
                    input type="hidden" name="side" value=(side.as_str());
                    input type="hidden" name="line" value=(line);
                }
                ThreadAt::Claim(claim) => {
                    input type="hidden" name="on" value="claim";
                    input type="hidden" name="claim" value=(claim);
                }
                ThreadAt::Verdict(verdict) => {
                    input type="hidden" name="on" value="verdict";
                    input type="hidden" name="verdict" value=(verdict);
                }
            }
            div class="at" { "New thread " (at.words()) ", revision " (shown) }
            div class="act" {
                select class="input sm" name="kind" aria-label="Kind" {
                    option value="question" { "Question" }
                    option value="concern" { "Concern" }
                    option value="note" { "Note" }
                }
                input class="input sm" type="text" name="body" placeholder="What do you want to say?" aria-label="What do you want to say?" required autofocus autocomplete="off";
                button class="btn2 sm" type="submit" { "Open" }
                a class="ghost sm" href={ "/" (repo) "/changes/" (change.number) "?r=" (shown) } { "Cancel" }
            }
        }
    }
}

/// What a claim says it did, in the page's words.
fn claim_kind_words(kind: ambolt_core::ClaimKind, passed: bool) -> &'static str {
    match (kind.as_str(), passed) {
        ("test", true) => "Tests pass",
        ("test", false) => "Tests fail",
        ("lint", true) => "Lint clean",
        ("lint", false) => "Lint fails",
        ("typecheck", true) => "Types check",
        ("typecheck", false) => "Types fail",
        ("build", true) => "Builds",
        ("build", false) => "Does not build",
        ("manual", _) => "Looked at it",
        ("reasoning", _) => "Reasoning",
        (_, true) => "Passed",
        (_, false) => "Failed",
    }
}

fn claim_row(claim: &Claim, verifications: &[Verification], people: &People) -> Markup {
    let runs: Vec<&Verification> = verifications
        .iter()
        .filter(|v| v.claim == claim.id)
        .collect();
    let disputed = runs.iter().any(|v| !v.agrees);
    let (display, agent) = people.name(&claim.by);
    let executed = claim.command.is_some();
    html! {
        div class="ev-row" {
            (avatar(claim.by.as_str(), display, agent, false))
            div {
                div class="h" {
                    b { (claim_kind_words(claim.kind, claim.passed)) }
                    @if disputed { span class="chip bad" { (ic("alert", "")) "disputed" } }
                    @else if !executed { span class="sec3" { "not a check" } }
                    @else if claim.passed { span class="chip good" { (ic("check", "")) "passed" } }
                    @else { span class="chip bad" { (ic("x", "")) "failed" } }
                    span class="sec3" { (display) }
                }
                @if let Some(command) = &claim.command {
                    div class="cmd" { (command) }
                }
                q { (claim.summary) }
                @for run in &runs {
                    @let (runner, _) = people.name(&run.by);
                    div class={ "sub" @if run.agrees { " good" } @else { " bad" } } {
                        (ic("rerun", "sm"))
                        (runner)
                        @if run.agrees { " re-ran it and saw the same: " } @else { " re-ran it and saw something else: " }
                        (run.observed)
                    }
                }
                @if runs.is_empty() && executed {
                    div class="sub" { "Not re-run by anyone yet." }
                }
                @if !executed {
                    div class="sub" { "An argument, not a check. Nothing to re-run." }
                }
                @for unchecked in &claim.unchecked {
                    div class="sub" { "Not checked: " (unchecked) }
                }
            }
        }
    }
}

fn verdict_row(verdict: &Verdict, people: &People) -> Markup {
    let (display, agent) = people.name(&verdict.by);
    html! {
        div class="ev-row" {
            (avatar(verdict.by.as_str(), display, agent, false))
            div {
                div class="h" {
                    b { (display) }
                    @match verdict.disposition {
                        Disposition::Approve => { span class="chip good" { (ic("check", "")) "approves" } }
                        Disposition::Concern => { span class="chip" { (ic("alert", "")) "concern" } }
                        Disposition::Block => { span class="chip bad" { (ic("x", "")) "blocks" } }
                    }
                    span class="sec3" { (verdict.domain.as_str()) @if agent { " · agent" } }
                }
                q { (verdict.rationale) }
            }
        }
    }
}

pub fn landing(
    theme: Theme,
    who: Reading<'_>,
    repo: &str,
    branch: &str,
    data: &LandingData,
) -> Markup {
    let numbers: Refs = data
        .numbers
        .iter()
        .map(|(id, (number, title))| (id.as_str(), (*number, title.as_str())))
        .collect();
    let people = &data.people;
    let brief = &data.brief;
    let latest_landed = data.outcomes.iter().find_map(|e| match &e.event {
        Event::ChangeMerged { change, .. } => Some(change.as_str()),
        _ => None,
    });
    let rail = html! {
        div class="panel" {
            header { h2 { "At work here" } span class="n" { (data.sessions.len()) } }
            @if data.sessions.is_empty() { div class="empty" { "Nobody is working here right now." } }
            @for session in &data.sessions {
                @let (display, agent) = people.name(&session.agent);
                @let paths: Vec<&str> = data.leases.iter().filter(|l| l.session == session.id).flat_map(|l| l.paths.iter().map(String::as_str)).collect();
                @let shared = data.leases.iter().any(|l| l.session != session.id && l.paths.iter().any(|p| paths.iter().any(|q| p.starts_with(q) || q.starts_with(p.as_str()))));
                div class="ev-row" {
                    (avatar(session.agent.as_str(), display, agent, true))
                    div {
                        div class="h" {
                            b { (display) }
                            @if let Some(title) = data.tasks.get(session.task.as_str()) { span class="sec2" { "on " (title) } }
                        }
                        @if !paths.is_empty() { span class="cmd" { (paths.join(", ")) } }
                        @if shared { div class="sub bad" { (ic("alert", "sm")) "overlaps another's path" } }
                    }
                }
            }
        }
        div class="panel" {
            header { h2 { "Just now" } }
            @if data.live.is_empty() { div class="empty" { "Nothing has happened here yet." } }
            @for envelope in &data.live {
                (event_row(&numbers, envelope, people))
            }
        }
    };
    layout_reading_with(
        theme,
        who,
        Some(repo),
        Some(Tab::Review),
        "Review",
        html! {
            div class="stats" {
                div class="stat" {
                    span class="k" { (ic("review", "sm")) "Needs you" }
                    span class="v" { (data.needs_you.len()) }
                    span class="d" { "ranked by what your judgment is worth" }
                }
                div class="stat" {
                    span class="k" { (ic("check", "sm")) "Landed lately" }
                    span class="v" { (brief.landed) }
                    span class="d" {
                        @match latest_landed {
                            Some(id) => { (change_ref(&numbers, id)) }
                            None => { "nothing has landed in this window" }
                        }
                        " " a class="quiet" href={ "/" (repo) "/activity?after=" (brief.since) } { "(counted from the log)" }
                    }
                }
                div class="stat" {
                    span class="k" { (ic("alert", "sm")) "Disputed claims" }
                    span class={ "v" @if brief.disputed > 0 { " bad-t" } } { (brief.disputed) }
                    span class="d" {
                        @if brief.disputed == 0 { "runners agreed with every claim they re-ran" }
                        @else if brief.disputed == 1 { "a runner saw something else" }
                        @else { "runners saw something else" }
                    }
                }
                div class="stat" {
                    span class="k" { (ic("agents", "sm")) "At work here" }
                    span class="v" { (data.sessions.len()) }
                    span class="d" {
                        @if data.sessions.is_empty() { "no session is open on this repository" }
                        @else { (names(data.sessions.iter().map(|s| people.name(&s.agent).0))) }
                    }
                }
            }

            div class="sec" {
                div class="sh" { h2 { "Needs you" } span class="n" { (data.needs_you.len()) } }
                div class="panel" {
                    @if data.needs_you.is_empty() {
                        div class="empty" { b { "Nothing needs you right now." } "Changes that want a person's judgment appear here, ranked by what that judgment is worth." }
                    }
                    @for item in &data.needs_you {
                        @let (chip, label) = attention_chip(item);
                        @let (display, agent) = people.name(&item.change.owner);
                        a class="row need" href={ "/" (repo) "/changes/" (item.change.number) } title=(attention_evidence(item)) {
                            span class=(chip) { (label) }
                            span class="tt" {
                                span class="t" { "#" (item.change.number) " " (item.change.title) }
                                span class="s tagline" {
                                    @if let Some(draw) = &item.drawn { span class="tag" { "picked " (draw.day) } }
                                    @for signal in item.signals.iter().filter(|s| s.kind != ambolt_core::SignalKind::Drawn) {
                                        span class="tag" { (signal.description) }
                                    }
                                }
                            }
                            span class="avs" { (avatar(item.change.owner.as_str(), display, agent, false)) }
                            span class="age" title=(item.change.updated_at) { (ago(&item.change.updated_at)) }
                        }
                    }
                }
            }

            div class="sec" {
                div class="sh" { h2 { "Landing on " (branch) } span class="n" { @if brief.landed > 0 { (brief.landed) " lately" } } }
                div class="panel" {
                    @for (index, entry) in data.queue.iter().enumerate() {
                        @let (display, agent) = people.name(&entry.enqueued_by);
                        div class="row need" {
                            span class={ "chip" @if index == 0 { " acc" } } { (ic("clock", "")) @if index == 0 { "landing" } @else { "queued" } }
                            span class="tt" {
                                span class="t" { (change_ref(&numbers, entry.change.as_str())) }
                                span class="s" { "sent by " (display) }
                            }
                            span class="avs" { (avatar(entry.enqueued_by.as_str(), display, agent, false)) }
                            span class="age" { (index + 1) }
                        }
                    }
                    @for outcome in &data.outcomes {
                        (outcome_row(&numbers, outcome, people))
                    }
                    @if data.queue.is_empty() {
                        div class="foot" { (ic("clock", "sm")) "Nothing waiting. Landing checks the rules once more and writes the result into the merge." }
                    }
                }
            }

            @if !brief.failed_sessions.is_empty() {
                div class="sec" {
                    div class="sh" {
                        h2 { "Stopped, with a lesson" }
                        span class="n" { (brief.failed_sessions.len()) }
                        div class="right" { a href={ "/" (repo) "/lessons" } { "All lessons" } }
                    }
                    div class="panel" {
                        @for lesson in &brief.failed_sessions {
                            @let (display, agent) = people.name(&lesson.agent);
                            div class="row ls" {
                                (avatar(lesson.agent.as_str(), display, agent, false))
                                span class="tt" {
                                    span class="t" { (lesson.task_title) }
                                    span class="s wrap" { (lesson.outcome) }
                                }
                                span class="age" { (display) }
                            }
                        }
                    }
                }
            }
        },
        Some(rail),
    )
}

type Refs<'a> = HashMap<&'a str, (i64, &'a str)>;

/// A change referred to the way a person would: number and title.
fn change_ref(numbers: &Refs, id: &str) -> Markup {
    match numbers.get(id) {
        Some((number, title)) => html! { "#" (number) " " (title) },
        None => html! { code { (short(id)) } },
    }
}

/// Everything behind a ranking, for the reader who wants the facts.
fn attention_evidence(item: &ambolt_core::AttentionItem) -> String {
    item.signals
        .iter()
        .map(|s| format!("{}: {}", s.description, s.evidence))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A compact reference for narrow columns: number and title, elided
/// by the layout rather than truncated here.
fn change_short(numbers: &HashMap<String, (i64, String)>, id: &str) -> Markup {
    match numbers.get(id) {
        Some((number, title)) => html! { "#" (number) " " (title) },
        None => html! { code { (short(id)) } },
    }
}

/// Just the number, where the title would repeat what sits beside it.
fn change_num(numbers: &Refs, id: &str) -> Markup {
    match numbers.get(id) {
        Some((number, _)) => html! { "#" (number) },
        None => html! { code { (short(id)) } },
    }
}

fn outcome_row(numbers: &Refs, envelope: &Envelope, people: &People) -> Markup {
    let (display, agent) = people.name(&envelope.actor);
    match &envelope.event {
        Event::ChangeMerged {
            change, merged_as, ..
        } => html! {
            div class="row need" {
                span class="chip good" { (ic("check", "")) "landed" }
                span class="tt" {
                    span class="t" { (change_ref(numbers, change.as_str())) }
                    span class="s" {
                        @if let Some(oid) = merged_as { "as " code { (short(oid)) } " · rebased · " }
                        "receipt signed"
                    }
                }
                span class="avs" { (avatar(envelope.actor.as_str(), display, agent, false)) }
                span class="age" title=(envelope.ts) { (ago(&envelope.ts)) }
            }
        },
        Event::ChangeDequeued { change, reason } => html! {
            div class="row need" {
                span class="chip bad" { (ic("x", "")) "left the queue" }
                span class="tt" {
                    span class="t" { (change_ref(numbers, change.as_str())) }
                    span class="s" { (reason) }
                }
                span class="avs" { (avatar(envelope.actor.as_str(), display, agent, false)) }
                span class="age" title=(envelope.ts) { (ago(&envelope.ts)) }
            }
        },
        _ => html! {},
    }
}

fn event_row(numbers: &Refs, envelope: &Envelope, people: &People) -> Markup {
    let (_, text) = describe(numbers, envelope, people);
    let (display, agent) = people.name(&envelope.actor);
    html! {
        div class="ev-row" {
            (avatar(envelope.actor.as_str(), display, agent, false))
            div {
                div class="h said" { (text) }
                div class="sub" title=(envelope.ts) { (ago(&envelope.ts)) }
            }
        }
    }
}

/// Where a thread sits, in the words a reader would use.
fn anchor_words(anchor: &Anchor) -> String {
    match anchor {
        Anchor::Change => String::new(),
        Anchor::Line { path, line, .. } => format!(" at {path}:{line}"),
        Anchor::Claim { .. } => " on a claim".into(),
        Anchor::Verdict { .. } => " on a review".into(),
    }
}

fn describe(numbers: &Refs, envelope: &Envelope, people: &People) -> (&'static str, Markup) {
    let actor = people.name(&envelope.actor).0;
    match &envelope.event {
        Event::RevisionPreferred {
            change,
            revision,
            over,
            ..
        } => (
            "dot ok",
            html! {
                b { (actor) } " preferred revision " (revision) " of " (change_num(numbers, change.as_str()))
                @if !over.is_empty() {
                    " over " @for (index, other) in over.iter().enumerate() { @if index > 0 { ", " } "revision " (other) }
                }
            },
        ),
        Event::ChangeMerged {
            change, merged_as, ..
        } => (
            "dot ok",
            html! {
                b { (change_num(numbers, change.as_str())) " landed" }
                @if let Some(oid) = merged_as { " as " code { (short(oid)) } }
            },
        ),
        Event::ChangeDequeued { change, .. } => (
            "dot bad",
            html! {
                b { (change_num(numbers, change.as_str())) } " left the queue"
            },
        ),
        Event::ChangeEnqueued { change } => (
            "dot idle",
            html! {
                b { (change_num(numbers, change.as_str())) } " entered the queue"
            },
        ),
        Event::ChangeAdmitted { change } => (
            "dot idle",
            html! {
                b { (actor) } " let runners at " (change_num(numbers, change.as_str()))
            },
        ),
        Event::ChangeDiscarded { change, reason } => (
            "dot bad",
            html! {
                b { (actor) } " discarded " (change_num(numbers, change.as_str())) ": " (reason)
            },
        ),
        Event::RevisionPushed {
            change, revision, ..
        } => (
            "dot idle",
            html! {
                b { (actor) } " pushed revision " (revision) " of " (change_num(numbers, change.as_str()))
            },
        ),
        Event::VerdictGiven {
            change,
            disposition,
            ..
        } => (
            match disposition {
                Disposition::Block => "dot bad",
                Disposition::Concern => "dot idle",
                Disposition::Approve => "dot ok",
            },
            html! {
                b { (actor) } " "
                @match disposition {
                    Disposition::Approve => { "approved " }
                    Disposition::Concern => { "raised a concern on " }
                    Disposition::Block => { "blocked " }
                }
                (change_num(numbers, change.as_str()))
            },
        ),
        Event::ClaimAttached {
            change,
            claim_kind,
            passed,
            ..
        } => (
            if *passed { "dot ok" } else { "dot bad" },
            html! {
                b { (actor) } " recorded a " (claim_kind.as_str()) " check on "
                (change_num(numbers, change.as_str()))
            },
        ),
        Event::ChangeOpened { number, title, .. } => (
            "dot idle",
            html! {
                b { (actor) } " opened #" (number) " " (title)
            },
        ),
        Event::TaskCreated { title, .. } => (
            "dot idle",
            html! { b { (actor) } " filed a task: " (title) },
        ),
        Event::TaskClaimed { .. } => ("dot idle", html! { b { (actor) } " claimed a task" }),
        Event::SessionOpened { .. } => ("dot idle", html! { b { (actor) } " started a session" }),
        Event::PathsDeclared { paths, .. } => (
            "dot idle",
            html! {
                b { (actor) } " is working on " (paths.join(", "))
            },
        ),
        Event::SessionEnded { state, .. } => (
            "dot idle",
            html! {
                b { (actor) } " ended a session · " (state.as_str())
            },
        ),
        Event::GrantIssued { grantee, .. } => (
            "dot idle",
            html! {
                b { (actor) } " granted " (people.name(grantee).0)
            },
        ),
        Event::GrantRevoked { .. } => ("dot idle", html! { b { (actor) } " revoked a grant" }),
        Event::RepoCreated { repo, .. } => ("dot idle", html! { b { (actor) } " created " (repo) }),
        Event::PasswordSet { principal, .. } => (
            "dot idle",
            html! { b { (actor) } " set the password for " (people.name(principal).0) },
        ),
        Event::HistoryImported {
            branch,
            commits,
            source,
            ..
        } => (
            "dot idle",
            html! {
                b { (actor) } " imported " (commits) " commits onto " (branch)
                " from " (source) ", unreviewed here"
            },
        ),
        Event::VisibilitySet { repo, visibility } => (
            "dot idle",
            html! {
                b { (actor) } " made " (repo) " " (visibility.as_str())
            },
        ),
        Event::RepoTransferOffered { repo, to } => (
            "dot idle",
            html! {
                b { (actor) } " offered " (repo) " to " (people.name(to).0)
            },
        ),
        Event::RepoTransferAccepted { repo } => (
            "dot ok",
            html! {
                b { (actor) } " now owns " (repo)
            },
        ),
        Event::RepoTransferDeclined { repo } => (
            "dot idle",
            html! {
                b { (actor) } " declined ownership of " (repo)
            },
        ),
        Event::TeamMemberAdded { team, member } => (
            "dot idle",
            html! {
                b { (actor) } " added " (people.name(member).0) " to " (team.as_str())
            },
        ),
        Event::PasswordResetRequested { principal } => (
            "dot idle",
            html! {
                b { (people.name(principal).0) } " asked for a new sign-in link"
            },
        ),
        Event::OriginOpened {
            repo,
            number,
            origin_kind,
            title,
            ..
        } => (
            "dot idle",
            html! {
                b { (actor) } " filed a " (origin_kind.as_str()) " on " (repo) ": "
                a href={ "/" (repo) "/community/" (number) } { "#" (number) " " (title) }
            },
        ),
        Event::OriginReplied { .. } => ("dot idle", html! { b { (actor) } " replied on a report" }),
        Event::OriginSettled { how, .. } => (
            "dot ok",
            html! { b { (actor) } " settled a report: " (how.as_str()) },
        ),
        Event::OriginDiscarded { reason, .. } => (
            "dot bad",
            html! { b { (actor) } " discarded a report: " (reason) },
        ),
        Event::ThreadOpened {
            change,
            thread_kind,
            anchor,
            ..
        } => (
            "dot idle",
            html! {
                b { (actor) } " raised a " (thread_kind.as_str()) " on "
                (change_num(numbers, change.as_str())) (anchor_words(anchor))
            },
        ),
        Event::AttentionDrawn {
            change,
            signals,
            reviewers,
            ..
        } => (
            "dot idle",
            html! {
                "the rules picked " (change_num(numbers, change.as_str())) " for a human look"
                @if !signals.is_empty() {
                    ": " (signals.iter().map(|s| s.as_str().replace('_', " ")).collect::<Vec<_>>().join(", "))
                }
                @if !reviewers.is_empty() {
                    " · asked " (reviewers.iter().map(|r| people.name(r).0).collect::<Vec<_>>().join(", "))
                }
            },
        ),
        Event::SessionCredentialMinted {
            session,
            until,
            scope,
            ..
        } => (
            "dot idle",
            html! {
                b { (actor) } " drew a credential from session " code { (short(session.as_str())) }
                " for " (scope.describe()) ", until " (day_of(until)) " " (clock_of(until))
            },
        ),
        Event::SessionCredentialsRevoked { session, revoked } => (
            "dot idle",
            html! {
                "session " code { (short(session.as_str())) } " ended; "
                (revoked) @if *revoked == 1 { " credential died with it" } @else { " credentials died with it" }
            },
        ),
        Event::IdentityLinked {
            principal, issuer, ..
        } => (
            "dot idle",
            html! { b { (people.name(principal).0) } " linked an identity at " (issuer) },
        ),
        Event::IdentityUnlinked {
            principal, issuer, ..
        } => (
            "dot idle",
            html! { b { (people.name(principal).0) } " unlinked an identity at " (issuer) },
        ),
        Event::WorkloadBound {
            principal,
            issuer,
            subject,
        } => (
            "dot idle",
            html! { b { (actor) } " bound workload " (subject) " at " (issuer) " to " (people.name(principal).0) },
        ),
        Event::WorkloadUnbound {
            principal,
            issuer,
            subject,
        } => (
            "dot idle",
            html! { b { (actor) } " unbound workload " (subject) " at " (issuer) " from " (people.name(principal).0) },
        ),
        Event::WorkloadCredentialMinted { issuer, until, .. } => (
            "dot idle",
            html! { b { (actor) } " proved itself to " (issuer) " and drew a credential to claim a task, until " (day_of(until)) " " (clock_of(until)) },
        ),
        Event::PrincipalDeactivated { principal } => (
            "dot bad",
            html! { b { (actor) } " deactivated " (people.name(principal).0) },
        ),
        Event::PrincipalReactivated { principal } => (
            "dot ok",
            html! { b { (actor) } " reactivated " (people.name(principal).0) },
        ),
        Event::RepoRenamed { repo, to } => (
            "dot idle",
            html! { b { (actor) } " renamed " (repo) " to " (to) },
        ),
        Event::RepoDescribed { repo, description } => (
            "dot idle",
            html! { b { (actor) } " described " (repo) @if description.is_empty() { " as nothing in particular" } @else { ": " (description) } },
        ),
        Event::RepoWatched { repo } => (
            "dot idle",
            html! { b { (actor) } " started watching " (repo) },
        ),
        Event::RepoUnwatched { repo } => (
            "dot idle",
            html! { b { (actor) } " stopped watching " (repo) },
        ),
        Event::RepoTopicsSet { repo, topics } => (
            "dot idle",
            html! { b { (actor) } " filed " (repo) @if topics.is_empty() { " under nothing" } @else { " under " (topics.join(", ")) } },
        ),
        Event::RepoArchived { repo } => ("dot idle", html! { b { (actor) } " archived " (repo) }),
        Event::RepoUnarchived { repo } => {
            ("dot idle", html! { b { (actor) } " unarchived " (repo) })
        }
        Event::RepoDeleted { repo } => ("dot bad", html! { b { (actor) } " deleted " (repo) }),
        Event::ThreadReplied { change, .. } => (
            "dot idle",
            html! {
                b { (actor) } " replied in a thread on " (change_num(numbers, change.as_str()))
            },
        ),
        Event::ThreadResolved {
            change,
            how,
            revision,
            ..
        } => (
            "dot ok",
            html! {
                b { (actor) } " resolved a thread on " (change_num(numbers, change.as_str()))
                " as " (how.as_str())
                @if let Some(revision) = revision { " in revision " (revision) }
            },
        ),
        Event::TeamMemberRemoved { team, member } => (
            "dot idle",
            html! {
                b { (actor) } " removed " (people.name(member).0) " from " (team.as_str())
            },
        ),
        Event::TeamRoleSet { team, member, role } => (
            "dot idle",
            html! {
                b { (actor) } " made " (people.name(member).0)
                @if *role == ambolt_core::TeamRole::Owner { " an owner of " } @else { " a member of " }
                (team.as_str())
            },
        ),
        Event::TeamSettingsSet { team, members_act } => (
            "dot idle",
            html! {
                b { (actor) } " set " (team.as_str()) "'s members to act as " (members_act.as_str())
            },
        ),
        Event::OrgTeamMade { organisation, team } => (
            "dot idle",
            html! { b { (actor) } " made the team " (organisation.as_str()) "/" (team) },
        ),
        Event::OrgTeamRemoved { organisation, team } => (
            "dot idle",
            html! { b { (actor) } " removed the team " (organisation.as_str()) "/" (team) },
        ),
        Event::OrgTeamMemberAdded {
            organisation,
            team,
            member,
        } => (
            "dot idle",
            html! {
                b { (actor) } " put " (people.name(member).0) " on " (organisation.as_str()) "/" (team)
            },
        ),
        Event::OrgTeamMemberRemoved {
            organisation,
            team,
            member,
        } => (
            "dot idle",
            html! {
                b { (actor) } " took " (people.name(member).0) " off " (organisation.as_str()) "/" (team)
            },
        ),
        Event::PolicySet { repo, .. } => (
            "dot idle",
            html! {
                b { (actor) } " set the policy for " (repo)
            },
        ),
        Event::MirrorSet { repo, mirror } => (
            "dot idle",
            html! {
                b { (actor) }
                @match mirror {
                    Some(mirror) => { " mirrors " (repo) " to " (mirror.url) }
                    None => { " stopped mirroring " (repo) }
                }
            },
        ),
        Event::MirrorPushed {
            branch, ok, detail, ..
        } => (
            if *ok { "dot ok" } else { "dot bad" },
            html! {
                @if *ok {
                    "mirrored " b { (branch) } " outward"
                } @else {
                    b { "mirror push failed" } " for " (branch)
                    @if let Some(detail) = detail { " — " (detail) }
                }
            },
        ),
        Event::TagPushed {
            name, commit_oid, ..
        } => (
            "dot ok",
            html! {
                b { (actor) } " tagged " b { (name) } " at " (commit_oid.get(..9).unwrap_or(commit_oid))
            },
        ),
        Event::PrincipalRegistered { principal, .. } => (
            "dot idle",
            html! {
                b { (actor) } " registered " (people.name(principal).0)
            },
        ),
        Event::QuotaSet { owner, .. } | Event::QuotaOverridden { owner, .. } => (
            "dot idle",
            html! {
                b { (actor) } " set what " (people.name(owner).0) " may take up"
            },
        ),
        Event::TaskStateChanged { state, .. } => (
            "dot idle",
            html! {
                b { (actor) } " marked a task " (state.as_str())
            },
        ),
        Event::ClaimVerified { change, agrees, .. } => (
            if *agrees { "dot ok" } else { "dot bad" },
            html! {
                b { (actor) }
                @if *agrees { " reproduced a claim on " } @else { " could not reproduce a claim on " }
                (change_num(numbers, change.as_str()))
            },
        ),
        Event::RebaseFailed { change, files, .. } => (
            "dot bad",
            html! {
                b { (change_num(numbers, change.as_str())) }
                " could not be carried onto the new base — conflicts in "
                (files.join(", "))
            },
        ),
        Event::ChangeAbandoned { change, .. } => (
            "dot bad",
            html! {
                b { (change_num(numbers, change.as_str())) } " was abandoned"
            },
        ),
        Event::TokenMinted { principal, .. } => (
            "dot idle",
            html! {
                b { (actor) } " minted a token for " (people.name(principal).0)
            },
        ),
        Event::TokenRevoked { .. } => ("dot idle", html! { b { (actor) } " revoked a token" }),
    }
}

/// What kind of thing an event is, for the filter pills: the log's own
/// tag, grouped the way a reader thinks about it.
/// Every principal an event names on a page, the actor first, so the
/// page can look their names up once.
pub fn named_in(envelope: &Envelope) -> Vec<&str> {
    let mut ids = vec![envelope.actor.as_str()];
    match &envelope.event {
        Event::GrantIssued { grantee, .. } => ids.push(grantee.as_str()),
        Event::PasswordSet { principal, .. }
        | Event::PasswordResetRequested { principal, .. }
        | Event::IdentityLinked { principal, .. }
        | Event::IdentityUnlinked { principal, .. }
        | Event::WorkloadBound { principal, .. }
        | Event::WorkloadUnbound { principal, .. }
        | Event::PrincipalDeactivated { principal, .. }
        | Event::PrincipalReactivated { principal, .. }
        | Event::PrincipalRegistered { principal, .. }
        | Event::TokenMinted { principal, .. } => ids.push(principal.as_str()),
        Event::RepoTransferOffered { to, .. } => ids.push(to.as_str()),
        Event::TeamMemberAdded { member, .. }
        | Event::TeamMemberRemoved { member, .. }
        | Event::TeamRoleSet { member, .. }
        | Event::OrgTeamMemberAdded { member, .. }
        | Event::OrgTeamMemberRemoved { member, .. } => ids.push(member.as_str()),
        Event::QuotaSet { owner, .. } => ids.push(owner.as_str()),
        Event::AttentionDrawn { reviewers, .. } => ids.extend(reviewers.iter().map(|r| r.as_str())),
        _ => {}
    }
    ids
}

pub fn event_group(event: &Event) -> &'static str {
    let tag = serde_json::to_value(event)
        .ok()
        .and_then(|v| v.get("kind").and_then(|k| k.as_str()).map(str::to_owned))
        .unwrap_or_default();
    match tag.as_str() {
        t if t.starts_with("change_") || t.starts_with("revision_") => "changes",
        t if t.starts_with("verdict_") || t.starts_with("thread_") => "reviews",
        t if t.starts_with("claim_") => "claims",
        t if t.starts_with("task_") || t.starts_with("session_") || t.starts_with("paths_") => {
            "work"
        }
        _ => "forge",
    }
}

fn group_words(group: &str) -> &'static str {
    match group {
        "changes" => "Changes",
        "reviews" => "Reviews",
        "claims" => "Claims",
        "work" => "Tasks and sessions",
        _ => "Repository",
    }
}

/// Day-grouped rows of events, each with its actor drawn and a sentence.
fn event_days(
    refs: &Refs,
    events: &[Envelope],
    people: &People,
    scopes: Option<&HashMap<i64, Option<String>>>,
) -> Markup {
    let mut days: Vec<(String, Vec<&Envelope>)> = Vec::new();
    for envelope in events {
        let day = day_of(&envelope.ts);
        match days.last_mut() {
            Some((last, group)) if *last == day => group.push(envelope),
            _ => days.push((day, vec![envelope])),
        }
    }
    html! {
        @for (day, group) in &days {
            div class="sec" {
                div class="sh" { h2 { (day_label(day)) } span class="n" { (short_day(day)) } }
                div class="panel" {
                    @for envelope in group {
                        @let (_, text) = describe(refs, envelope, people);
                        @let (display, agent) = people.name(&envelope.actor);
                        div class="row feed" {
                            (avatar(envelope.actor.as_str(), display, agent, false))
                            span class="s wrap" {
                                @if let Some(Some(repo)) = scopes.and_then(|s| s.get(&envelope.seq.0)) { a class="chip" href={ "/" (repo) "/activity" } { (repo) } " " }
                                (text)
                                @if let Some(via) = &envelope.via { span class="sec3" title=(via.as_str()) { " · in a session" } }
                            }
                            span class="age" title=(envelope.ts) { (ago(&envelope.ts)) }
                        }
                    }
                }
            }
        }
    }
}

pub struct ActivityPage<'a> {
    pub theme: Theme,
    pub who: Reading<'a>,
    pub repo: &'a str,
    pub numbers: &'a HashMap<String, (i64, String)>,
    pub after: i64,
    pub events: &'a [Envelope],
    pub group: Option<&'a str>,
    pub people: &'a People,
    /// Where the next window of events starts, when this one was full.
    pub next: Option<i64>,
}

pub fn log(page: ActivityPage<'_>) -> Markup {
    let ActivityPage {
        theme,
        who,
        repo,
        numbers,
        after,
        events,
        group,
        people,
        next,
    } = page;
    let refs: Refs = numbers
        .iter()
        .map(|(id, (number, title))| (id.as_str(), (*number, title.as_str())))
        .collect();
    let href = |g: Option<&str>| match g {
        Some(g) => format!("/{repo}/activity?kind={g}"),
        None => format!("/{repo}/activity"),
    };
    layout_reading(
        theme,
        who,
        Some(repo),
        Some(Tab::Activity),
        "Activity",
        html! {
            div class="sec top" {
                div class="filters" {
                    a class=[group.is_none().then_some("on")] href=(href(None)) { "All" }
                    @for g in ["changes", "reviews", "claims", "work", "forge"] {
                        a class=[(group == Some(g)).then_some("on")] href=(href(Some(g))) { (group_words(g)) }
                    }
                }
                @if events.is_empty() {
                    div class="panel" { div class="empty" { @if after == 0 { "Nothing has happened here yet." } @else { "Nothing more recent." } } }
                }
            }
            (event_days(&refs, events, people, None))
            @if let Some(next) = next {
                div class="sec" { a class="btn2 sm" href={ "/" (repo) "/activity?after=" (next) @if let Some(g) = group { "&kind=" (g) } } { "Later events" } }
            }
        },
    )
}

/// Blame that answers what was *known*, not just who typed. Each line
/// carries the change that landed it; lines whose change never ran an
/// executed check, or whose claims named a gap, are marked — the
/// question "which code here was never actually verified" is the one
/// this view exists to answer.
pub fn blame(theme: Theme, who: Reading<'_>, repo: &str, path: &str, rows: &[BlameRow]) -> Markup {
    let state_of = |row: &BlameRow| ambolt_core::line_state(row.provenance.as_deref());
    let count =
        |state: ambolt_core::LineState| rows.iter().filter(|r| state_of(r) == state).count();
    let reproduced = count(ambolt_core::LineState::Reproduced);
    let claimed = count(ambolt_core::LineState::Claimed);
    let with_gaps = count(ambolt_core::LineState::Gap);
    let argued = count(ambolt_core::LineState::Argued);
    let unattributed = count(ambolt_core::LineState::Imported);
    let bytes: usize = rows.iter().map(|r| r.text.len() + 1).sum();
    let mut coder = super::highlight::Coder::for_path(path, bytes);
    layout_reading(
        theme,
        who,
        Some(repo),
        Some(Tab::Code),
        path,
        html! {
            div class="crumbs" { (breadcrumbs(repo, path)) }
            div class="code blame" {
                header {
                    (ic("review", ""))
                    code { (path) }
                    (kv(&[
                        ("Lines", html! { (rows.len()) }),
                        ("Reproduced", html! { (reproduced) }),
                        ("Claimed only", html! { (claimed) }),
                        ("Under a gap", html! { @if with_gaps > 0 { span class="warn" { (with_gaps) } } @else { "0" } }),
                        ("Argued", html! { (argued) }),
                        ("Imported", html! { (unattributed) }),
                    ]))
                    div class="right" {
                        a class="ghost sm" href={ "/" (repo) "/coverage" } { (ic("coverage", "sm")) "Whole repository" }
                        a class="ghost sm" href={ "/" (repo) "/tree/" (path) } { (ic("code", "sm")) "Source" }
                    }
                }
                pre class="source" {
                    @for (index, row) in rows.iter().enumerate() {
                        // Attribution is labelled once per run of lines from
                        // the same change, the way a reader scans it.
                        @let starts_run = index == 0
                            || rows[index - 1].provenance.as_ref().map(|p| p.change.number)
                                != row.provenance.as_ref().map(|p| p.change.number);
                        @let state = state_of(row);
                        div class={ "cline " (state.as_str()) @if starts_run { " run" } } {
                            span class="who" {
                                @if starts_run {
                                    @match &row.provenance {
                                        Some(p) => {
                                            a class="link" href={ "/" (repo) "/changes/" (p.change.number) }
                                              title=(attribution(p)) {
                                                "#" (p.change.number)
                                            }
                                        }
                                        None => { span class="sec3" { "—" } }
                                    }
                                }
                            }
                            span class="no" title=(state_words(state)) { (row.number) }
                            code class="src" { (PreEscaped(coder.line(&row.text, &[]))) }
                        }
                    }
                }
            }
            (coverage_gaps(repo, rows))
        },
    )
}

/// The tooltip a line carries: what was claimed, who approved, and
/// what nobody checked.
fn attribution(p: &ambolt_core::Provenance) -> String {
    let mut parts = vec![p.change.title.clone()];
    for claim in &p.claims {
        let mark = if claim.passed { "passed" } else { "failed" };
        parts.push(format!(
            "{} {mark} — {}",
            claim.kind.as_str(),
            claim.summary
        ));
    }
    for verdict in p.approvals() {
        parts.push(format!(
            "approved by {} ({})",
            verdict.by,
            verdict.domain.as_str()
        ));
    }
    for gap in p.unchecked() {
        parts.push(format!("not checked: {gap}"));
    }
    parts.join("\n")
}

/// Everything the changes behind this file declared out of scope,
/// collected in one place.
fn state_words(state: ambolt_core::LineState) -> &'static str {
    match state {
        ambolt_core::LineState::Reproduced => {
            "reproduced: a runner re-ran the claim that landed this"
        }
        ambolt_core::LineState::Claimed => "claimed: its author ran something nobody re-ran",
        ambolt_core::LineState::Gap => "gap: the claim that landed this said what it did not check",
        ambolt_core::LineState::Argued => "argued: only a reasoning claim, nothing executed",
        ambolt_core::LineState::Imported => {
            "imported: from before the forge; nothing here judged it"
        }
    }
}

/// One stacked bar of the five states, widths in percent of the total.
/// Drawn as SVG rectangles: their widths are attributes, which the
/// stylesheet policy allows where an inline style would be refused.
/// Two lines over the tips the map was drawn at: total debt, and the
/// imported part of it. No axes: the legend under it carries the numbers.
fn burndown(history: &[ambolt_core::DebtSnapshot]) -> Markup {
    let (w, h) = (700.0_f64, 120.0_f64);
    let debt = |p: &ambolt_core::DebtSnapshot| (p.claimed + p.gap + p.argued + p.imported) as f64;
    let top = history.iter().map(debt).fold(1.0_f64, f64::max);
    let x = |i: usize| {
        if history.len() < 2 {
            0.0
        } else {
            i as f64 / (history.len() - 1) as f64 * w
        }
    };
    let y = |v: f64| h - 6.0 - (v / top) * (h - 14.0);
    let points = |f: &dyn Fn(&ambolt_core::DebtSnapshot) -> f64| -> String {
        history
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{:.1},{:.1}", x(i), y(f(p))))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let debt_points = points(&debt);
    let imported_points = points(&|p| p.imported as f64);
    html! {
        svg class="burndown" viewBox={ "0 0 " (w) " " (h) } preserveAspectRatio="none" role="img" aria-label="debt and imported lines over landings" {
            line class="axis" x1="0" y1=(h - 1.0) x2=(w) y2=(h - 1.0) {}
            polyline class="imported" points=(imported_points) {}
            polyline class="debt" points=(debt_points) {}
        }
    }
}

fn stack(counts: &crate::debt::Counts) -> Markup {
    let total = counts.total().max(1) as f64;
    let parts = [
        ("s-reproduced", counts.reproduced),
        ("s-claimed", counts.claimed),
        ("s-gap", counts.gap),
        ("s-argued", counts.argued),
        ("s-imported", counts.imported),
    ];
    let mut x = 0.0;
    let mut rects = Vec::new();
    for (class, n) in parts {
        if n > 0 {
            let w = n as f64 * 100.0 / total;
            rects.push((class, format!("{x:.3}"), format!("{w:.3}")));
            x += w;
        }
    }
    html! {
        svg class="bars" viewBox="0 0 100 6" preserveAspectRatio="none" aria-hidden="true" {
            @for (class, x, w) in &rects {
                rect class=(class) x=(x) y="0" width=(w) height="6" {}
            }
        }
    }
}

fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// The verification map: what backs every line of the default branch,
/// for the whole and by file, most debt first. Ten files show unless
/// all are asked for; the API pages the rest.
pub struct CoveragePage<'a> {
    pub theme: Theme,
    pub who: Reading<'a>,
    pub repo: &'a str,
    pub map: &'a crate::debt::DebtMap,
    pub history: &'a [ambolt_core::DebtSnapshot],
    pub show_all: bool,
    pub people: &'a People,
}

pub fn debt(page: CoveragePage<'_>) -> Markup {
    let CoveragePage {
        theme,
        who,
        repo,
        map,
        history,
        show_all,
        people,
    } = page;
    let c = &map.counts;
    let signed = who.viewer().is_some();
    let total = c.total().max(1);
    let percent = c.reproduced * 100 / total;
    let shown = if show_all {
        map.files.len()
    } else {
        map.files.len().min(10)
    };
    // The verified share at each of the last tips, for the sparkline.
    let spark: Vec<f64> = history
        .iter()
        .map(|p| {
            let all = (p.reproduced + p.claimed + p.gap + p.argued + p.imported).max(1) as f64;
            p.reproduced as f64 / all
        })
        .collect();
    let week_gain: Option<i64> = history
        .first()
        .zip(history.last())
        .map(|(a, b)| b.reproduced - a.reproduced);
    let rail = html! {
        div class="stat" {
            span class="k" { (ic("coverage", "sm")) "Verified" }
            span class="v" { (percent) small { "%" } }
            span class={ "d" @if week_gain.is_some_and(|g| g > 0) { " good-t" } } {
                @match week_gain {
                    Some(g) if g > 0 => { "+" (thousands(g as usize)) " lines over the last " (history.len()) " tips" }
                    Some(g) if g < 0 => { (thousands(g.unsigned_abs() as usize)) " fewer lines reproduced than " (history.len()) " tips ago" }
                    _ => { (thousands(c.reproduced)) " of " (thousands(c.total())) " lines reproduced" }
                }
            }
            @if spark.len() >= 2 {
                @let points: String = spark.iter().enumerate().map(|(i, v)| format!("{:.1},{:.1}", i as f64 * 120.0 / (spark.len() - 1) as f64, 30.0 - v * 28.0)).collect::<Vec<_>>().join(" ");
                div class="spark" { svg viewBox="0 0 120 32" preserveAspectRatio="none" aria-hidden="true" { polyline points=(points) {} } }
            }
        }
        @if signed && map.files.iter().any(|f| f.counts.debt() > 0 && f.task.is_none()) {
            div class="panel" {
                header { h2 { "Pay it down" } span class="n" { (thousands(c.imported)) " imported" } }
                form class="pad form" method="post" action={ "/" (repo) "/debt/tasks" } {
                    p class="what" { "Each task names a file and the lines nobody here has judged, and asks for a claim a runner can re-run." }
                    div class="line" {
                        input class="input sm num" id="count" name="count" type="number" min="1" max="50" value="5" aria-label="How many";
                        span class="lbl" { "most indebted files without a task" }
                        button class="btn sm" type="submit" { "Create tasks" }
                    }
                }
            }
        }
        @if !map.paid_down.is_empty() {
            div class="panel" {
                header { h2 { "Paid down" } span class="n" { "by reproduced covering claims" } }
                @for paid in &map.paid_down {
                    @let id = ambolt_core::PrincipalId(paid.by.clone());
                    @let (display, agent) = people.name(&id);
                    div class="ev-row" {
                        (avatar(&paid.by, display, agent, false))
                        div {
                            div class="h" { b { (display) } span class="sec2" { "paid down " (thousands(paid.lines)) " lines" } }
                            div class="sub good" { (ic("check", "sm")) (paid.claims) " covering claim(s), reproduced · " (paid.files) " file(s)" }
                        }
                    }
                }
            }
        }
    };
    layout_reading_with(
        theme,
        who,
        Some(repo),
        Some(Tab::Coverage),
        "Coverage",
        html! {
            div class="chg-body" {
                div class="panel" {
                    header {
                        h2 { "What backs this code" }
                        (kv(&[
                            ("Branch", html! { (map.branch) " at " code { (short(&map.tip)) } }),
                            ("Lines", html! { (thousands(c.total())) " in " (map.files.len()) " files" }),
                            if map.skipped > 0 { ("Not counted", html! { (map.skipped) " binary or large" }) } else { ("", html! {}) },
                        ]))
                    }
                    div class="pad" {
                        div class="bigbars" { (stack(c)) }
                        div class="legend" {
                            span { i class="s-reproduced" {} b { (thousands(c.reproduced)) } " reproduced" }
                            span { i class="s-claimed" {} b { (thousands(c.claimed)) } " claimed, not re-run" }
                            span { i class="s-gap" {} b { (thousands(c.gap)) } " under a declared gap" }
                            span { i class="s-argued" {} b { (thousands(c.argued)) } " argued only" }
                            span { i class="s-imported" {} b { (thousands(c.imported)) } " imported, never judged" }
                        }
                    }
                    div class="foot" { (ic("coverage", "sm")) "A line is reproduced when a runner re-ran the claim that landed it. A gap is a line that landed under a claim which said it did not check this. Imported lines predate this forge." }
                }
                @if history.len() >= 2 {
                    @let first = &history[0];
                    @let last = &history[history.len() - 1];
                    div class="panel" {
                        header { h2 { "Burndown" } span class="n" { (map.branch) " · last " (history.len()) " tips" } }
                        div class="pad" {
                            (burndown(history))
                            div class="legend" {
                                span { i class="s-debt" {} b { (thousands((last.claimed + last.gap + last.argued + last.imported) as usize)) } " debt · was " (thousands((first.claimed + first.gap + first.argued + first.imported) as usize)) }
                                span { i class="s-imported" {} b { (thousands(last.imported as usize)) } " imported · was " (thousands(first.imported as usize)) }
                            }
                        }
                        div class="foot" { "Debt is every line short of a reproduced claim. It falls when a runner reproduces a claim that covers existing code, or when a file is rewritten under one." }
                    }
                }
                div class="panel" {
                    header { h2 { "By file" } span class="n" { (map.files.len()) } div class="right" { span class="sec3" { "sorted by lines unjudged" } } }
                    @if map.files.is_empty() {
                        div class="empty" { "Nothing on " (map.branch) " yet." }
                    } @else {
                        div class="thead fv" { span { "File" } span {} span class="r" { "Unjudged" } span class="r" { "Lines" } }
                        @for file in map.files.iter().take(shown) {
                            a class="row fv" href={ "/" (repo) "/blame/" (file.path) } {
                                span class="tt" {
                                    code { (file.path) }
                                    @if let Some((_, holders)) = &file.task {
                                        span class="s" { "task open" @if !holders.is_empty() { " · claimed by " (holders.iter().map(|h| people.name(&ambolt_core::PrincipalId(h.clone())).0.to_owned()).collect::<Vec<_>>().join(", ")) } }
                                    } @else if let Some(cover) = file.covered_by.first() {
                                        span class="s" { "covered by #" (cover.change) @if cover.reproduced { " · runner reproduced" } @else { " · not yet re-run" } }
                                    }
                                }
                                (stack(&file.counts))
                                span class={ "num r" @if file.counts.debt() == 0 { " sec3" } } { (thousands(file.counts.debt())) }
                                span class="num r sec3" { (thousands(file.counts.total())) }
                            }
                        }
                        @if map.files.len() > shown {
                            div class="foot" { a class="btn2 sm" href={ "/" (repo) "/coverage?all=1" } { "Show all " (map.files.len()) " files" } }
                        } @else if show_all && map.files.len() > 10 {
                            div class="foot" { a class="btn2 sm" href={ "/" (repo) "/coverage" } { "Show the top ten" } }
                        }
                    }
                }
            }
        },
        Some(rail),
    )
}

fn coverage_gaps(repo: &str, rows: &[BlameRow]) -> Markup {
    let mut seen: Vec<(i64, String, String)> = Vec::new();
    for row in rows {
        let Some(p) = &row.provenance else { continue };
        for gap in p.unchecked() {
            let entry = (p.change.number, p.change.title.clone(), gap.to_owned());
            if !seen.contains(&entry) {
                seen.push(entry);
            }
        }
    }
    html! {
        @if !seen.is_empty() {
            section class="gaps" {
                header { h2 { "Declared gaps" } span { (seen.len()) } }
                @for (number, title, gap) in &seen {
                    div class="gap-row" {
                        a class="link sec2" href={ "/" (repo) "/changes/" (number) } { "#" (number) " " (title) }
                        span { (gap) }
                    }
                }
            }
        }
    }
}

/// What earlier attempts learned. A corpus nobody has to maintain: the
/// protocol already refuses to let a session end without recording an
/// outcome, so failure leaves knowledge behind by construction.
pub fn lessons(
    theme: Theme,
    who: Reading<'_>,
    repo: &str,
    search: Option<&str>,
    lessons: &[ambolt_core::Lesson],
    people: &People,
) -> Markup {
    layout_reading(
        theme,
        who,
        Some(repo),
        Some(Tab::Lessons),
        "Lessons",
        html! {
            div class="sec top" {
                div class="sh" {
                    h2 { "Lessons" }
                    span class="n" { (lessons.len()) }
                    form class="right search-in" method="get" action={ "/" (repo) "/lessons" } {
                        (ic("search", "sm"))
                        input class="input sm" type="search" name="q" value=[search]
                              placeholder="Has anyone tried this before?";
                    }
                }
                div class="panel" {
                    @if lessons.is_empty() {
                        div class="empty" {
                            @match search {
                                Some(term) => { "Nothing recorded matches " (term) "." }
                                None => { "No sessions have ended yet." }
                            }
                        }
                    }
                    @for lesson in lessons {
                        @let (display, agent) = people.name(&lesson.agent);
                        div class="row ls" {
                            (avatar(lesson.agent.as_str(), display, agent, false))
                            span class="tt" {
                                span class="t" {
                                    (lesson.task_title) " "
                                    @if lesson.state == ambolt_core::SessionState::Failed { span class="chip bad" { "stopped" } } @else { span class="chip good" { "done" } }
                                }
                                span class="s wrap" { (lesson.outcome) }
                            }
                            span class="age" { (display) }
                        }
                    }
                    div class="foot" { (ic("sparkle", "sm")) "When an agent stops, it says why. Those reasons are searched before the next attempt." }
                }
            }
        },
    )
}

/// A principal's record in one quiet line: what the log says a runner
/// found of their claims, and what humans said of their changes.
pub fn record_words(record: &ambolt_core::Record) -> String {
    let mut words = if record.judged > 0 {
        format!(
            "{} of {} claims reproduced",
            record.reproduced, record.judged
        )
    } else if record.claims > 0 {
        format!("{} claims, none re-run yet", record.claims)
    } else {
        "no claims yet".to_owned()
    };
    if record.disputed > 0 {
        words.push_str(&format!(" · {} disputed", record.disputed));
    }
    if record.blocks > 0 {
        words.push_str(&format!(
            " · blocked by a person {} time{}",
            record.blocks,
            if record.blocks == 1 { "" } else { "s" }
        ));
    }
    words
}

/// A team inside an organisation: who is on it, and what it holds on
/// the organisation's repositories. Its owners change it from here.
#[allow(clippy::too_many_arguments)] // one page, one set of facts about the team
pub fn org_team(
    theme: Theme,
    viewer: &Viewer,
    organisation: &ambolt_core::PrincipalId,
    team: &str,
    members: &[ambolt_core::PrincipalId],
    holds: &[ambolt_core::Grant],
    may_manage: bool,
    error: Option<&str>,
    people: &People,
) -> Markup {
    let named = format!("{organisation}/{team}");
    layout(
        theme,
        Some(viewer),
        None,
        None,
        &named,
        html! {
            div class="pagehead" {
                div class="who" {
                    div {
                        h1 { (named) }
                        div class="meta" {
                            a href={ "/" (organisation.as_str()) } { (organisation.as_str()) }
                            span class="chip" { (ic("agents", "")) "Team" }
                        }
                    }
                }
            }
            @if let Some(error) = error { div class="notice bad" { (ic("alert", "")) span { (error) } } }
            div class="sec" {
                div class="sh" { h2 { "Members" } span class="n" { (members.len()) } }
                div class="panel" {
                    @if members.is_empty() { div class="empty" { "Nobody yet. Only the organisation's members can be on its teams." } }
                    @for member in members {
                        @let (display, agent) = people.name(member);
                        div class="row member" {
                            (avatar(member.as_str(), display, agent, false))
                            span class="tt" {
                                a class="t" href={ "/" (member.as_str()) } { (display) }
                                span class="s" { (member.as_str()) }
                            }
                            span class="acts" {
                                @if may_manage {
                                    form method="post" action={ "/" (organisation.as_str()) "/teams/" (team) "/members" } {
                                        input type="hidden" name="action" value="remove";
                                        input type="hidden" name="member" value=(member.as_str());
                                        button class="ghost sm danger" type="submit" { "Remove" }
                                    }
                                }
                            }
                        }
                    }
                    @if may_manage {
                        form class="foot" method="post" action={ "/" (organisation.as_str()) "/teams/" (team) "/members" } {
                            input type="hidden" name="action" value="add";
                            input class="input sm" type="text" name="member" placeholder="Who, on the organisation" pattern="[a-z0-9-]{2,64}" required aria-label="Member";
                            button class="btn2 sm" type="submit" { "Add" }
                        }
                    }
                }
            }
            div class="sec" {
                div class="sh" { h2 { "Holds" } span class="n" { (holds.len()) } }
                div class="panel" {
                    @if holds.is_empty() { div class="empty" { "Nothing yet. Grant it access from a repository's settings, as " code { (named) } "." } }
                    @for grant in holds {
                        div class="line" {
                            a href={ "/" (grant.repo.as_deref().unwrap_or("")) "/settings" } { (grant.repo.as_deref().unwrap_or("everything")) }
                            @for action in &grant.actions { span class="chip" { (action.as_str()) } }
                            span class="s" { "from " (grant.grantor.as_str()) }
                        }
                    }
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials_take_two_words_or_two_letters() {
        assert_eq!(initials("Mandip Adhikari"), "MA");
        assert_eq!(initials("scout"), "SC");
        assert_eq!(initials("Ada Byron Lovelace"), "AB");
        assert_eq!(initials(""), "?");
    }

    #[test]
    fn a_mark_is_an_image_for_a_person_or_an_agent_and_a_tile_for_an_organisation() {
        let agent = avatar("scout", "Scout", true, true).into_string();
        assert!(agent.contains("av agent live"), "{agent}");
        assert!(agent.contains("/avatars/2/agent/scout.svg?live"), "{agent}");
        let person = avatar("mandip", "Mandip Adhikari", false, false).into_string();
        assert!(person.contains("/avatars/2/person/mandip.svg"), "{person}");
        assert!(!person.contains("agent"), "{person}");
        let org = avatar_of("crew", "Crew", ambolt_core::PrincipalKind::Team, false).into_string();
        assert!(org.contains("av org") && org.contains(">CR<"), "{org}");
    }

    #[test]
    fn only_a_chosen_theme_stamps_the_root() {
        assert_eq!(Theme::System.attr(), None);
        assert_eq!(Theme::Light.attr(), Some("light"));
        assert_eq!(Theme::Dark.attr(), Some("dark"));
    }

    #[test]
    fn every_icon_the_pages_ask_for_is_in_the_sprite() {
        for name in [
            "home", "inbox", "tasks", "agents", "repo", "check", "x", "alert", "search", "plus",
            "code", "changes", "review", "coverage", "activity", "settings", "chev", "user", "key",
            "globe", "sun", "moon", "logout", "menu",
        ] {
            assert!(SPRITE.contains(&format!("id=\"i-{name}\"")), "{name}");
        }
        assert!(ic("check", "sm").into_string().contains("#i-check"));
    }
}
