class RustyRelayClient < Formula
  desc "The client which can connect to a rusty-relay server"
  homepage "https://github.com/larscom/rusty-relay"
  version "{{version}}"

  on_macos do
    on_intel do
      url "https://github.com/larscom/rusty-relay/releases/download/{{version}}/rusty-relay-client-{{version}}-macos-x86_64.tar.gz"
      sha256 "{{sha256_macos_intel}}"
    end

    on_arm do
      url "https://github.com/larscom/rusty-relay/releases/download/{{version}}/rusty-relay-client-{{version}}-macos-arm64.tar.gz"
      sha256 "{{sha256_macos_arm}}"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/larscom/rusty-relay/releases/download/{{version}}/rusty-relay-client-{{version}}-linux-x86_64.tar.gz"
      sha256 "{{sha256_linux_intel}}"
    end
  end

  def install
    bin.install "rusty-relay-client"
  end

  test do
    system "#{bin}/rusty-relay-client", "--help"
  end
end
