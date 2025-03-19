ExUnit.start()

defmodule TestHelper do
  @doc """
  Initializes the Eppo client for testing with the given test name.
  For offline tests, disables polling. For other tests, configures a local test server.
  """
  use ExUnit.Case

  def init_client_for(test_name) do
    config =
      %EppoSdk.Client.Config{
        api_key: "test-api-key",
        assignment_logger: EppoSdk.AssignmentLogger,
        base_url: "http://127.0.0.1:8378/#{test_name}/api"
      }

    start_supervised({EppoSdk.Server, config})

    # Wait for initialization to complete
    unless test_name == "offline" do
      client = EppoSdk.Server.get_instance()
      EppoSdk.Client.wait_for_initialization(client)
    end
  end
end
