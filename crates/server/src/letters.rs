//! What the forge puts in somebody's inbox.
//!
//! Every message is the same shape: the wordmark, one sentence saying
//! what happened, the facts it rests on, one thing to do, and the line
//! that says what to do if it was not you. The HTML is written the way
//! mail clients still want it — tables, inline styles, one 600px column
//! — with a plain-text twin that says the same thing, because a message
//! whose HTML is stripped must still be readable and a link must still
//! be copyable.

use maud::{PreEscaped, html};

/// One message, in both the forms it is sent in.
pub struct Letter {
    pub subject: String,
    pub text: String,
    pub html: String,
}

/// Paper ground, iron ink, one cobalt for the thing to do — the web
/// tokens, as literals, because a mail client has no stylesheet.
const INK: &str = "#0F1420";
const INK2: &str = "#4B5565";
const INK3: &str = "#5F6A7B";
const HAIR: &str = "#E6E9EF";
const PAPER: &str = "#F2F4F7";
const CARD: &str = "#FFFFFF";
const PANEL: &str = "#F7F8FA";
const ACCENT: &str = "#2F5BEA";
const ACCENT_SOFT: &str = "#EDF1FE";
const SANS: &str = "'Familjen Grotesk',-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif";
const MONO: &str = "'JetBrains Mono',ui-monospace,SFMono-Regular,Menlo,Consolas,monospace";

/// A label and the fact under it, as a row of the small table under the
/// heading. Only facts the reader can act on or check belong here.
type Fact<'a> = (&'a str, String);

/// The one message shape. Everything the forge sends fills this in.
struct Body<'a> {
    /// The chip above the heading: what kind of message this is.
    kind: &'a str,
    title: &'a str,
    lead: &'a str,
    facts: Vec<Fact<'a>>,
    /// The one thing to do, and where it goes.
    action: Option<(&'a str, &'a str)>,
    /// Under the rule: what the link does, how long it lasts, and what
    /// to do if the reader was not expecting it.
    fine: Vec<String>,
    /// The forge this came from, as a person would type it.
    host: String,
}

