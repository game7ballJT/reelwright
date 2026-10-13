# What this fork changes

Reelwright is a fork of [FilmCraft](https://github.com/storytold/filmcraft) (upstream commit
`5231852`, version 0.4.0, 2026-10-08). The upstream history is preserved in this repository, so
`git log` shows who wrote what; this page lists what the fork changed on top of it.

## Rebrand

- Every crate, binary, app ID, string, document and test is renamed: `filmcraft-*` is
  `reelwright-*`, the app ID is `online.jonmcgee.reelwright`, the project extension is still
  `.fcproj` so existing projects open unchanged.
- The upstream brand is removed: `docs/brand/` (its logos are not open source), the Discord and
  website links, the family table. The Help menu links to this repository and, as the credit the
  fork owes, to the upstream project. A new, original app icon (a film reel) is drawn in code
  (`packaging/icons.ps1`).
- Legal files kept and corrected: `LICENSE-MIT` names the original copyright holders,
  `LICENSE-APACHE` is untouched, `NOTICE` and `ATTRIBUTION.md` say what is upstream and what is new.
  The README screenshots are the upstream project's own; they are credited to its contributors in
  `ATTRIBUTION.md` and their sidecars.

## New features

| Feature | Command | Notes |
|---|---|---|
| Edit Statistics | `sequence.editStats` | Sequence › Edit Statistics… reports shots, cuts, average and median shot length, cuts per minute, longest and shortest shot and a shot-length histogram (`crates/engine/src/sequence_insight.rs`). Overlapping clips that start together count as one cut |
| Export Chapter List | `markers.exportChapters` | Markers › Export Chapter List… writes the Chapter markers as a YouTube-style list or JSON, and warns about what a video platform rejects (no 0:00 chapter, fewer than three chapters, chapters under ten seconds) |
| Check Caption Readability | `captions.checkReadability` | Sequence › Captions › Check Readability… checks reading speed, line length, lines per caption, on-screen time and gaps for one track or all; limits are parameters; `select: true` selects the captions with a problem; the report dialog jumps the playhead to a caption (`crates/engine/src/caption_check.rs`) |
| Project links | `help.website`, `help.github`, `help.upstream`, `help.reportIssue` | Replace the upstream community links |
| Web build size | `Cargo.toml`, `cargo xtask web` | A `wasm-release` profile (opt-level `s`, fat LTO) keeps the browser build small |

Everything above is a registered command (so it is in the menus, the command search, the control
channel, `reelwright-cli` and MCP) and has unit tests; the two reports also have a UI test.
