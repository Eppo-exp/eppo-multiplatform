defmodule EppoSdk.Client do
  require Logger

  @moduledoc """
  Client for interacting with Eppo feature flags and experiments.

  Provides methods to evaluate feature flags and experiments for subjects based on
  targeting rules and randomization. Supports string and JSON value assignments with
  optional evaluation details.

  ## Configuration

  The client is configured using `EppoSdk.Client.Config`:

  - api_key: SDK API key for authentication (Can be configured in the [Eppo SDK keys page](https://eppo.cloud/configuration/environments/keys))
  - assignment_logger: Module for logging assignments (implements EppoSdk.AssignmentLogger)
  - is_graceful_mode: Whether to fail gracefully on errors (default: true)
  - poll_interval_seconds: Interval between config polls in seconds (default: 30)
  - poll_jitter_seconds: Random jitter added to poll interval (default: 3)
  - base_url: Base URL for the Eppo API (default: https://fscdn.eppo.cloud/api)

  ## Usage

  ### Initialization

  To create a new client, use the `new/1` function with a `Config` struct:

  ```elixir
  config = %EppoSdk.Client.Config{api_key: "your-api-key", assignment_logger: YourApp.AssignmentLogger}
  {:ok, client} = EppoSdk.Client.new(config)
  ```

  ### Evaluating Feature Flags

  Use the client to evaluate feature flags and experiments for subjects:

  ```elixir
  assignment = EppoSdk.Client.get_string_assignment(
    client,
    "flag-key",
    "user-123",
    %{"country" => "US", "age" => 25},
    "default")
  ```
  Note that these functions will never return an error or nil value.
  When an error occurs, the client will return the default value.

  ### Debugging with get_assignment_details

  To get more information about the assignment, you can use get_assignment_details functions.
  These functions return a tuple with the assignment value and a map of additional details.
  This is less efficient than using the get_assignment functions, and should only be used for debugging.

  ```elixir
  {value, details} = EppoSdk.Client.get_string_assignment_details(
    client,
    "flag-key",
    "user-123",
    %{"country" => "US", "age" => 25},
    "default")
  ```
  """

  defstruct [:client_ref, :assignment_logger]

  defmodule Config do
    @moduledoc """
    Configuration for the Eppo client.

    ## Fields
      - api_key: Required API key for authentication
      - assignment_logger: Optional module implementing EppoSdk.AssignmentLogger for tracking assignments
      - is_graceful_mode: Whether to fail gracefully on errors (default: true)
      - poll_interval_seconds: Interval between config polls in seconds (default: 30)
      - poll_jitter_seconds: Random jitter added to poll interval (default: 3)
      - base_url: Base URL for the Eppo API (default: https://fscdn.eppo.cloud/api)
    """
    defstruct [
      :api_key,
      :assignment_logger,
      is_graceful_mode: true,
      poll_interval_seconds: 30,
      poll_jitter_seconds: 3,
      base_url: "https://fscdn.eppo.cloud/api"
    ]
  end

  @doc """
  Creates a new Eppo client for evaluating feature flags.

  Takes a Config struct and returns {:ok, client} on success or {:error, reason} on failure.

  ```elixir
  {:ok, client} = EppoSdk.Client.new(config)
  ```
  """
  def new(%Config{} = config) do
    try do
      client_ref =
        EppoSdk.Core.init(%EppoSdk.Core.Config{
          api_key: config.api_key,
          base_url: config.base_url,
          is_graceful_mode: config.is_graceful_mode,
          poll_interval_seconds: config.poll_interval_seconds,
          poll_jitter_seconds: config.poll_jitter_seconds
        })

      {:ok,
       %__MODULE__{
         client_ref: client_ref,
         assignment_logger: config.assignment_logger
       }}
    rescue
      e in ArgumentError ->
        {:error, Exception.message(e)}
    end
  end
  
  @doc """
  Waits for client to fetch configuration and get ready to serve
  assignments.

  This method blocks the current thread until configuration is
  successfully fetched or `timeout_seconds` passes.

  ## Parameters
    - timeout_seconds: Timeout in seconds (default: 1.0)

  ## Usage
  ```elixir
  EppoSdk.Client.wait_for_initialization(client, 2.0)
  ```
  """
  def wait_for_initialization(%__MODULE__{} = client, timeout_seconds \\ 1.0) do
    EppoSdk.Core.wait_for_initialization(client.client_ref, timeout_seconds)
    :ok
  end

  @doc """
  Assigns a string variant based on the provided flag configuration.

  ## Parameters
    - flag_key: Identifies which set of configuration rules to use
    - subject_key: Unique identifier for the subject (usually a user ID)
    - subject_attributes: Optional key-value pairs for rule evaluation
    - default: Fallback value if assignment fails


  ```elixir
  assignment = EppoSdk.Client.get_string_assignment(
    client,
    "flag-key",
    "user-123",
    %{"country" => "US", "age" => 25},
    "default")
  ```
  """
  def get_string_assignment(
        %__MODULE__{} = client,
        flag_key,
        subject_key,
        subject_attributes,
        default
      ) do
    get_assignment(client, flag_key, subject_key, subject_attributes, default, :string)
  end

  @doc """
  Like get_string_assignment/4 but returns additional evaluation details.
  Returns `{value, details}` tuple.

  ```elixir
  {value, details} = EppoSdk.Client.get_string_assignment_details(
    client,
    "flag-key",
    "user-123",
    %{"country" => "US", "age" => 25},
    "default")
  ```
  """
  def get_string_assignment_details(
        %__MODULE__{} = client,
        flag_key,
        subject_key,
        subject_attributes,
        default
      ) do
    get_assignment_details(client, flag_key, subject_key, subject_attributes, default, :string)
  end

  @doc """
  Assigns a boolean variant based on the provided flag configuration.

  ## Parameters
    - flag_key: Identifies which set of configuration rules to use
    - subject_key: Unique identifier for the subject (usually a user ID)
    - subject_attributes: Optional key-value pairs for rule evaluation
    - default: Fallback value if assignment fails

  ```elixir
  assignment = EppoSdk.Client.get_boolean_assignment(
    client,
    "flag-key",
    "user-123",
    %{"country" => "US", "age" => 25},
    false)
  ```
  """
  def get_boolean_assignment(
        %__MODULE__{} = client,
        flag_key,
        subject_key,
        subject_attributes,
        default
      ) do
    get_assignment(client, flag_key, subject_key, subject_attributes, default, :boolean)
  end

  @doc """
  Like get_boolean_assignment/4 but returns additional evaluation details.
  Returns {value, details} tuple.
  """
  def get_boolean_assignment_details(
        %__MODULE__{} = client,
        flag_key,
        subject_key,
        subject_attributes,
        default
      ) do
    get_assignment_details(client, flag_key, subject_key, subject_attributes, default, :boolean)
  end

  @doc """
  Assigns an integer variant based on the provided flag configuration.

  ## Parameters
    - flag_key: Identifies which set of configuration rules to use
    - subject_key: Unique identifier for the subject (usually a user ID)
    - subject_attributes: Optional key-value pairs for rule evaluation
    - default: Fallback value if assignment fails

  ```elixir
  assignment = EppoSdk.Client.get_integer_assignment(
    client,
    "flag-key",
    "user-123",
    %{"country" => "US", "age" => 25},
    10)
  ```
  """
  def get_integer_assignment(
        %__MODULE__{} = client,
        flag_key,
        subject_key,
        subject_attributes,
        default
      ) do
    get_assignment(client, flag_key, subject_key, subject_attributes, default, :integer)
  end

  @doc """
  Like get_integer_assignment/4 but returns additional evaluation details.
  Returns {value, details} tuple.
  """
  def get_integer_assignment_details(
        %__MODULE__{} = client,
        flag_key,
        subject_key,
        subject_attributes,
        default
      ) do
    get_assignment_details(client, flag_key, subject_key, subject_attributes, default, :integer)
  end

  @doc """
  Assigns a numeric (float) variant based on the provided flag configuration.

  ## Parameters
    - flag_key: Identifies which set of configuration rules to use
    - subject_key: Unique identifier for the subject (usually a user ID)
    - subject_attributes: Optional key-value pairs for rule evaluation
    - default: Fallback value if assignment fails

  ```elixir
  assignment = EppoSdk.Client.get_numeric_assignment(
    client,
    "flag-key",
    "user-123",
    %{"country" => "US", "age" => 25},
    3.14159)
  ```
  """
  def get_numeric_assignment(
        %__MODULE__{} = client,
        flag_key,
        subject_key,
        subject_attributes,
        default
      ) do
    get_assignment(client, flag_key, subject_key, subject_attributes, default, :numeric)
  end

  @doc """
  Like get_numeric_assignment/4 but returns additional evaluation details.
  Returns {value, details} tuple.
  """
  def get_numeric_assignment_details(
        %__MODULE__{} = client,
        flag_key,
        subject_key,
        subject_attributes,
        default
      ) do
    get_assignment_details(client, flag_key, subject_key, subject_attributes, default, :numeric)
  end

  @doc """
  Assigns a JSON variant based on the provided flag configuration.
  Returns a Map

  ## Parameters
    - flag_key: Identifies which set of configuration rules to use
    - subject_key: Unique identifier for the subject (usually a user ID)
    - subject_attributes: Optional key-value pairs for rule evaluation
    - default: Fallback Map if assignment fails


  ```elixir
  assignment = EppoSdk.Client.get_json_assignment(
    client,
    "flag-key",
    "user-123",
    %{"country" => "US", "age" => 25},
    %{"default" => "value"})
  ```
  """
  def get_json_assignment(
        %__MODULE__{} = client,
        flag_key,
        subject_key,
        subject_attributes,
        default
      ) do
    # Use default as "" to force use of default value when decoding
    value_json = get_assignment(client, flag_key, subject_key, subject_attributes, "", :json)

    case decode_value(value_json) do
      nil -> default
      value -> value
    end
  end

  @doc """
  Like get_json_assignment/4 but returns additional evaluation details.
  Returns `{value, details}` tuple.
  """
  def get_json_assignment_details(
        %__MODULE__{} = client,
        flag_key,
        subject_key,
        subject_attributes,
        default
      ) do
    # Use default as "" to force use of default value when decoding
    {value_json, details} =
      get_assignment_details(client, flag_key, subject_key, subject_attributes, "", :json)

    case decode_value(value_json) do
      nil -> {default, details}
      value -> {value, details}
    end
  end

  defp get_assignment(
         %__MODULE__{} = client,
         flag_key,
         subject_key,
         subject_attributes,
         default,
         expected_type
       ) do
    subject_attributes = validate_subject_attributes(subject_attributes)

    assignment =
      EppoSdk.Core.get_assignment(
        client.client_ref,
        flag_key,
        subject_key,
        subject_attributes,
        expected_type
      )

    case assignment do
      {:error, error} ->
        Logger.error("Error getting assignment", %{
          flag: flag_key,
          subject: subject_key,
          error: error
        })

        default

      {value, event} ->
        Logger.debug("Assignment", %{
          flag: flag_key,
          subject: subject_key,
          value: value
        })

        log_assignment(client.assignment_logger, event)

        value
    end
  end

  defp get_assignment_details(
         %__MODULE__{} = client,
         flag_key,
         subject_key,
         subject_attributes,
         default,
         expected_type
       ) do
    subject_attributes = validate_subject_attributes(subject_attributes)

    assignment =
      EppoSdk.Core.get_assignment_details(
        client.client_ref,
        flag_key,
        subject_key,
        subject_attributes,
        expected_type
      )

    case assignment do
      {:error, _} ->
        Logger.error("Error getting assignment details", %{
          flag: flag_key,
          subject: subject_key
        })

        {default, nil}

      {result, event} ->
        # If no variation is found, use the default value
        value = case Map.get(result, "variation") do
          nil -> default
          value -> value
        end

        Logger.debug("Assignment details", %{
          flag: flag_key,
          subject: subject_key,
          value: value
        })

        log_assignment(client.assignment_logger, event)

        if {:ok, details} = Jason.decode(Map.get(result, "details")),
          do: {value, details},
          else: {value, nil}
    end
  end

  defp log_assignment(_, nil), do: nil

  defp log_assignment(logger, event) do
    logger.log_assignment(event)
  end

  defp decode_value(nil), do: nil

  defp decode_value(value_json) do
    case Jason.decode(value_json) do
      {:ok, value} -> value
      {:error, _} -> nil
    end
  end

  defp validate_subject_attributes(subject_attributes) do
    if not is_map(subject_attributes) do
      Logger.warning("Invalid subject attributes: must be a map", %{
        subject: subject_attributes
      })

      %{}
    else
      subject_attributes
    end
  end
end