impl Body<'_> {
    fn html(&self) -> String {
        let style = |s: &str| PreEscaped(s.to_owned());
        let page = html! {
            (PreEscaped("<!doctype html>"))
            html lang="en" {
                head {
                    meta charset="utf-8";
                    meta name="viewport" content="width=device-width, initial-scale=1";
                    meta name="color-scheme" content="light dark";
                    meta name="supported-color-schemes" content="light dark";
                    title { (self.title) }
                    style type="text/css" {
                        (PreEscaped(DARK_CSS))
                    }
                }
                body style=(style(&format!("margin:0;padding:0;background:{PAPER};-webkit-font-smoothing:antialiased;"))) {
                    // What an inbox shows beside the subject, and nothing
                    // twice: the lead, hidden in the message itself.
                    div style="display:none;max-height:0;overflow:hidden;opacity:0;" { (self.lead) }
                    table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" class="ground" style=(style(&format!("background:{PAPER};"))) {
                        tr { td align="center" style="padding:40px 12px 48px 12px;" {
                            table role="presentation" width="600" cellpadding="0" cellspacing="0" border="0" style="width:600px;max-width:600px;" {
                                tr { td style="padding:0 4px 18px 4px;" {
                                    span class="ink" style=(style(&format!("font:700 16px/1 {SANS};letter-spacing:-0.02em;color:{INK};"))) { "ambolt" }
                                } }
                                tr { td class="card" style=(style(&format!("background:{CARD};border:1px solid {HAIR};border-radius:14px;"))) {
                                    table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" {
                                        @let alone = self.facts.is_empty() && self.action.is_none() && self.fine.is_empty();
                                        tr { td style=(style(if alone { "padding:30px 34px;" } else { "padding:30px 34px 0 34px;" })) {
                                            table role="presentation" cellpadding="0" cellspacing="0" border="0" { tr {
                                                td class="chip" style=(style(&format!("background:{ACCENT_SOFT};border-radius:999px;padding:5px 11px;font:600 12px/1 {SANS};color:{ACCENT};"))) { (self.kind) }
                                            } }
                                            h1 class="ink" style=(style(&format!("margin:16px 0 0;font:600 25px/1.3 {SANS};letter-spacing:-0.02em;color:{INK};"))) { (self.title) }
                                            @for para in self.lead.split("\n\n") {
                                                p class="ink2" style=(style(&format!("margin:12px 0 0;font:400 15px/1.65 {SANS};color:{INK2};white-space:pre-wrap;"))) { (para) }
                                            }
                                        } }
                                        @if !self.facts.is_empty() {
                                            tr { td style="padding:22px 34px 0 34px;" {
                                                table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" style=(style(&format!("font:400 13.5px/1.5 {SANS};"))) {
                                                    @for (label, value) in &self.facts {
                                                        tr {
                                                            td class="hairtop ink3" width="150" style=(style(&format!("width:150px;padding:9px 12px 9px 0;border-top:1px solid {HAIR};color:{INK3};"))) { (label) }
                                                            td class="hairtop ink" style=(style(&format!("padding:9px 0;border-top:1px solid {HAIR};color:{INK};font-weight:600;"))) { (value) }
                                                        }
                                                    }
                                                }
                                            } }
                                        }
                                        @if let Some((label, href)) = self.action {
                                            tr { td style="padding:26px 34px 0 34px;" {
                                                table role="presentation" cellpadding="0" cellspacing="0" border="0" { tr {
                                                    td style=(style(&format!("background:{ACCENT};border-radius:10px;"))) {
                                                        a href=(href) class="btn" style=(style(&format!("display:inline-block;padding:13px 24px;font:600 15px/1 {SANS};color:#FFFFFF;text-decoration:none;"))) { (label) }
                                                    }
                                                } }
                                            } }
                                            tr { td style="padding:18px 34px 0 34px;" {
                                                p class="ink3" style=(style(&format!("margin:0 0 7px;font:400 12.5px/1.5 {SANS};color:{INK3};"))) { "Or paste this into your browser:" }
                                                table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" class="panel" style=(style(&format!("background:{PANEL};border-radius:10px;"))) {
                                                    tr { td style="padding:11px 13px;" {
                                                        a href=(href) class="url" style=(style(&format!("font:400 12px/1.6 {MONO};color:{ACCENT};text-decoration:none;word-break:break-all;"))) { (href) }
                                                    } }
                                                }
                                            } }
                                        }
                                        @if !self.fine.is_empty() {
                                            tr { td style="padding:26px 34px 30px 34px;" {
                                                @if self.action.is_some() {
                                                    div class="rule" style=(style(&format!("height:1px;background:{HAIR};line-height:1px;font-size:0;margin:0 0 16px;"))) { (PreEscaped("&nbsp;")) }
                                                }
                                                @for line in &self.fine {
                                                    p class="ink3" style=(style(&format!("margin:0 0 6px;font:400 13px/1.65 {SANS};color:{INK3};"))) { (line) }
                                                }
                                            } }
                                        } @else if !alone {
                                            // The card still needs a floor under whatever ended it.
                                            tr { td style="padding:0 34px 30px 34px;" {} }
                                        }
                                    }
                                } }
                                tr { td style="padding:16px 4px 0 4px;" {
                                    p class="ink3" style=(style(&format!("margin:0;font:400 12.5px/1.6 {SANS};color:{INK3};"))) {
                                        "Sent by the forge at " a href=(format!("https://{}", self.host)) class="ink3" style=(style(&format!("color:{INK3};text-decoration:none;"))) { (self.host) }
                                    }
                                } }
                            }
                        } }
                    }
                }
            }
        };
        page.into_string()
    }

    fn text(&self) -> String {
        let mut out = format!("{}\n\n{}\n", self.title, self.lead);
        for (label, value) in &self.facts {
            out.push_str(&format!("\n{label}: {value}"));
        }
        if !self.facts.is_empty() {
            out.push('\n');
        }
        if let Some((label, href)) = self.action {
            out.push_str(&format!("\n{label}:\n\n  {href}\n"));
        }
        if !self.fine.is_empty() {
            out.push('\n');
            for line in &self.fine {
                out.push_str(&format!("{line}\n"));
            }
        }
        out.push_str(&format!("\nSent by the forge at {}\n", self.host));
        out
    }
}

/// Dark where the reader's mail client says dark, and left alone where
/// it says nothing: every colour above is the light one, and these
/// replace them.
const DARK_CSS: &str = "@media (prefers-color-scheme: dark) {\
 body, .ground { background: #0B0D12 !important; }\
 .card { background: #10131A !important; border-color: #232A38 !important; }\
 .panel { background: #161A23 !important; }\
 .chip { background: #1B2540 !important; color: #8FB0FF !important; }\
 .ink, h1.ink { color: #F1F3F7 !important; }\
 .ink2 { color: #B4BBC8 !important; }\
 .ink3 { color: #7E8797 !important; }\
 .url { color: #8FB0FF !important; }\
 .hairtop { border-top-color: #232A38 !important; }\
 .rule { background: #232A38 !important; }\
}";

