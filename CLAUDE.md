# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`w2m` is a Rust CLI that downloads a web page (static or JS-rendered) and converts it to Markdown with locally-downloaded images. Single binary, async (Tokio), uses headless Chrome only as a fallback.

## Common commands

```bash
cargo build --release            # release binary at target/release/w2m
cargo test                       # unit + integration tests (no Chrome needed)
cargo test --lib                 # unit tests only (per-module #[cfg(test)] blocks)
cargo test --test integration_static
cargo test <name>                # filter by test name (e.g. `detects_empty_spa`)
cargo run -- <URL> [flags]       # run against a real URL
```

The integration test (`tests/integration_static.rs`) uses `wiremock` for HTTP and forces `no_render: true`, so it never invokes Chrome. CI is release-only — there is no test workflow.

Several unit tests read fixtures via the relative path `tests/fixtures/...`, so they must be run from the crate root (Cargo's default).

## Architecture

`main.rs` is just argv parsing, config loading, and exit-code mapping. All real work lives in `pipeline::run`, which orchestrates the fixed module sequence:

```
fetcher (static HTTP) ─┐
                       ├─→ extractor ─→ converter ─→ assets ─→ output
renderer (headless) ───┘                  ↓                       ↓
                                       AssetMap                index.md + assets/
```

- **`fetcher`** — `reqwest` GET with a 30s timeout and `w2m/<version>` UA.
- **`renderer`** — `chromiumoxide`-driven headless Chrome. Honors `CHROME_PATH`; navigates, sleeps `settle` ms, returns `page.content()`. The browser handler runs as a spawned task that's aborted on completion.
- **`pipeline::looks_like_empty_spa`** — the heuristic that decides whether to fall back to headless after a static fetch. Triggers on body text < 200 chars, OR < 500 chars when an `id="root"` / `id="app"` / `id="__next"` mount is present. Tested against fixtures in `tests/fixtures/`.
- **`extractor`** — Two paths: a `--selector` CSS override, or `readability::extractor::extract` for the default heuristic. Both can return `ExtractionEmpty`, which the pipeline catches to retry under headless (when neither `--render` nor `--no-render` is set).
- **`converter`** — `htmd` does HTML→Markdown. Before conversion, `rewrite_image_srcs` runs the HTML through `lol_html` and rewrites the `src` attribute of every real `<img>` element to its local path from the `AssetMap`. The earlier implementation did a global string `.replace()`, which could clobber substrings inside `<code>` blocks or unrelated attributes that happened to share a URL — `lol_html` scopes the change to actual element attributes.
- **`assets`** — Concurrent downloads via `futures::stream::buffer_unordered(concurrency)`. Failures are logged and skipped, never propagated; the returned `AssetMap` only includes successes. Filenames come from the URL's last path segment, sanitized and prefixed with the index for uniqueness.
- **`output`** — Writes `index.md` with YAML frontmatter (`title`, `source_url`, `fetched_at`, `render_mode`). Refuses to overwrite an existing `index.md` (`OutputExists` error), but tolerates a pre-existing `assets/` subdir (the pipeline creates it first).

### Render strategy

`pipeline::Opts::render` is a `RenderStrategy` enum (`Auto | Force | Disabled`), not two booleans. The previous `force_render: bool + no_render: bool` pair could end up both-true after CLI/config merging — host config `render = true` plus CLI `--no-render` silently picked headless. `main::resolve_render` now collapses the four signals (CLI `--render`, CLI `--no-render`, host `render`, host `no_render`) into one variant with explicit "CLI always wins" priority.

Two static→headless fallback triggers, both gated by `RenderStrategy::Auto`:

1. Static HTML looks like an empty SPA shell (`looks_like_empty_spa`).
2. Static extraction returned `ExtractionEmpty`.

When `RenderStrategy::Disabled` is paired with an empty SPA shell *and* extraction fails, the pipeline returns `W2mError::EmptySpaWithoutRender` (exit 4) instead of the generic `ExtractionEmpty` "try --selector" message — that hint doesn't help on an empty shell.

Both fallback paths call `timed_render` and accumulate into `total_render_time`, surfaced in the final summary.

### Output precheck

`output::precheck(dir)` runs at the very top of `pipeline::run`, before any HTTP fetch or filesystem write. It rejects the run if `dir/index.md` already exists. This stops a multi-MB image download from happening just to fail at the final write. `output::write_bundle` keeps its own check as defense in depth.

### Config + flag resolution

`config.rs` loads `~/.config/w2m/config.toml` (preferred) or `dirs::config_dir()` fallback. `Config::rules_for(host)` merges `[defaults]` with `[hosts."<host>"]` (host wins per-field). `main::resolve_opts` then layers CLI flags on top:

```
CLI flag > host rule > defaults > hardcoded constant
```

The hardcoded constants (`DEFAULT_CONCURRENCY = 8`, `DEFAULT_WAIT_MS = 2000`) live only in `main.rs`. When adding a new tunable, thread it through `cli.rs` → `config.rs::HostRules` → `pipeline::Opts` and add a final `unwrap_or` in `resolve_opts`.

### Error → exit code

`W2mError::exit_code()` is the single source of truth: `Http=2`, `ChromeNotFound|Render=3`, `ExtractionEmpty|ExtractionFailed=4`, everything else `=1`. `main` reads this directly. New error variants should pick a category here, not a new code.

## Release & distribution

Tagging `v*` triggers `.github/workflows/release.yml`, which cross-builds for `aarch64-apple-darwin` and `x86_64-unknown-linux-gnu` and attaches tarballs + `.sha256` to a GitHub Release. The Homebrew formula in `packaging/homebrew/w2m.rb` is hand-copied into the sibling `homebrew-w2m` tap repo per release; see `packaging/homebrew/README.md`. Bumping a release version touches `Cargo.toml`, the formula `version`/sha256s, and the tag.
