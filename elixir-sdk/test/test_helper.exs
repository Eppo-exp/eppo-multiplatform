ExUnit.start()

defmodule TestHelper do
  @doc """
  Initializes the Eppo client for testing with the given test name.
  For offline tests, disables polling. For other tests, configures a local test server.
  """
  def init_client_for(test_name) do
    config =
      case test_name do
        "offline" ->
          %Client.Config{
            api_key: "test-api-key",
            assignment_logger: AssignmentLogger,
            poll_interval_seconds: nil,
            is_graceful_mode: true
          }

        _ ->
          %Client.Config{
            api_key: "test-api-key",
            base_url: "http://127.0.0.1:8378/#{test_name}/api"
          }
      end

    Client.init(config)

    # Sleep to allow client to fetch config
    unless test_name == "offline" do
      :timer.sleep(50)
    end
  end
end
