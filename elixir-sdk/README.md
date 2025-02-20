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
    config = %Eppo.Client.Config{
      api_key: System.get_env("EPPO_API_KEY"),
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

### Using the Client
Once the server is started, you can access the client instance anywhere in your application:

```elixir
# Get the client instance
client = Eppo.Server.get_instance()

# Use the client to evaluate feature flags
assignment = Eppo.Client.get_string_assignment(
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
config = %Eppo.Client.Config{api_key: "your-api-key"}
{:ok, _pid} = Eppo.Server.start_link(config)

# When testing locally, wait for the client to initialize
Process.sleep(1000)

# Then use as normal
client = Eppo.Server.get_instance()
```

Or you can use the client directly:
```elixir
{:ok, client} = Eppo.Client.new(config)

# When testing locally, wait for the client to initialize
Process.sleep(1000)
```

## Development

To run the tests:
```bash
mix test
```
