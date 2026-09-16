//! The stylesheet is one file that every page shares, so a selector written
//! twice is a rule that silently loses to its later twin. The forms once
//! vanished that way: `.stack` named a form column and, further down, a
//! six-pixel bar. Every rule is declared once.

const STYLE: &str = include_str!("../../../server/src/web/style.css");
const VIEWS: &str = include_str!("../../../server/src/web/views.rs");

/// Labels read in sentence case; nothing on a page is set in capitals.
#[test]
fn no_label_is_set_in_capitals() {
    assert!(
        !STYLE.contains("text-transform: uppercase"),
        "a rule sets text in capitals; labels read in sentence case here"
    );
}

/// Two facts are two lines or two columns, never one line with a dot
/// between them.
#[test]
fn no_fact_is_joined_to_another_with_a_dot() {
    let joins: Vec<usize> = VIEWS
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(" · "))
        .map(|(i, _)| i + 1)
        .collect();
    assert!(
        joins.is_empty(),
        "facts joined with a dot at views.rs lines {joins:?}; give each its own line or column"
    );
}

/// Top-level selectors and the at-rule scope they sit in, one entry per
/// selector in a comma-separated list. Declarations are skipped, comments
/// are ignored, and at-rules (`@media`, `@keyframes`) open a scope of their
/// own so a narrow-screen override of `.trow` is not a duplicate of `.trow`.
fn selectors(css: &str) -> Vec<(Vec<String>, String)> {
    let mut out = Vec::new();
    let mut scope: Vec<String> = Vec::new();
    let mut head = String::new();
    let mut chars = css.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '/' && css[i..].starts_with("/*") {
            let end = css[i..].find("*/").map(|n| i + n + 2).unwrap_or(css.len());
            while let Some(&(j, _)) = chars.peek() {
                if j >= end {
                    break;
                }
                chars.next();
            }
            continue;
        }
        match c {
            '{' => {
                let text = head.split_whitespace().collect::<Vec<_>>().join(" ");
                head.clear();
                if text.starts_with('@') {
                    scope.push(text);
                    continue;
                }
                for sel in text.split(',') {
                    let sel = sel.split_whitespace().collect::<Vec<_>>().join(" ");
                    if !sel.is_empty() {
                        out.push((scope.clone(), sel));
                    }
                }
                // Declarations never contain a brace; skip to the rule's end.
                for (_, d) in chars.by_ref() {
                    if d == '}' {
                        break;
                    }
                }
            }
            '}' => {
                scope.pop();
                head.clear();
            }
            _ => head.push(c),
        }
    }
    out
}

#[test]
fn every_selector_is_declared_once() {
    let mut seen = std::collections::HashSet::new();
    let mut twice = Vec::new();
    for (scope, sel) in selectors(STYLE) {
        if !seen.insert((scope.clone(), sel.clone())) {
            twice.push(match scope.last() {
                Some(at) => format!("{sel} (inside {at})"),
                None => sel,
            });
        }
    }
    assert!(
        twice.is_empty(),
        "declared more than once, so one copy silently loses: {}",
        twice.join(", ")
    );
}

/// A comment's tail left behind by an edit reads as a selector, and the
/// browser drops that rule and the one after it without a word. No
/// selector carries the end of a comment or a full stop followed by a
/// space, which prose has and selectors never do.
#[test]
fn no_rule_is_swallowed_by_stray_text() {
    // A font face's source is a placeholder the server fills in, and it
    // reads as a block to this parser; it is not a rule.
    let stray: Vec<String> = selectors(STYLE)
        .into_iter()
        .filter(|(scope, _)| !scope.iter().any(|at| at.starts_with("@font-face")))
        .map(|(_, sel)| sel)
        .filter(|sel| sel.contains("*/") || sel.contains(". ") || sel.split(' ').count() > 8)
        .collect();
    assert!(
        stray.is_empty(),
        "text outside any rule: {}",
        stray.join(" | ")
    );
}

/// Every `var(--x)` the stylesheet reads is declared on `:root`, so no
/// token exists only inside a media or theme block and silently falls
/// back to nothing in another state.
#[test]
fn every_token_read_is_declared_on_root() {
    let root_start = STYLE.find(":root {").expect("a :root block");
    let root_end = STYLE[root_start..].find('}').unwrap() + root_start;
    let root = &STYLE[root_start..root_end];
    let mut missing = std::collections::BTreeSet::new();
    for piece in STYLE.split("var(--").skip(1) {
        let name: String = piece
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        if !root.contains(&format!("--{name}:")) {
            missing.insert(name);
        }
    }
    assert!(
        missing.is_empty(),
        "read but not declared on :root: {}",
        missing.into_iter().collect::<Vec<_>>().join(", ")
    );
}

#[test]
fn the_parser_sees_scopes_and_lists() {
    let css = "/* a { */ .a, .b { color: red } @media (x) { .a { color: blue } } .c{}";
    let got = selectors(css);
    assert_eq!(got[0], (vec![], ".a".to_string()));
    assert_eq!(got[1], (vec![], ".b".to_string()));
    assert_eq!(got[2], (vec!["@media (x)".to_string()], ".a".to_string()));
    assert_eq!(got[3], (vec![], ".c".to_string()));
}
