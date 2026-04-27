class W2m < Formula
  desc "Download web pages (incl. SPAs) and convert them to Markdown"
  homepage "https://github.com/procmeans/w2m"
  version "0.1.4"
  license "MIT" # change if you pick a different license

  on_macos do
    on_arm do
      url "https://github.com/procmeans/w2m/releases/download/v#{version}/w2m-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "d98c4642c28ac9872c184005bd2d143f34ede3c4320677e789a5c43519b368ad"
    end
    on_intel do
      odie "Intel macOS is not currently distributed as a prebuilt binary. Build from source: cargo install --git https://github.com/procmeans/w2m"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/procmeans/w2m/releases/download/v#{version}/w2m-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "4f01618eaf62e079484a3d6e732d4e31c06954b29dcc58fcdf7d84e349fea494"
    end
    on_arm do
      odie "Linux ARM is not currently distributed as a prebuilt binary. Build from source: cargo install --git https://github.com/procmeans/w2m"
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
