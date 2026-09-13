//! Code as a reader expects it: highlighted on the server, so a page
//! needs no script for it and carries no inline style. Every token is a
//! span whose classes are the grammar's scope names; the stylesheet maps
//! the few that matter to the token colours of each theme.

use std::sync::LazyLock;

use syntect::parsing::{
    BasicScopeStackOp, ParseState, Scope, ScopeStack, SyntaxDefinition, SyntaxReference, SyntaxSet,
};

/// The grammars that ship with the highlighter, plus TOML, which it
/// lacks and every Rust repository has.
static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(|| {
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    if let Ok(toml) = SyntaxDefinition::load_from_str(TOML, true, Some("TOML")) {
        builder.add(toml);
    }
    builder.build()
});

const TOML: &str = include_str!("toml.sublime-syntax");

/// Past this many bytes a file renders plain: the grammar walk is
/// linear, but the HTML it produces is several times the source, and a
/// page that size is not read anyway.
pub const LIMIT: usize = 512 * 1024;

/// Byte ranges of a line, in order, that a diff changed.
pub type Marks = Vec<(usize, usize)>;

fn syntax_for(path: &str) -> Option<&'static SyntaxReference> {
    let name = path.rsplit('/').next().unwrap_or(path);
    let set = &*SYNTAXES;
    match name {
        "Dockerfile" | "Containerfile" => return set.find_syntax_by_name("Dockerfile"),
        "Makefile" | "GNUmakefile" => return set.find_syntax_by_name("Makefile"),
        "Cargo.lock" => return set.find_syntax_by_extension("toml"),
        _ => {}
    }
    let extension = name.rsplit_once('.').map(|(_, ext)| ext)?;
    let extension = match extension {
        "mjs" | "cjs" | "jsx" | "ts" | "tsx" => "js",
        "yml" => "yaml",
        "zsh" | "bash" => "sh",
        "hbs" | "vue" | "svelte" | "htm" => "html",
        "toml" | "lock" => "toml",
        other => other,
    };
    set.find_syntax_by_extension(extension)
}

/// The language a file is highlighted as, by the name a person would
/// say, or nothing when it renders plain.
pub fn language(path: &str) -> Option<&'static str> {
    if path.ends_with(".ts") || path.ends_with(".tsx") {
        return syntax_for(path).map(|_| "TypeScript");
    }
    syntax_for(path).map(|syntax| match syntax.name.as_str() {
        "Bourne Again Shell (bash)" => "Shell",
        "JavaScript (Babel)" => "JavaScript",
        "Plain Text" => "Text",
        other => other,
    })
}

/// A walk through one file, a line at a time. The grammar's state
/// carries across lines, so a block comment stays a comment on its
/// second line; each line's HTML is still complete in itself, with the
/// spans still open at its end closed and reopened on the next.
pub struct Coder {
    state: Option<(ParseState, ScopeStack)>,
}

impl Coder {
    pub fn for_path(path: &str, bytes: usize) -> Coder {
        let state = (bytes <= LIMIT)
            .then(|| syntax_for(path))
            .flatten()
            .map(|syntax| (ParseState::new(syntax), ScopeStack::new()));
        Coder { state }
    }

    /// Whether the walk highlights anything at all.
    pub fn highlights(&self) -> bool {
        self.state.is_some()
    }