/// The forge's address as a person would type it, taken from a link the
/// forge just made.
fn host_of(link: &str) -> String {
    link.split("://")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .unwrap_or("this forge")
        .to_owned()
}

fn letter(subject: String, body: Body<'_>) -> Letter {
    Letter {
        subject,
        text: body.text(),
        html: body.html(),
    }
}

/// Somebody was invited: by whom, where to, and what the link does.
pub fn invitation(
    by: &str,
    username: &str,
    organisation: Option<&str>,
    link: &str,
    days: i64,
) -> Letter {
    let host = host_of(link);
    let subject = match organisation {
        Some(org) => format!("{by} invited you to {org} on {host}"),
        None => format!("{by} invited you to {host}"),
    };
    let title = match organisation {
        Some(org) => format!("{by} invited you to {org}"),
        None => format!("{by} invited you to the forge"),
    };
    let mut facts = vec![("Forge", host.clone())];
    if let Some(org) = organisation {
        facts.push(("Organisation", org.to_owned()));
    }
    facts.push(("Your username", username.to_owned()));
    letter(
        subject,
        Body {
            kind: "Invitation",
            title: &title,
            lead: "Open the link to sign in and choose a password. Your account is waiting under the username below.",
            facts,
            action: Some(("Accept the invitation", link)),
            fine: vec![
                match days {
                    1 => "The link signs you in once and stops working tomorrow.".to_owned(),
                    days => format!("The link signs you in once and stops working in {days} days."),
                },
                "If you were not expecting this, ignore it — nothing happens.".to_owned(),
            ],
            host,
        },
    )
}

/// An address was given for an account: prove it is the reader's.
pub fn confirm_address(who: &str, link: &str) -> Letter {
    let host = host_of(link);
    letter(
        format!("Confirm your address on {host}"),
        Body {
            kind: "Address",
            title: "Confirm this address",
            lead: "This address was given for an account on the forge. Confirming it is what lets the forge reach you — for a password reset, and for nothing you did not ask for.",
            facts: vec![("Account", who.to_owned()), ("Forge", host.clone())],
            action: Some(("Confirm this address", link)),
            fine: vec![
                "The link works once, within a day.".to_owned(),
                "If that was not you, ignore this and nothing changes.".to_owned(),
            ],
            host,
        },
    )
}

/// Somebody asked to reset a password. It may not have been the reader.
pub fn reset_password(who: &str, link: &str) -> Letter {
    let host = host_of(link);
    letter(
        format!("Reset your password on {host}"),
        Body {
            kind: "Password",
            title: "Set a new password",
            lead: "Somebody asked to reset the password for this account. If that was you, the link below takes you to a new one.",
            facts: vec![("Account", who.to_owned()), ("Forge", host.clone())],
            action: Some(("Set a new password", link)),
            fine: vec![
                "The link works once, within thirty minutes.".to_owned(),
                "If it was not you, nothing has changed and you can ignore this.".to_owned(),
            ],
            host,
        },
    )
}

/// A way in without a password, for whoever asked from the sign-in page.
pub fn signin_link(who: &str, link: &str) -> Letter {
    let host = host_of(link);
    letter(
        format!("Your sign-in link for {host}"),
        Body {
            kind: "Sign in",
            title: "Your sign-in link",
            lead: "Open it and you are signed in — no password needed this time.",
            facts: vec![("Account", who.to_owned()), ("Forge", host.clone())],
            action: Some(("Sign in", link)),
            fine: vec![
                "The link works once, within fifteen minutes.".to_owned(),
                "If you did not ask for it, ignore this; nothing changes.".to_owned(),
            ],
            host,
        },
    )
}

/// What a person said broke, for whoever runs the forge. Their words go
/// in as they wrote them; the chrome is the same as everything else.
pub fn report(
    id: i64,
    kind: &str,
    version: &str,
    place: &str,
    from: &str,
    what: &str,
    url: &str,
) -> Letter {
    let host = host_of(url);
    let title = match kind {
        "abuse" => format!("Report {id}: conduct or a name"),
        _ => format!("Report {id}: something broke"),
    };
    let mut facts = vec![("From", from.to_owned())];
    if !place.trim().is_empty() {
        facts.push(("Where", place.trim().to_owned()));
    }
    facts.push(("Version", version.to_owned()));
    letter(
        format!("Report {id} on {host}"),
        Body {
            kind: "Report",
            title: &title,
            lead: what.trim(),
            facts,
            action: None,
            fine: vec!["Reports are kept beside the waitlist, not in the log, so one can be removed when somebody asks.".to_owned()],
            host,
        },
    )
}

