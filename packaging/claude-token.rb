# Homebrew Cask template for claude-token.
# CI (.github/workflows/release.yml) fills in the version and sha256 below,
# then writes the result to ayuayuyu/homebrew-tap Casks/claude-token.rb.
cask "claude-token" do
  version "__VERSION__"
  sha256 "__SHA256__"

  url "https://github.com/ayuayuyu/claude-token/releases/download/v#{version}/claude-token-#{version}-universal.dmg"
  name "Claude Token"
  desc "Menu bar + desktop widget showing Claude Code usage limits"
  homepage "https://github.com/ayuayuyu/claude-token"

  app "claude-token.app"

  caveats <<~EOS
    claude-token は個人開発のため Apple 公証を受けていません。
    初回起動時に「開発元を確認できません」と表示された場合、以下のいずれかで対処してください:

      # 方法 1: install 後に quarantine 属性を除去（推奨）
      xattr -dr com.apple.quarantine "/Applications/claude-token.app"

      # 方法 2: 環境変数で install 時に quarantine を付けない
      HOMEBREW_CASK_OPTS="--no-quarantine" brew install --cask ayuayuyu/tap/claude-token

      # 方法 3: Finder で右クリック → 開く（初回のみ）

    ソースは https://github.com/ayuayuyu/claude-token で公開しています。
  EOS

  zap trash: [
    "~/Library/Application Support/dev.ayuayuyu.claudetoken",
    "~/Library/Caches/dev.ayuayuyu.claudetoken",
    "~/Library/WebKit/dev.ayuayuyu.claudetoken",
  ]
end
