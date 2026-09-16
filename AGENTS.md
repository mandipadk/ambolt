# Working on Ambolt

Ambolt is a git forge that records how software came to exist. It is a
Rust workspace: `ambolt-core` (event log, projections, policy),
`ambolt-git` (bare repositories, the wire protocol), `ambolt-server`
(API, git over HTTP, the pages), `ambolt-client` (the MCP adapter and
the claim runner), and the `ambolt` binary in `crates/cli`.

Read `docs/architecture.md` before changing the model. `CONTRIBUTING.md`
covers sign-off and licensing, which apply to every change.

## Reporting, requests and questions

This repository lives on Ambolt: https://ambolt.sh/ambolt/ambolt
Issues on the GitHub mirror are read, but this is where work happens.

Something is broken:

```
ambolt report bug --title "..." --body "..." \
  --version <what you ran> --command <the command that shows it> \
  --observed <what happened> --expected <what should have>
```

Run it inside a clone and it finds the forge and the repository from the
git remote. A report carrying the command gets re-run by a runner and
reaches a maintainer with evidence behind it; one without waits for
somebody to add the command. File in prose anyway if that is all you
have — anyone can sharpen it afterwards, including you, later.

Something is missing: `ambolt report request`, saying what you were
trying to do and what you do instead today. How something works:
`ambolt report question`.

`ambolt guide` prints what a repository takes and from whom, and needs
no token.

## Changes

Branches move only by merge; a direct push is refused with the reason.

```sh
git commit -s -m $'What this does\n\nChange-Id: I<hex>'
git push origin HEAD:refs/for/main
```

Every commit needs a `Change-Id:` trailer — pushing again with the same
one makes revision 2 of the same change rather than a second change. A
commit that does work for a task carries `Task: t-...` as well. Sign off
with `-s`; unsigned commits are not landed.

Then attach a claim naming the command that checked it, and say what it
did not check. A claim that nobody can re-run is not evidence here.

## The gate, before you push

Run all of it. Nothing lands that has not been through this:

```sh
cargo fmt --all
cargo clippy -j 2 --workspace --all-targets -- -D warnings
GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 \
  cargo test -j 2 --workspace --no-fail-fast
cargo fmt --all --check      # last, and again after any further edit
```

The git configuration is emptied on purpose: the suite drives real git,
and a developer's global configuration has broken it before.
`--no-fail-fast` is not optional — cargo otherwise stops at the first
failing crate and you will report a pass that never ran.

The first `fmt` rewrites and says nothing; the last one is what CI runs,
and it fails. Anything you edit after the first line — a rename while
reading the diff, a secret swapped for a placeholder at commit time —
leaves the tree unformatted with no local signal at all, and the mirror
goes red minutes after the change has already landed. End on `--check`.

## Things this codebase will not forgive

**Projections say only what the log says.** Every projection table is
the log applied. Never write one from anywhere but an apply arm, and
never invent a side effect there that no event carries;
`ambolt admin fsck` replays the log into a shadow database and compares,
so an invented write is found, but only after it has already lied to
somebody.

**A new projection table means a new `SCHEMA_VERSION`.** The schema batch
runs only when the version changes, so a table added without the bump
simply never exists on a running forge. This has taken production down
once. Three tests pin the shapes — the projection schema's digest, the
stored policy's JSON, the stored quota's JSON — and each failure message
says to bump the version and repin.

**Events are append-only and additive.** Once an event kind ships, its
payload may gain optional fields and nothing else. There are no
migrations here; a schema change is a replay.

**Authority is checked in the core, never at the edge.** Every command
calls `authorize` before it does anything, and a refusal names the
missing capability and the grant that would fix it. Routes contain no
domain logic.

**The pages run no script.** Content-Security-Policy forbids inline
styles, so a `style="..."` attribute silently does nothing — put it in
`crates/server/src/web/style.css`.

**Caps on anything a caller writes.** An append-only log keeps whatever
it is given for good: `bounded()` every free-text field.

## Conventions

- Comments state constraints the code cannot show. Not what the next
  line does, and never an argument that the change is correct.
- Tests exercise behaviour, not implementation, and read as sentences:
  `a_mention_reaches_who_could_read_it_and_links_to_them`.
- British spelling in prose and in identifiers (`organisation`).
- Errors say what would fix them, addressed to whoever reads the log.
- `scripts/first-run.sh` walks what the README claims, against an empty
  forge, in CI. If you change what the documents promise, change it too.
