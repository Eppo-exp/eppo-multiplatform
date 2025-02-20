defmodule Eppo.Server do
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
