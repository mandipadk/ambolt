//! `@name` in prose: who a piece of discussion calls on.
//!
//! A mention is an `@` followed by something shaped like a principal
//! id, standing on its own: not inside an address (`ada@example.org`)
//! and not glued to a word. Whether the name is anyone is the caller's
//! to check; this only reads.

/// Every `@name` in `text`, in order, as written. Duplicates stay,
/// since the caller decides what to do with them.
pub fn in_text(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'@' {
            let stands_alone = i == 0 || !is_word(bytes[i - 1]);
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && is_slug(bytes[end]) {
                end += 1;
            }
            // A trailing hyphen is punctuation, not part of a name.
            while end > start && bytes[end - 1] == b'-' {
                end -= 1;
            }
            if stands_alone && end > start && end - start <= 64 {
                found.push(&text[start..end]);
            }
            i = end.max(i + 1);
        } else {
            i += 1;
        }
    }
    found
}

fn is_slug(b: u8) -> bool {
    b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'
}

fn is_word(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'.'
}

#[cfg(test)]
mod tests {
    use super::in_text;

    #[test]
    fn names_are_read_and_addresses_are_not() {
        assert_eq!(
            in_text("ping @ada and @scout-2, thanks"),
            vec!["ada", "scout-2"]
        );
        assert_eq!(in_text("write to ada@example.org"), Vec::<&str>::new());
        assert_eq!(in_text("@ada"), vec!["ada"]);
        assert_eq!(in_text("(@ada) @Ada @"), vec!["ada"]);
        assert_eq!(in_text("see @ada-"), vec!["ada"]);
    }
}
