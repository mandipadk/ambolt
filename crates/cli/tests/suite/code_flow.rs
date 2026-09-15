//! Code looks like code: a file, a diff and a README each render in
//! their own frame, highlighted on the server, with nothing inline and
//! no script needed.

use crate::common::*;

#[tokio::test(flavor = "multi_thread")]
async fn a_file_a_diff_and_a_readme_are_highlighted_on_the_server() {
    let forge = boot().await;
    let app = &forge.app;
    git(
        &forge.work,
        &[
            "clone",
            &format!("http://scout:x@{}/git/ada/demo", forge.addr),
            "wc",
        ],
    );
    let wc = forge.work.join("wc");
    std::fs::create_dir_all(wc.join("src")).unwrap();
    std::fs::write(
        wc.join("README.md"),
        "# Demo\n\nA paragraph.\n\n```rust\nfn main() {}\n```\n",
    )
    .unwrap();
    std::fs::write(wc.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
    commit_file(
        &wc,
        "src/lib.rs",
        "/// Two.\npub const LIMIT: usize = 2 << 20;\n",
        "Add the crate\n\nChange-Id: Icode01",
    );
    git(&wc, &["push", "origin", "HEAD:refs/for/main"]);
    let (_, change) = api(app, "GET", "/api/repos/ada/demo/changes/1", "ada", None).await;
    let id = change["id"].as_str().unwrap().to_owned();
    approve_and_merge(app, &id).await;

    let (_, ada) = sign_in_as(&forge, "ada").await;
    // The file: a code frame, the language named, every token a span
    // with the grammar's names and nothing inline.
    let (_, page) = page_with_cookie(app, "/ada/demo/tree/src/lib.rs", &ada).await;
    assert!(page.contains(r#"class="code""#), "{page}");
    assert!(
        page.contains(r#"<span class="k">Lines</span><span class="v">2</span>"#),
        "{page}"
    );
    assert!(
        page.contains(r#"<span class="k">Language</span><span class="v">Rust</span>"#),
        "{page}"
    );
    assert!(
        page.contains(r#"<span class="hl-storage hl-modifier hl-rust">pub</span>"#),
        "{page}"
    );
    assert!(page.contains(r#"class="hl-comment"#), "{page}");
    assert!(!page.contains("style="), "{page}");
    let (_, toml) = page_with_cookie(app, "/ada/demo/tree/Cargo.toml", &ada).await;
    assert!(
        toml.contains("TOML") && toml.contains("hl-entity hl-name hl-section hl-toml"),
        "{toml}"
    );

    // The README: a panel of prose whose fenced code is highlighted too.
    let (_, repo) = page_with_cookie(app, "/ada/demo", &ada).await;
    assert!(repo.contains(r#"class="panel readme""#), "{repo}");
    assert!(repo.contains(r#"<h1>Demo</h1>"#), "{repo}");
    assert!(
        repo.contains(r#"<pre><code class="src"><span class="hl-source hl-rust">"#),
        "{repo}"
    );

    // A second change edits one word; its diff marks that word on both
    // sides and heads the hunk with a line range rather than an index.
    commit_file(
        &wc,
        "src/lib.rs",
        "/// Two.\npub const LIMIT: usize = 4 << 20;\n",
        "Raise the limit\n\nChange-Id: Icode02",
    );
    git(&wc, &["push", "origin", "HEAD:refs/for/main"]);
    let (_, page) = page_with_cookie(app, "/ada/demo/changes/2", &ada).await;
    assert!(page.contains(r#"class="diff""#), "{page}");
    assert!(page.contains(r#"class="plus">+1</span>"#), "{page}");
    assert!(page.contains(r#"class="minus">−1</span>"#), "{page}");
    assert!(page.contains(r#"class="hunk-head">lines 1–2"#), "{page}");
    assert!(
        page.contains("<mark>2</mark>") && page.contains("<mark>4</mark>"),
        "{page}"
    );
    assert!(!page.contains("@@ "), "{page}");

    // Blame renders in the same frame.
    let (_, blame) = page_with_cookie(app, "/ada/demo/blame/src/lib.rs", &ada).await;
    assert!(blame.contains(r#"class="code blame""#), "{blame}");
    assert!(
        blame.contains(r#"<span class="hl-storage hl-modifier hl-rust">pub</span>"#),
        "{blame}"
    );
}
