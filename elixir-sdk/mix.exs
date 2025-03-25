defmodule EppoSdk.MixProject do
  use Mix.Project

  def project do
    [
      app: :eppo_sdk,
      version: "0.2.3",
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      description: "Elixir SDK for Eppo's feature flagging and experimentation platform",
      package: package(),
      deps: deps(),

      # Docs
      name: "Eppo SDK",
      source_url: "https://github.com/Eppo-exp/eppo-multiplatform/tree/main/elixir-sdk",
      homepage_url: "http://www.geteppo.com",
      docs: [
        # The main page in the docs
        main: "readme",
        extras: ["README.md", "../LICENSE"]
      ]
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger]
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      # {:dep_from_hexpm, "~> 0.3.0"},
      # {:dep_from_git, git: "https://github.com/elixir-lang/my_dep.git", tag: "0.1.0"}
      {:rustler, "~> 0.30.0"},
      {:jason, "~> 1.4.4"},
      {:ex_doc, "~> 0.21", only: :dev, runtime: false}
    ]
  end

  defp package do
    [
      name: "eppo_sdk",
      licenses: ["MIT"],
      links: %{
        "GitHub" => "https://github.com/Eppo-exp/eppo-multiplatform/tree/main/elixir-sdk",
        "Eppo" => "https://www.geteppo.com",
        "Documentation" => "https://docs.geteppo.com/sdks/server-sdks/"
      },
      files: ~w(lib native/sdk_core mix.exs README.md)
    ]
  end
end
