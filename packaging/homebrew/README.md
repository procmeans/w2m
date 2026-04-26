# Homebrew distribution for w2m

This directory holds the formula template. The actual tap is a *separate* GitHub
repository following Homebrew's naming convention.

## One-time setup

1. **Push this repo to GitHub** as `procmeans/w2m`.

2. **Create a sibling tap repo** named `homebrew-w2m`:
   ```bash
   gh repo create procmeans/homebrew-w2m --public --description "Homebrew tap for w2m"
   git clone https://github.com/procmeans/homebrew-w2m
   cd homebrew-w2m
   mkdir Formula
   ```

3. **Cut the first release** in the main `w2m` repo:
   ```bash
   git tag v0.1.0
   git push --tags
   ```
   GitHub Actions (`.github/workflows/release.yml`) builds binaries for
   `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`,
   and `aarch64-unknown-linux-gnu`, and uploads them to the GitHub Release
   along with `.sha256` files.

4. **Fill in the formula's checksums.** From the Releases page, download each
   `.sha256` file (or compute locally) and paste the hashes into
   `packaging/homebrew/w2m.rb`. Then copy the file into the tap repo:
   ```bash
   cp packaging/homebrew/w2m.rb /path/to/homebrew-w2m/Formula/w2m.rb
   cd /path/to/homebrew-w2m
   git add Formula/w2m.rb
   git commit -m "w2m 0.1.0"
   git push
   ```

## Users install with

```bash
brew tap procmeans/w2m
brew install w2m

# or one-shot
brew install procmeans/w2m/w2m
```

## Per-release update

After cutting `v0.1.1`:

1. `git tag v0.1.1 && git push --tags` (workflow rebuilds binaries).
2. Bump `version` in `Formula/w2m.rb`, paste new sha256s, commit to the tap.

`brew bump-formula-pr` can automate steps 2 if you eventually upstream into
homebrew-core.
