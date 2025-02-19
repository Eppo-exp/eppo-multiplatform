defmodule Eppo.CoreTest do
  use ExUnit.Case

  alias Eppo.Core

  @base_url "https://fscdn.eppo.cloud/api"

  test "init with valid config succeeds" do
    config = %Core.Config{
      api_key: "test-key",
      base_url: @base_url
    }

    assert {:ok, _} = Core.init(config)
  end

  test "init with empty api_key fails" do
    config = %Core.Config{
      api_key: "",
      base_url: @base_url
    }

    assert {:error, "Invalid value for api_key: cannot be blank"} = Core.init(config)
  end

  test "shutdown succeeds after init" do
    config = %Core.Config{
      api_key: "test-key",
      base_url: @base_url
    }

    {:ok, _} = Core.init(config)
    assert {:ok, _} = Core.shutdown()
  end
end
