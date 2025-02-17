defmodule SdkCoreTest do
  use ExUnit.Case

  @base_url "https://fscdn.eppo.cloud/api"

  test "init with valid config succeeds" do
    config = %SdkCore.Config{
      api_key: "test-key",
      base_url: @base_url
    }

    assert {:ok, _} = SdkCore.init(config)
  end

  test "init with empty api_key fails" do
    config = %SdkCore.Config{
      api_key: "",
      base_url: @base_url
    }

    assert {:error, "Invalid value for api_key: cannot be blank"} = SdkCore.init(config)
  end

  test "shutdown succeeds after init" do
    config = %SdkCore.Config{
      api_key: "test-key",
      base_url: @base_url
    }

    {:ok, _} = SdkCore.init(config)
    assert :ok = SdkCore.shutdown()
  end
end
