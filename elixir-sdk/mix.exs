defmodule EppoSdk.MixProject do
  use Mix.Project

  def project do
    [
      app: :eppo_sdk,
      version: "0.1.0",
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      deps: deps(),

      # Docs
      name: "Eppo SDK",
      source_url: "https://github.com/Eppo-exp/eppo-multiplatform/tree/main/elixir-sdk",
      homepage_url: "http://www.geteppo.com",
      docs: [
        # The main page in the docs
        main: "EppoSdk",
        extras: ["README.md"]
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
end
