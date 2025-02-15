defmodule SdkCoreTest do
  use ExUnit.Case

  test "init with valid config succeeds" do
    config = %SdkCore.Config{
      api_key: "test-key",
      base_url: "https://api.eppo.cloud",
      assignment_logger: nil
    }

    assert {:ok, _} = SdkCore.init(config)
  end

  test "init with empty api_key fails" do
    config = %SdkCore.Config{
      api_key: "",
      base_url: "https://api.eppo.cloud",
      assignment_logger: nil
    }

    assert {:error, "Invalid value for api_key: cannot be blank"} = SdkCore.init(config)
  end

  test "get_instance returns client after init" do
    config = %SdkCore.Config{
      api_key: "test-key",
      base_url: "https://api.eppo.cloud",
      assignment_logger: nil
    }

    {:ok, _} = SdkCore.init(config)
    assert {:ok, _} = SdkCore.get_instance()
  end

  test "get_instance fails if init is not called" do
    assert {:error, "init() must be called before get_instance()"} = SdkCore.get_instance()
  end

  test "shutdown succeeds after init" do
    config = %SdkCore.Config{
      api_key: "test-key",
      base_url: "https://api.eppo.cloud",
      assignment_logger: nil
    }

    {:ok, _} = SdkCore.init(config)
    assert :ok = SdkCore.shutdown()
  end
end
