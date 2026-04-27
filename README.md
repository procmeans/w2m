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
| `--wait-ms <N>` | 2000 | Settle delay after navigation (headless path) |
| `--config <PATH>` | XDG default | Override config file path |
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

## Configuration

Per-host defaults can be set in `~/.config/w2m/config.toml` so you don't have
to repeat flags for sites you visit often. Lookup priority:

> CLI flag > `[hosts."<exact-host>"]` > `[defaults]` > built-in default

Example:

```toml
[defaults]
# wait_ms = 2000
# concurrency = 8

[hosts."open.oceanengine.com"]
render = true
wait_ms = 5000
selector = ".doc-content-body"

[hosts."react.dev"]
render = true
wait_ms = 3000
```

After this is in place, `w2m https://open.oceanengine.com/...` works with no
extra flags.

Available keys per host: `render`, `no_render`, `selector`, `no_assets`,
`concurrency`, `wait_ms`. The same keys are accepted under `[defaults]`.

w2m also looks at the platform-native config dir (`dirs::config_dir()`) as a
fallback — on macOS that's `~/Library/Application Support/w2m/config.toml` —
but `~/.config/w2m/config.toml` is preferred and used by default.

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
