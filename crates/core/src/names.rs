//! What a name may be. One policy for people, organisations,
//! repositories and teams: the forge's own words, shapes that pass for
//! the forge, and a short list of words nobody gets to be called here. A
//! refusal on those says only that the name is not available, so the
//! lists are not a game; a page's name says why, since an honest person
//! picking again is helped by knowing.

use crate::error::{CoreError, CoreResult};

/// Words the forge keeps for itself: an account or a repository under
/// them would read as the forge speaking.
const FORGE_WORDS: &[&str] = &[
    "ambolt",
    "admin",
    "administrator",
    "support",
    "security",
    "staff",
    "official",
    "help",
    "billing",
    "noreply",
    "no-reply",
    "root",
    "system",
    "moderator",
    "abuse",
    "postmaster",
    "webmaster",
    "hostmaster",
    "operator",
    "ops",
    "status",
    "legal",
    "press",
];

/// Shapes that pass for the forge or its people.
const FORGE_PREFIXES: &[&str] = &["ambolt-"];
const FORGE_SUFFIXES: &[&str] = &["-official", "-staff", "-admin", "-support", "-team"];

/// Words nobody gets to be called, whole or as the start or end of a
/// name, after the spellings that dodge a list are undone.
const REFUSED_ANYWHERE: &[&str] = &[
    "nigger",
    "nigga",
    "faggot",
    "wetback",
    "tranny",
    "retard",
    "raghead",
    "towelhead",
    "beaner",
];
/// The same, matched whole only: too short to be safe as a fragment.
const REFUSED_WHOLE: &[&str] = &["kike", "spic", "chink", "gook", "coon", "paki", "fag"];

/// The name with the usual dodges undone: hyphens dropped, the digits
/// and signs that stand for letters put back.
fn plain(name: &str) -> String {
    name.chars()
        .filter(|c| *c != '-')
        .map(|c| match c {
            '0' => 'o',
            '1' => 'i',
            '3' => 'e',
            '4' => 'a',
            '5' => 's',
            '7' => 't',
            '8' => 'b',
            '@' => 'a',
            '$' => 's',
            other => other.to_ascii_lowercase(),
        })
        .collect()
}

/// Whether `name` may be taken. `by_operator` lets whoever runs the forge
/// take the forge's own words on purpose; nothing lets anyone take the
/// rest.
pub fn acceptable(name: &str, by_operator: bool) -> CoreResult<()> {
    let lower = name.to_ascii_lowercase();
    let flat = plain(&lower);
    let refused = REFUSED_WHOLE.iter().any(|w| flat == *w)
        || REFUSED_ANYWHERE
            .iter()
            .any(|w| flat == *w || flat.starts_with(w) || flat.ends_with(w));
    if refused {
        return Err(CoreError::Invalid(NOT_AVAILABLE.to_owned()));
    }
    if by_operator {
        return Ok(());
    }
    let forge = FORGE_WORDS.contains(&lower.as_str())
        || FORGE_PREFIXES.iter().any(|p| lower.starts_with(p))
        || FORGE_SUFFIXES.iter().any(|s| lower.ends_with(s));
    if forge {
        return Err(CoreError::Invalid(NOT_AVAILABLE.to_owned()));
    }
    Ok(())
}

/// The one thing a refusal says.
pub const NOT_AVAILABLE: &str = "that name is not available";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_forges_words_and_shapes_are_kept_unless_the_operator_takes_them() {
        for name in ["support", "ambolt-help", "acme-official", "Admin"] {
            assert!(acceptable(name, false).is_err(), "{name}");
            assert!(acceptable(name, true).is_ok(), "{name} for the operator");
        }
        for name in ["ada", "acme", "supporting-cast", "helpful", "ops-team-x"] {
            assert!(acceptable(name, false).is_ok(), "{name}");
        }
    }

    #[test]
    fn a_word_nobody_gets_to_be_called_is_refused_however_it_is_spelt() {
        for name in ["n1gga", "f4gg0t", "re-tard", "fag", "ret4rds"] {
            let err = acceptable(name, true).unwrap_err();
            assert!(err.to_string().contains(NOT_AVAILABLE), "{name}: {err}");
        }
        // Short words are refused whole, not as fragments of longer ones.
        for name in ["tycoon", "cocoon", "spice", "gookaloo", "chinkara"] {
            assert!(acceptable(name, false).is_ok(), "{name}");
        }
    }
}
