defmodule Eppo.Server do
  @moduledoc """
  A GenServer that maintains a singleton Eppo SDK client instance for querying feature flags.

  The server can be started directly:

      config = %Eppo.Client.Config{
        api_key: api_key,
        assignment_logger: Eppo.AssignmentLogger,
        is_graceful_mode: true,
        poll_interval_seconds: 30,
        poll_jitter_seconds: 3
      }

      {:ok, _pid} = Eppo.Server.start_link(config)
      client = Eppo.Server.get_instance()

  Or added to your application's supervision tree:

      # In your application.ex
      defmodule YourApp.Application do
        use Application

        def start(_type, _args) do
          config = %Eppo.Client.Config{
            api_key: System.get_env("EPPO_API_KEY"),
            assignment_logger: YourApp.AssignmentLogger,
            # ... other config options ...
          }

          children = [
            # ... other children ...
            {Eppo.Server, config}
          ]

          opts = [strategy: :one_for_one, name: YourApp.Supervisor]
          Supervisor.start_link(children, opts)
        end
      end
  """
  use GenServer

  def start_link(config) do
    GenServer.start_link(__MODULE__, config, name: __MODULE__)
  end

  @doc """
  Returns the singleton client instance.
  The Server must be started (typically in the consuming application's supervision tree).

  Returns client struct or raises if client is not initialized.
  """
  def get_instance() do
    case GenServer.whereis(__MODULE__) do
      nil -> raise "Eppo client not initialized. Ensure Eppo.Client.Server is started."
      pid -> GenServer.call(pid, :get_client)
    end
  end

  @impl true
  def init(%Eppo.Client.Config{} = config) do
    case Eppo.Client.new(config) do
      {:ok, client} -> {:ok, client}
      error -> {:stop, error}
    end
  end

  @impl true
  def handle_call(:get_client, _from, client) do
    {:reply, client, client}
  end
end
