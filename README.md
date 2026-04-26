# w2m

Download a web page (including JS-rendered SPAs) and convert it to Markdown,
downloading inline images alongside.

## Install

`--render` requires a local Chrome / Chromium. Set `CHROME_PATH=/path/to/chrome`
if it isn't auto-detected.

### Homebrew (macOS, Linux)

```bash
brew install procmeans/w2m/w2m
```

### From source (any platform with Rust 1.74+)

```bash
cargo install --git https://github.com/procmeans/w2m
```

Or, in a clone of this repo:

```bash
cargo build --release
# binary at target/release/w2m
```

## Usage

```bash
w2m <URL> [OPTIONS]
```

Flags:

| Flag | Default | Description |
|---|---|---|
| `-o, --out <DIR>` | derived from URL | Output directory |
| `--render` | off | Force headless rendering |
| `--no-render` | off | Disable headless fallback |
| `--selector <CSS>` | none | Override readability with a CSS selector |
| `--no-assets` | off | Skip image downloads |
| `--concurrency <N>` | 8 | Parallel image downloads |
| `-v`, `-vv` | off | Verbose / very verbose |

Exit codes: `0` success; `1` generic; `2` network; `3` render; `4` extraction.

If Chrome is in a non-standard location, set `CHROME_PATH=/path/to/chrome`.

## Output layout

```
<out-dir>/
├── index.md         # converted Markdown with frontmatter
└── assets/          # downloaded images (unless --no-assets)
    └── ...
```

`index.md` starts with frontmatter:

```yaml
---
title: "<extracted title>"
source_url: "<original URL>"
fetched_at: <RFC3339 timestamp>
render_mode: static | headless
---
```

## Behavior

By default w2m tries a static HTTP fetch first. If the page looks like an
empty SPA shell (sparse body, or sparse body with `id="root"` / `id="app"` /
`id="__next"`), it falls back to a headless Chrome render. Pass `--render`
to skip the static attempt, or `--no-render` to disable the fallback.

Content extraction uses a readability heuristic by default. Pass
`--selector "main article"` (or any CSS selector) to override.

## Manual smoke targets

Not in CI. Run by hand to validate paths.

| URL | Tests |
|---|---|
| `https://example.com` | minimal page, exercises the headless fallback |
| any static blog post | static path, readability extraction |
| `https://react.dev/learn` (with `--render`) | SPA path, `id="root"` mount |
| any Next.js page (with `--render`) | `__next` mount detection |

## Development

```bash
cargo test            # unit + integration (no Chrome required)
cargo build --release # release binary
```

The integration test in `tests/integration_static.rs` uses `wiremock` for a
mock HTTP server; it does not exercise the headless path.