    /// One line, without its newline, as HTML. `marks` are byte ranges
    /// of the line to wrap in `<mark>`: the words a diff changed.
    pub fn line(&mut self, text: &str, marks: &[(usize, usize)]) -> String {
        let Some((state, stack)) = self.state.as_mut() else {
            return marked(text, marks);
        };
        let mut owned = String::with_capacity(text.len() + 1);
        owned.push_str(text);
        owned.push('\n');
        let Ok(ops) = state.parse_line(&owned, &SYNTAXES) else {
            return marked(text, marks);
        };
        let mut html = String::with_capacity(text.len() * 2);
        for scope in stack.as_slice() {
            open(&mut html, *scope);
        }
        let mut at = 0;
        let mut empty_since = None;
        for (index, op) in &ops {
            let index = (*index).min(text.len());
            if index > at {
                write_text(&mut html, &text[at..index], at, marks);
                at = index;
                empty_since = None;
            }
            let hook = |basic: BasicScopeStackOp, _: &[Scope]| match basic {
                BasicScopeStackOp::Push(scope) => {
                    empty_since = Some(html.len());
                    open(&mut html, scope);
                }
                BasicScopeStackOp::Pop => {
                    // A span with nothing in it is dropped rather than
                    // kept as an empty pair.
                    match empty_since.take() {
                        Some(start) => html.truncate(start),
                        None => html.push_str("</span>"),
                    }
                }
            };
            if stack.apply_with_hook(op, hook).is_err() {
                return marked(text, marks);
            }
        }
        if at < text.len() {
            write_text(&mut html, &text[at..], at, marks);
        }
        for _ in stack.as_slice() {
            html.push_str("</span>");
        }
        html
    }
}

fn open(html: &mut String, scope: Scope) {
    html.push_str("<span class=\"");
    let name = scope.build_string();
    html.push_str(&name.replace('.', " "));
    html.push_str("\">");
}

/// Escaped text with the marked ranges wrapped. Marks are given for the
/// whole line; `offset` is where this slice of it begins.
fn write_text(html: &mut String, text: &str, offset: usize, marks: &[(usize, usize)]) {
    let end = offset + text.len();
    let mut at = offset;
    for &(from, to) in marks {
        let from = from.max(offset).min(end);
        let to = to.max(offset).min(end);
        if from >= to {
            continue;
        }
        if from > at {
            escape(html, &text[at - offset..from - offset]);
        }
        html.push_str("<mark>");
        escape(html, &text[from - offset..to - offset]);
        html.push_str("</mark>");
        at = to;
    }
    if at < end {
        escape(html, &text[at - offset..]);
    }
}

fn marked(text: &str, marks: &[(usize, usize)]) -> String {
    let mut html = String::with_capacity(text.len() + 16);
    write_text(&mut html, text, 0, marks);
    html
}

fn escape(html: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => html.push_str("&amp;"),
            '<' => html.push_str("&lt;"),
            '>' => html.push_str("&gt;"),
            '"' => html.push_str("&quot;"),
            other => html.push(other),
        }
    }
}

/// The parts of two lines that differ, as byte ranges into each: the
/// words a change touched, so a diff reads at the word rather than the
/// line. Lines that share nothing get no marks; the whole-line tint
/// already says everything about them.
pub fn marks(old: &str, new: &str) -> (Marks, Marks) {
    let a = tokens(old);
    let b = tokens(new);
    if a.len() > 160 || b.len() > 160 {
        return (Vec::new(), Vec::new());
    }
    // Longest common subsequence over tokens.
    let mut table = vec![vec![0u16; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            table[i][j] = if a[i].1 == b[j].1 {
                table[i + 1][j + 1] + 1
            } else {
                table[i + 1][j].max(table[i][j + 1])
            };
        }
    }
    let common = usize::from(table[0][0]);
    let shared_text: usize = {
        let mut i = 0;
        let mut j = 0;
        let mut shared = 0;
        while i < a.len() && j < b.len() {
            if a[i].1 == b[j].1 {
                shared += a[i].1.trim().len();
                i += 1;
                j += 1;
            } else if table[i + 1][j] >= table[i][j + 1] {
                i += 1;
            } else {
                j += 1;
            }
        }
        shared
    };
    let text_len = old.trim().len().max(new.trim().len());
    if common == 0 || text_len == 0 || shared_text * 4 < text_len {
        return (Vec::new(), Vec::new());
    }
    let mut old_marks = Marks::new();
    let mut new_marks = Marks::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() || j < b.len() {
        if i < a.len() && j < b.len() && a[i].1 == b[j].1 {
            i += 1;
            j += 1;
        } else if j >= b.len() || (i < a.len() && table[i + 1][j] >= table[i][j + 1]) {
            push_range(&mut old_marks, a[i].0, a[i].1.len());
            i += 1;
        } else {
            push_range(&mut new_marks, b[j].0, b[j].1.len());
            j += 1;
        }
    }
    (old_marks, new_marks)
}

