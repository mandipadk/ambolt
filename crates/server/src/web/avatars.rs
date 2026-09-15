//! Marks for people and agents, drawn from the id alone.
//!
//! A person is a Notionists face, composed from Zoish's CC0 parts vendored
//! beside this file, on a tinted disc. An agent is a Lens: an iron housing
//! with one eye whose iris print and pupil are the agent's own, blinking
//! while the agent is at work. Nothing is stored and nothing is uploaded;
//! the same id always draws the same mark, so the pictures are served as
//! immutable files.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::LazyLock;

/// Bumped whenever a drawing changes, so the immutable addresses change
/// with it.
pub const GENERATION: u32 = 1;

const NOTIONISTS: &str = include_str!("notionists.json");

/// Every part by group, in the order the export listed them.
static PARTS: LazyLock<HashMap<String, Vec<(String, String)>>> =
    LazyLock::new(|| serde_json::from_str(NOTIONISTS).expect("notionists parts are well-formed"));

/// FNV-1a, so the same id draws the same mark on every forge.
fn hash(id: &str) -> u32 {
    id.bytes().fold(2166136261u32, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(16777619)
    })
}

/// xorshift32 over the hash: enough to pick parts, and the same
/// sequence the mockups used.
struct Rng(u32);

impl Rng {
    fn next(&mut self) -> f64 {
        let mut x = if self.0 == 0 { 1 } else { self.0 };
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        f64::from(x) / 4294967296.0
    }

    fn pick<'a>(&mut self, parts: &'a [(String, String)]) -> &'a str {
        let i = (self.next() * parts.len() as f64) as usize;
        &parts[i.min(parts.len() - 1)].1
    }

    fn chance(&mut self, percent: u32) -> bool {
        self.next() * 100.0 < f64::from(percent)
    }
}

/// Soft grounds a black line drawing reads on in either theme.
const DISCS: [&str; 6] = [
    "#E8E3D9", "#D9E6DD", "#DCE4EE", "#EADADA", "#E6E1D0", "#DEE0E5",
];

/// A person's mark: a Notionists face on a tinted disc.
pub fn person(id: &str) -> String {
    let h = hash(id);
    let mut r = Rng(h ^ 0x9e37_79b9);
    let parts = &*PARTS;
    let group = |name: &str| parts.get(name).map(Vec::as_slice).unwrap_or(&[]);
    let disc = DISCS[(h % DISCS.len() as u32) as usize];
    // Bodies were exported once per icon they may carry, keyed
    // `variant+icon`; three in four carry one, as the style does.
    let bodies = group("body");
    let with_icon = r.chance(75);
    let plain: Vec<&(String, String)> = bodies.iter().filter(|(n, _)| !n.contains('+')).collect();
    let body = if with_icon {
        let icons = ["electric", "saturn", "galaxy"];
        let icon = icons[(r.next() * 3.0) as usize % 3];
        let n = (r.next() * plain.len() as f64) as usize % plain.len().max(1);
        let key = format!("{}+{icon}", plain[n].0);
        bodies
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, svg)| svg.as_str())
            .unwrap_or("")
    } else {
        let n = (r.next() * plain.len() as f64) as usize % plain.len().max(1);
        plain[n].1.as_str()
    };
    let base = r.pick(group("base"));
    let hair = r.pick(group("hair"));
    let lips = r.pick(group("lips"));
    let beard = if r.chance(10) {
        r.pick(group("beard"))
    } else {
        ""
    };
    let nose = r.pick(group("nose"));
    let eyes = r.pick(group("eyes"));
    let glasses = if r.chance(20) {
        r.pick(group("glasses"))
    } else {
        ""
    };
    let brows = r.pick(group("brows"));
    let gesture = if r.chance(10) {
        r.pick(group("gesture"))
    } else {
        ""
    };
    let mut svg = String::with_capacity(24 * 1024);
    let _ = write!(
        svg,
        concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1744 1744\" fill=\"none\" shape-rendering=\"auto\">",
            "<clipPath id=\"c\"><circle cx=\"872\" cy=\"872\" r=\"872\"/></clipPath>",
            "<circle cx=\"872\" cy=\"872\" r=\"872\" fill=\"{disc}\"/><g clip-path=\"url(#c)\">",
            "<g transform=\"translate(531 487)\">{base}</g>",
            "<g transform=\"translate(178 1057)\">{body}</g>",
            "<g transform=\"translate(266 207)\">{hair}</g>",
            "<g transform=\"translate(791 871)\">{lips}</g>",
            "<g transform=\"translate(653 805)\">{beard}</g>",
            "<g transform=\"translate(901 668)\">{nose}</g>",
            "<g transform=\"translate(610 680)\">{eyes}</g>",
            "<g transform=\"translate(610 680)\">{glasses}</g>",
            "<g transform=\"translate(774 657)\">{brows}</g>",
            "<g transform=\"translate(0 559)\">{gesture}</g>",
            "</g></svg>"
        ),
        disc = disc,
        base = base,
        body = body,
        hair = hair,
        lips = lips,
        beard = beard,
        nose = nose,
        eyes = eyes,
        glasses = glasses,
        brows = brows,
        gesture = gesture,
    );
    svg
}

