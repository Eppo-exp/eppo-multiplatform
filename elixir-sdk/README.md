# EppoSdk

[Eppo](https://www.geteppo.com/) is a modular flagging and experimentation analysis tool.
Before proceeding you'll need an Eppo account.

## Features

- Feature gates
- Kill switches
- Progressive rollouts
- A/B/n experiments
- Mutually exclusive experiments (Layers)
- Holdouts
- Contextual multi-armed bandits
- Dynamic configuration

## Installation

Add `eppo_sdk` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:eppo_sdk, "~> 0.1.0"}
  ]
end
```

## Usage

### Setup
Add the Eppo Client Server to your application's supervision tree:

```elixir
# In your application.ex
defmodule YourApp.Application do
  use Application

  def start(_type, _args) do
    config = %EppoSdk.Client.Config{
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
```

### Implementing an Assignment Logger

The assignment logger is used to track experiment assignments for analytics. Implement the `Eppo.AssignmentLogger` behaviour in your application:

```elixir
defmodule YourApp.AssignmentLogger do
  @behaviour Eppo.AssignmentLogger

  @impl true
  def log_assignment(event) do
    # Implement your logging logic here
    IO.inspect(event, label: "log_assignment")
  end
end
```

### Using the Client
Once the server is started, you can access the client instance anywhere in your application:

```elixir
# Get the client instance
client = Eppo.Server.get_instance()

# Use the client to evaluate feature flags
assignment = EppoSdk.Client.get_string_assignment(
  client,
  "flag-key",
  "user-123",
  %{},
  "default"
)
```

### Manual Start (e.g., for testing)
If you need to start the server manually (not recommended for production):

```elixir
config = %EppoSdk.Client.Config{api_key: "your-api-key", assignment_logger: YourApp.AssignmentLogger}
{:ok, _pid} = EppoSdk.Server.start_link(config)

# When testing locally, wait for the client to initialize
client = EppoSdk.Server.get_instance()
EppoSdk.Client.wait_for_initialization(client)

# Then use as normal
client = EppoSdk.Server.get_instance()
```

Or you can use the client directly:
```elixir
{:ok, client} = EppoSdk.Client.new(config)

# When testing locally, wait for the client to initialize
EppoSdk.Client.wait_for_initialization(client)
```

## Development

To run the tests:
```bash
mix test
```
