class W2m < Formula
  desc "Download web pages (incl. SPAs) and convert them to Markdown"
  homepage "https://github.com/procmeans/w2m"
  version "0.1.0"
  license "MIT" # change if you pick a different license

  on_macos do
    on_arm do
      url "https://github.com/procmeans/w2m/releases/download/v#{version}/w2m-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256_FROM_RELEASE"
    end
    on_intel do
      url "https://github.com/procmeans/w2m/releases/download/v#{version}/w2m-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256_FROM_RELEASE"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/procmeans/w2m/releases/download/v#{version}/w2m-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_SHA256_FROM_RELEASE"
    end
    on_arm do
      url "https://github.com/procmeans/w2m/releases/download/v#{version}/w2m-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_SHA256_FROM_RELEASE"
    end
  end

  def install
    bin.install "w2m"
  end

  def caveats
    <<~EOS
      w2m's --render flag drives a local Chrome / Chromium installation.
      If Chrome is not in a standard location, set CHROME_PATH=/path/to/chrome.

      On macOS install Chrome from https://www.google.com/chrome/ or via
        brew install --cask google-chrome
      On Linux install chromium via your distro's package manager, e.g.
        sudo apt install chromium-browser
    EOS
  end

  test do
    assert_match "w2m", shell_output("#{bin}/w2m --version")
  end
end
