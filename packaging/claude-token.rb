# Homebrew Cask template for claude-token.
# CI (.github/workflows/release.yml) fills in the version and sha256 below,
# then writes the result to ayuayuyu/homebrew-tap Casks/claude-token.rb.
#
# Manual install (unsigned app, so --no-quarantine is required):
#   brew install --cask --no-quarantine ayuayuyu/tap/claude-token
cask "claude-token" do
  version "__VERSION__"
  sha256 "__SHA256__"

  url "https://github.com/ayuayuyu/claude-token/releases/download/v#{version}/claude-token-#{version}-universal.dmg"
  name "Claude Token"
  desc "Menu bar + desktop widget showing Claude Code usage limits"
  homepage "https://github.com/ayuayuyu/claude-token"

  app "claude-token.app"

  caveats <<~EOS
    claude-token is not code-signed. Install with --no-quarantine:
      brew install --cask --no-quarantine claude-token
    Or, after a normal install, clear the quarantine flag once:
      xattr -dr com.apple.quarantine "/Applications/claude-token.app"
  EOS

  zap trash: [
    "~/Library/Application Support/dev.ayuayuyu.claudetoken",
    "~/Library/Caches/dev.ayuayuyu.claudetoken",
    "~/Library/WebKit/dev.ayuayuyu.claudetoken",
  ]
end
