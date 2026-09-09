defmodule NostrElixir.MixProject do
  use Mix.Project

  def project do
    [
      app: :nostr_elixir,
      version: "0.3.0",
      elixir: "~> 1.14",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: description(),
      package: package(),
      source_url: "https://github.com/rust-nostr/nostr-elixir",
      docs: [
        main: "readme",
        extras: ["README.md", "CHANGELOG.md"],
        source_url: "https://github.com/rust-nostr/nostr-elixir"
      ]
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:rustler, "~> 0.36.2", runtime: false},
      {:jason, "~> 1.4"},
      {:ex_doc, "~> 0.29", only: :dev, runtime: false},
      {:dialyxir, "~> 1.3", only: [:dev], runtime: false}
    ]
  end

  defp description do
    """
    An Elixir wrapper for the rust-nostr library, built with Rustler.
    Provides key management, NIP-19 (including naddr/nevent/nprofile), events, filters, and common NIPs.
    """
  end

  defp package do
    [
      name: "nostr_elixir",
      files: ~w(lib native mix.exs README.md CHANGELOG.md LICENSE),
      licenses: ["MIT"],
      links: %{
        "GitHub" => "https://github.com/rust-nostr/nostr-elixir",
        "nostr Rust Library" => "https://github.com/rust-nostr/nostr"
      }
    ]
  end
end