/// A short operational message to whoever runs the forge: the watcher
/// saying a forge went down or came back, and anything else that is one
/// title and a few lines.
pub fn notice(kind: &str, title: &str, body: &str, url: &str) -> Letter {
    let host = host_of(url);
    letter(
        title.to_owned(),
        Body {
            kind,
            title,
            lead: body,
            facts: Vec::new(),
            action: None,
            fine: Vec::new(),
            host,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Write every message to a directory, to be looked at in a browser
    /// rather than read as a string: `AMBOLT_LETTERS_OUT=/tmp/letters
    /// cargo test -p ambolt-server --lib -- --ignored`. Ignored because
    /// it proves nothing; it is for whoever is changing how they look.
    #[test]
    #[ignore]
    fn every_letter_is_written_out_to_look_at() {
        let out = std::env::var("AMBOLT_LETTERS_OUT").unwrap_or_else(|_| "/tmp".into());
        let link = "https://ambolt.sh/join?token=ambolt_examplelinkfromthedocsnotarealtoken0000000000000";
        let all = [
            ("invitation", invitation("mandip", "jane", None, link, 7)),
            (
                "invitation-org",
                invitation("mandip", "jane", Some("acme"), link, 7),
            ),
            (
                "confirm",
                confirm_address(
                    "jane",
                    "https://ambolt.sh/verify?token=ambolt_examplelinkfromthedocsnotarealtoken00000",
                ),
            ),
            (
                "reset",
                reset_password(
                    "jane",
                    "https://ambolt.sh/reset?token=ambolt_examplelinkfromthedocsnotareal",
                ),
            ),
            (
                "signin",
                signin_link(
                    "jane",
                    "https://ambolt.sh/signin?token=ambolt_examplelinkfromthedocsnotar",
                ),
            ),
            (
                "report",
                report(
                    12,
                    "bug",
                    "0.1.0-alpha.3",
                    "ambolt/forge",
                    "jane@example.test, signed in as jane",
                    "The blame page hangs on files over about ten thousand lines. Firefox 131, macOS.",
                    "https://ambolt.sh/report",
                ),
            ),
            (
                "watch",
                notice(
                    "Forge watch",
                    "https://ambolt.sh is down",
                    "Was up since 15 September 2026, 04:10.\n\nhealthz answered 502: Bad Gateway",
                    "https://ambolt.sh",
                ),
            ),
        ];
        for (name, l) in all {
            std::fs::write(format!("{out}/letter-{name}.html"), &l.html).unwrap();
            std::fs::write(
                format!("{out}/letter-{name}.txt"),
                format!("Subject: {}\n\n{}", l.subject, l.text),
            )
            .unwrap();
        }
    }

    #[test]
    fn a_letter_says_the_same_thing_in_both_forms() {
        let l = invitation(
            "mandip",
            "jane",
            None,
            "https://ambolt.sh/join?token=ambolt_abc",
            7,
        );
        assert_eq!(l.subject, "mandip invited you to ambolt.sh");
        for part in [&l.text, &l.html] {
            assert!(part.contains("https://ambolt.sh/join?token=ambolt_abc"));
            assert!(part.contains("jane"));
            assert!(part.contains("mandip"));
            assert!(part.contains("stops working in 7 days"));
        }
        assert!(l.html.starts_with("<!doctype html>"));
        assert!(l.text.contains("Accept the invitation:"));
    }

    #[test]
    fn an_organisation_is_named_where_it_matters() {
        let l = invitation(
            "ada",
            "jane",
            Some("acme"),
            "https://forge.example/join?token=t",
            3,
        );
        assert_eq!(l.subject, "ada invited you to acme on forge.example");
        assert!(l.html.contains("acme"));
        assert!(l.text.contains("Organisation: acme"));
    }

    #[test]
    fn what_a_stranger_wrote_cannot_carry_markup_into_the_message() {
        let l = report(
            7,
            "bug",
            "0.1.0",
            "<b>repo</b>",
            "nobody@example.test",
            "<script>alert(1)</script> the page broke",
            "https://ambolt.sh/report",
        );
        assert!(!l.html.contains("<script>"));
        assert!(l.html.contains("&lt;script&gt;"));
        assert!(l.text.contains("<script>alert(1)</script> the page broke"));
    }

    #[test]
    fn the_host_comes_from_the_link() {
        assert_eq!(host_of("https://ambolt.sh/join?token=x"), "ambolt.sh");
        assert_eq!(host_of("http://127.0.0.1:6160/reset"), "127.0.0.1:6160");
        assert_eq!(host_of("nonsense"), "this forge");
    }
}