/// The four pupils: round, tall, wide, a slit.
const PUPILS: [(f64, f64); 4] = [(4.6, 4.6), (3.4, 5.6), (5.8, 3.6), (1.9, 5.8)];

/// An agent's mark: the Lens. One eye on an iron housing; the gaze, the
/// iris print and the pupil are the agent's own. At work, it blinks.
pub fn agent(id: &str, live: bool) -> String {
    let h = hash(id);
    let mut r = Rng(h ^ 0x9e37_79b9);
    let dx = (r.next() - 0.5) * 12.0;
    let dy = (r.next() - 0.5) * 8.0;
    let cx = 32.0 + dx;
    let cy = 33.0 + dy * 0.6;
    let (rx, ry) = PUPILS[(h % 4) as usize];
    let mut ticks = String::new();
    for i in 0..20u32 {
        let bit = (h >> (i % 31)) & 1;
        let bit2 = (h >> ((i * 7) % 31)) & 1;
        if bit == 0 && bit2 == 0 {
            continue;
        }
        let a = f64::from(i) / 20.0 * std::f64::consts::TAU;
        let r1 = 6.2;
        let r2 = if bit == 1 && bit2 == 1 { 9.4 } else { 8.2 };
        let _ = write!(
            ticks,
            "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\"/>",
            cx + a.cos() * r1,
            cy + a.sin() * r1,
            cx + a.cos() * r2,
            cy + a.sin() * r2
        );
    }
    let style = if live {
        concat!(
            "<style>@keyframes blink{0%,92%,100%{transform:scaleY(1)}96%{transform:scaleY(.08)}}",
            ".eye{transform-origin:32px 33px;animation:blink 5s infinite}",
            "@media (prefers-reduced-motion:reduce){.eye{animation:none}}</style>"
        )
    } else {
        ""
    };
    format!(
        concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 64 64\">{style}",
            "<rect width=\"64\" height=\"64\" rx=\"17\" fill=\"#1E222B\"/>",
            "<rect x=\"0.5\" y=\"0.5\" width=\"63\" height=\"63\" rx=\"16.5\" fill=\"none\" stroke=\"#363C48\"/>",
            "<g class=\"eye\"><ellipse cx=\"32\" cy=\"33\" rx=\"17\" ry=\"12\" fill=\"#F1F3F7\"/>",
            "<g stroke=\"#4F7DFF\" stroke-width=\"1.6\" stroke-linecap=\"round\">{ticks}</g>",
            "<ellipse cx=\"{cx:.1}\" cy=\"{cy:.1}\" rx=\"{rx}\" ry=\"{ry}\" fill=\"#0B0D12\"/>",
            "<circle cx=\"{hx:.1}\" cy=\"{hy:.1}\" r=\"1.5\" fill=\"#F1F3F7\"/></g></svg>"
        ),
        style = style,
        ticks = ticks,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        hx = cx + 1.8,
        hy = cy - 2.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_group_has_parts() {
        for group in [
            "base", "body", "hair", "lips", "beard", "nose", "eyes", "glasses", "brows", "gesture",
        ] {
            assert!(!PARTS[group].is_empty(), "{group}");
        }
    }

    #[test]
    fn a_mark_is_the_ids_own() {
        assert_eq!(person("ada"), person("ada"));
        assert_ne!(person("ada"), person("bee"));
        assert_eq!(agent("scout", false), agent("scout", false));
        assert_ne!(agent("scout", false), agent("arbiter", false));
        assert!(agent("scout", true).contains("@keyframes"));
        assert!(!agent("scout", false).contains("@keyframes"));
        let face = person("ada");
        assert!(face.starts_with("<svg xmlns"));
        assert!(face.contains("translate(531 487)"));
    }

    #[test]
    fn faces_vary_across_a_crowd() {
        let faces: std::collections::HashSet<String> =
            (0..40).map(|i| person(&format!("person-{i}"))).collect();
        assert!(faces.len() >= 39, "{}", faces.len());
    }
}
