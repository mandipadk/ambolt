//! Saying something about a repository from inside a clone of it.
//!
//! This exists for the case that decides whether a community surface is
//! used at all: somebody asks the agent in their editor how to report a
//! bug, and the agent answers from whatever it can find. An agent that
//! finds only a git remote it does not recognise assumes the conventions
//! of another forge and files where nobody is looking.
//!
//! So the remote is the answer. `ambolt report` in a working copy reads
//! `origin`, works out which forge and which repository that is, and
//! needs to be told nothing else.

use anyhow::{Context, bail};
use serde_json::{Value, json};
use std::path::Path;
use std::process::Command;

/// Which forge, and which repository, a working copy belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    /// The forge's base URL, without a trailing slash.
    pub server: String,
    /// The repository as the graph names it: `owner/name`.
    pub repo: String,
}

/// Read a clone's remote and pull the forge and repository out of it.
///
/// An Ambolt remote is `<base>/git/<owner>/<name>`, optionally with a
/// `.git` suffix and optionally with a username in front of the host,
/// which is how git spells a token-authenticated clone. Anything else
/// is somebody else's forge and this says so rather than guessing.
pub fn from_remote(dir: &Path, remote: &str) -> anyhow::Result<Origin> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["remote", "get-url", remote])
        .output()
        .context("running git")?;
    if !out.status.success() {
        bail!(
            "no git remote named {remote} here. Pass --server and --repo, \
             or run this inside a clone of the repository"
        );
    }
    let url = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    parse_remote(&url).with_context(|| format!("remote {remote} is {url}"))
}

/// The parsing on its own, so it can be tested without a repository.
pub fn parse_remote(url: &str) -> anyhow::Result<Origin> {
    let url = url.trim().trim_end_matches('/');
    let url = url.strip_suffix(".git").unwrap_or(url);
    let Some((scheme, rest)) = url.split_once("://") else {
        bail!("only http and https remotes name an Ambolt forge");
    };
    if scheme != "http" && scheme != "https" {
        bail!("only http and https remotes name an Ambolt forge");
    }
    // A credential in the URL is git's way of carrying a token. It is
    // not part of the address and must not be echoed anywhere.
    let rest = rest.rsplit_once('@').map_or(rest, |(_, after)| after);
    let Some((host, path)) = rest.split_once('/') else {
        bail!("that remote names a host and nothing else");
    };
    let Some(repo) = path.strip_prefix("git/") else {
        bail!("an Ambolt repository is served under /git/<owner>/<name>; that remote is not");
    };
    if repo.split('/').count() != 2 || repo.split('/').any(|part| part.is_empty()) {
        bail!("an Ambolt repository is <owner>/<name>; that remote names {repo}");
    }
    Ok(Origin {
        server: format!("{scheme}://{host}"),
        repo: repo.to_owned(),
    })
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into()
}

/// How to act on a repository, as the forge itself describes it. Needs
/// no token, because somebody working out where to send a bug report
/// does not have one yet.
pub fn guide(server: &str, repo: &str) -> anyhow::Result<Value> {
    let mut response = agent()
        .get(&format!(
            "{}/api/repos/{repo}/guide",
            server.trim_end_matches('/')
        ))
        .call()?;
    let status = response.status().as_u16();
    let body: Value = response.body_mut().read_json()?;
    if status != 200 {
        bail!("{}", said(&body, status));
    }
    Ok(body)
}

/// What somebody came to say. Only `kind`, `title` and `body` are
/// required; for a bug the `command` is what decides whether anyone
/// else can check it, so it is worth the trouble when there is one.
#[derive(Debug, Clone, Default)]
pub struct Filing<'a> {
    pub kind: &'a str,
    pub title: &'a str,
    pub body: &'a str,
    pub version: Option<&'a str>,
    pub command: Option<&'a str>,
    pub observed: Option<&'a str>,
    pub expected: Option<&'a str>,
}

/// File a report. Returns the number it took in that repository.
pub fn file(server: &str, token: &str, repo: &str, what: &Filing<'_>) -> anyhow::Result<i64> {
    let mut payload = json!({
        "kind": what.kind,
        "title": what.title,
        "body": what.body,
    });
    for (key, value) in [
        ("version", what.version),
        ("command", what.command),
        ("observed", what.observed),
        ("expected", what.expected),
    ] {
        if let Some(value) = value.filter(|v| !v.trim().is_empty()) {
            payload[key] = json!(value);
        }
    }
    let mut response = agent()
        .post(&format!(
            "{}/api/repos/{repo}/reports",
            server.trim_end_matches('/')
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send_json(&payload)?;
    let status = response.status().as_u16();
    let body: Value = response.body_mut().read_json()?;
    if status != 200 {
        bail!("{}", said(&body, status));
    }
    body["number"]
        .as_i64()
        .context("the forge did not say which number this took")
}

/// The forge's own words for a refusal, which name what would fix it.
fn said(body: &Value, status: u16) -> String {
    body["error"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| format!("the forge answered {status}"))
}

#[cfg(test)]
mod tests {
    use super::parse_remote;

    #[test]
    fn a_clone_url_names_its_forge_and_its_repository() {
        let got = parse_remote("https://ambolt.sh/git/ada/demo").unwrap();
        assert_eq!(got.server, "https://ambolt.sh");
        assert_eq!(got.repo, "ada/demo");
        // The three spellings git actually produces.
        assert_eq!(
            parse_remote("https://ambolt.sh/git/ada/demo.git").unwrap(),
            got
        );
        assert_eq!(
            parse_remote("https://ambolt.sh/git/ada/demo/").unwrap(),
            got
        );
        assert_eq!(
            parse_remote("https://ada@ambolt.sh/git/ada/demo").unwrap(),
            got
        );
    }

    /// A token in the remote is git's way of carrying a credential. It
    /// never becomes part of the address this hands back, because that
    /// address is printed.
    #[test]
    fn a_credential_in_the_remote_is_left_behind() {
        let got = parse_remote("https://ada:ambolt_secret@ambolt.sh/git/ada/demo").unwrap();
        assert_eq!(got.server, "https://ambolt.sh");
        assert!(!format!("{got:?}").contains("secret"));
    }

    #[test]
    fn somebody_elses_forge_is_said_to_be_somebody_elses() {
        for url in [
            "https://github.com/ada/demo.git",
            "git@github.com:ada/demo.git",
            "https://ambolt.sh/ada/demo",
            "https://ambolt.sh/git/demo",
        ] {
            assert!(parse_remote(url).is_err(), "{url} should not parse");
        }
    }
}
