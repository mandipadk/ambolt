//! What a person's page counts beyond the record: what they decided on
//! other people's changes, where what landed in their name touched the
//! tree, and how much of it was an agent's rather than their own.
//! Nothing here is stored; each figure is a query over the projections,
//! cut at a window the way the record is.

use crate::error::CoreResult;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// What somebody decided about other people's changes over a window:
/// the looks they gave, what they said, whether a block held, the
/// attempts they chose between, the questions they answered, and how
/// many of the looks the attention budget asked of them they gave.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Judgement {
    pub window_days: u32,
    pub since: String,
    /// Verdicts on changes that are not their own.
    pub looks: u32,
    /// On how many distinct changes.
    pub changes: u32,
    pub approved: u32,
    pub concerns: u32,
    pub blocked: u32,
    /// Blocks followed by a later revision or an abandonment: the block
    /// changed what happened.
    pub held: u32,
    /// Revisions they preferred among competing attempts.
    pub compared: u32,
    /// Questions they settled as answered.
    pub answered: u32,
    /// Draws that named them as a reviewer, and those a verdict of
    /// theirs followed.
    pub asked: u32,
    pub asked_answered: u32,
}

/// How many landings were the person's own, and how many were an agent's
/// in their name. Ownership is recorded, so this is observed rather than
/// declared.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Split {
    pub in_person: u32,
    pub by_agents: u32,
}

/// The first event at or after a window's start, and the start itself.
/// With nothing since, the cut is past the end and nothing counts.
fn window(conn: &Connection, days: u32) -> CoreResult<(i64, String)> {
    let days = days.max(1);
    let since = (jiff::Timestamp::now() - jiff::SignedDuration::from_hours(24 * i64::from(days)))
        .to_string();
    let since_seq: i64 = conn.query_row(
        "SELECT COALESCE((SELECT MIN(seq) FROM events WHERE ts >= ?1),
                         (SELECT COALESCE(MAX(seq), 0) + 1 FROM events))",
        params![since],
        |row| row.get(0),
    )?;
    Ok((since_seq, since))
}

pub(crate) fn judgement_of(conn: &Connection, who: &str, days: u32) -> CoreResult<Judgement> {
    let (since_seq, since) = window(conn, days)?;
    let mut out = Judgement {
        window_days: days.max(1),
        since,
        ..Default::default()
    };
    let verdicts: Vec<(String, String, i64, String, i64)> = conn
        .prepare_cached(
            "SELECT v.change_id, v.disposition, v.revision, c.state, c.latest_revision
               FROM verdicts v JOIN changes c ON c.id = v.change_id
              WHERE v.by = ?1 AND c.owner != ?1 AND v.seq >= ?2",
        )?
        .query_map(params![who, since_seq], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut changes = BTreeSet::new();
    for (change, disposition, revision, state, latest) in verdicts {
        out.looks += 1;
        changes.insert(change);
        match disposition.as_str() {
            "approve" => out.approved += 1,
            "concern" => out.concerns += 1,
            "block" => {
                out.blocked += 1;
                if latest > revision || state == "abandoned" {
                    out.held += 1;
                }
            }
            _ => {}
        }
    }
    out.changes = changes.len() as u32;
    out.compared = conn.query_row(
        "SELECT COUNT(*) FROM events WHERE kind = 'revision_preferred' AND actor = ?1 AND seq >= ?2",
        params![who, since_seq],
        |row| row.get::<_, i64>(0),
    )? as u32;
    out.answered = conn.query_row(
        "SELECT COUNT(*) FROM origins
          WHERE settled_by = ?1 AND settled_how = 'answered' AND settled_at >= ?2",
        params![who, out.since],
        |row| row.get::<_, i64>(0),
    )? as u32;
    let draws: Vec<(String, i64)> = conn
        .prepare_cached(
            "SELECT change_id, seq FROM attention_draws
              WHERE seq >= ?2
                AND EXISTS (SELECT 1 FROM json_each(attention_draws.reviewers) WHERE value = ?1)",
        )?
        .query_map(params![who, since_seq], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (change, seq) in draws {
        out.asked += 1;
        let gave: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM verdicts WHERE change_id = ?1 AND by = ?2 AND seq > ?3)",
            params![change, who, seq],
            |row| row.get(0),
        )?;
        if gave {
            out.asked_answered += 1;
        }
    }
    Ok(out)
}

/// Whether a runner re-ran the revision that landed and agreed with what
/// was claimed on it.
pub(crate) fn reproduced_on(conn: &Connection, change: &str, revision: i64) -> CoreResult<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM verifications WHERE change_id = ?1 AND revision = ?2 AND agrees = 1)",
        params![change, revision],
        |row| row.get(0),
    )?)
}

/// The directories what landed in somebody's name touched, most first:
/// every path of every landed revision since a moment, grouped by the
/// directory holding the file, at most four segments deep. Files at the
/// top of the tree group under their own name. Returns the groups and
/// the total number of paths.
pub(crate) fn landed_tree(
    conn: &Connection,
    who: &str,
    since: &str,
) -> CoreResult<(Vec<(String, u32)>, u32)> {
    let paths: Vec<String> = conn
        .prepare_cached(
            "SELECT r.paths FROM changes c
               JOIN revisions r ON r.change_id = c.id
                AND r.number = COALESCE(c.landed_revision, c.latest_revision)
              WHERE c.owner = ?1 AND c.state = 'merged' AND c.updated_at >= ?2",
        )?
        .query_map(params![who, since], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    let mut groups: BTreeMap<String, u32> = BTreeMap::new();
    let mut total = 0u32;
    for list in paths {
        let list: Vec<String> = serde_json::from_str(&list).unwrap_or_default();
        for path in list {
            total += 1;
            let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            let group = match segments.len() {
                0 => continue,
                1 => segments[0].to_owned(),
                n => segments[..(n - 1).min(4)].join("/"),
            };
            *groups.entry(group).or_default() += 1;
        }
    }
    let mut groups: Vec<(String, u32)> = groups.into_iter().collect();
    groups.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    Ok((groups, total))
}

/// Landings by the person and by the agents they hold.
pub(crate) fn split_of(conn: &Connection, who: &str) -> CoreResult<Split> {
    let in_person: i64 = conn.query_row(
        "SELECT COUNT(*) FROM changes WHERE owner = ?1 AND state = 'merged'",
        params![who],
        |row| row.get(0),
    )?;
    let by_agents: i64 = conn.query_row(
        "SELECT COUNT(*) FROM changes
          WHERE state = 'merged'
            AND owner IN (SELECT id FROM principals WHERE kind = 'agent' AND owner = ?1)",
        params![who],
        |row| row.get(0),
    )?;
    Ok(Split {
        in_person: in_person as u32,
        by_agents: by_agents as u32,
    })
}