fn push_range(ranges: &mut Vec<(usize, usize)>, start: usize, len: usize) {
    match ranges.last_mut() {
        Some((_, end)) if *end == start => *end = start + len,
        _ => ranges.push((start, start + len)),
    }
}

/// Words, runs of blank, and single marks of punctuation, each with
/// where it starts.
fn tokens(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut kind = None;
    for (index, c) in text.char_indices() {
        let this = if c.is_alphanumeric() || c == '_' {
            0
        } else if c.is_whitespace() {
            1
        } else {
            2
        };
        if kind != Some(this) || this == 2 {
            if index > start {
                out.push((start, &text[start..index]));
            }
            start = index;
            kind = Some(this);
        }
    }
    if start < text.len() {
        out.push((start, &text[start..]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rust_line_gets_its_keyword_named() {
        let mut coder = Coder::for_path("src/main.rs", 100);
        let html = coder.line("pub fn main() {}", &[]);
        assert!(html.contains("class=\"storage modifier rust\""), "{html}");
        assert!(html.contains(">pub<"), "{html}");
        assert!(html.contains("main"), "{html}");
        assert_eq!(
            html.matches("<span").count(),
            html.matches("</span>").count()
        );
    }

    #[test]
    fn state_carries_across_lines_and_each_line_closes_itself() {
        let mut coder = Coder::for_path("a.rs", 100);
        let first = coder.line("/* a comment that", &[]);
        let second = coder.line("   goes on */ let x = 1;", &[]);
        assert!(first.contains("comment"), "{first}");
        assert!(
            second.starts_with("<span class=\"source rust\"><span class=\"comment"),
            "{second}"
        );
        for line in [&first, &second] {
            assert_eq!(
                line.matches("<span").count(),
                line.matches("</span>").count()
            );
        }
    }

    #[test]
    fn unknown_and_oversized_files_render_plain_and_escaped() {
        let mut coder = Coder::for_path("notes.unknownext", 10);
        assert!(!coder.highlights());
        assert_eq!(coder.line("a < b & c", &[]), "a &lt; b &amp; c");
        let big = Coder::for_path("big.rs", LIMIT + 1);
        assert!(!big.highlights());
    }

    #[test]
    fn languages_are_named_for_people() {
        assert_eq!(language("x/y.rs"), Some("Rust"));
        assert_eq!(language("Cargo.toml"), Some("TOML"));
        assert_eq!(language("README.md"), Some("Markdown"));
        assert_eq!(language("a.json"), Some("JSON"));
        assert_eq!(language("run.sh"), Some("Shell"));
        assert_eq!(language("ci.yml"), Some("YAML"));
        assert_eq!(language("a.bin"), None);
    }

    #[test]
    fn a_manifest_reads_as_one() {
        let mut coder = Coder::for_path("Cargo.toml", 100);
        let html = coder.line("[package]", &[]);
        assert!(html.contains("entity name section toml"), "{html}");
        let html = coder.line("name = \"ambolt\" # the forge", &[]);
        assert!(html.contains("entity name tag toml"), "{html}");
        assert!(html.contains("string quoted double toml"), "{html}");
        assert!(html.contains("comment line"), "{html}");
        let html = coder.line("edition = 2024", &[]);
        assert!(html.contains("constant numeric toml"), "{html}");
    }

    #[test]
    fn marks_find_the_word_that_changed() {
        let (old, new) = marks("let limit = 2 << 20;", "let limit = 4 << 20;");
        assert_eq!(old, vec![(12, 13)]);
        assert_eq!(new, vec![(12, 13)]);
        let mut coder = Coder::for_path("a.rs", 100);
        let html = coder.line("let limit = 4 << 20;", &new);
        assert!(html.contains("<mark>4</mark>"), "{html}");
    }

    #[test]
    fn lines_that_share_nothing_get_no_marks() {
        let (old, new) = marks("fn alpha() {", "    return None;");
        assert!(old.is_empty() && new.is_empty());
        let (old, new) = marks("", "new line");
        assert!(old.is_empty() && new.is_empty());
    }
}
