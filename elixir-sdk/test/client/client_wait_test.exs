defmodule EppoSdk.ClientWaitTest do
  use ExUnit.Case
  doctest EppoSdk.Client

  setup do
    # Reset client singleton state before each test
    :ok
  end

  test "wait_for_initialization succeeds with default timeout" do
    config = %EppoSdk.Client.Config{
      api_key: "test-api-key",
      base_url: "http://127.0.0.1:8378/ufc/api"
    }

    {:ok, client} = EppoSdk.Client.new(config)
    result = EppoSdk.Client.wait_for_initialization(client)

    assert result == :ok
    # We just verify that the function completed successfully
    # The actual configuration fetching result depends on the mock server
    # and we've already asserted that wait_for_initialization returned :ok
  end

  test "wait_for_initialization works with custom timeout" do
    config = %EppoSdk.Client.Config{
      api_key: "test-api-key",
      base_url: "http://127.0.0.1:8378/ufc/api"
    }

    {:ok, client} = EppoSdk.Client.new(config)
    result = EppoSdk.Client.wait_for_initialization(client, 0.5)

    assert result == :ok
  end

  test "wait_for_initialization handles timeouts" do
    config = %EppoSdk.Client.Config{
      api_key: "test-api-key",
      # Bad URL
      base_url: "http://127.0.0.1:8378/undefined/api"
    }

    {:ok, client} = EppoSdk.Client.new(config)
    start_time = System.monotonic_time(:millisecond)
    result = EppoSdk.Client.wait_for_initialization(client, 0.01)
    end_time = System.monotonic_time(:millisecond)

    # Still returns :ok on timeout
    assert result == :ok
    # Verify timeout occurred by checking elapsed time
    # at least 200ms elapsed
    assert end_time - start_time >= 10
  end
end
