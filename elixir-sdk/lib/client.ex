defmodule Client do
  alias SdkCore, as: Core
  require Logger

  @moduledoc """
  Configuration struct for the Eppo client.

  Fields:
  - api_key: API key for authentication
  - assignment_logger: Module for logging assignments
  - is_graceful_mode: Whether to fail gracefully on errors
  - poll_interval_seconds: Interval between config polls in seconds
  - poll_jitter_seconds: Random jitter added to poll interval
  - base_url: Base URL for the Eppo API (default: https://fscdn.eppo.cloud/api)
  """
  defmodule Config do
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
  Initializes the Eppo client with the provided configuration.
  Returns {:ok, _} on success or {:error, reason} on failure.
  """
  def init(%Config{} = config) do
    Process.put(:eppo_sdk_logger, config.assignment_logger)

    core_config = %Core.Config{
      api_key: config.api_key,
      base_url: config.base_url,
      is_graceful_mode: config.is_graceful_mode,
      poll_interval_seconds: config.poll_interval_seconds,
      poll_jitter_seconds: config.poll_jitter_seconds
    }

    Core.init(core_config)
  end

  def shutdown, do: Core.shutdown()

  @doc """
  Assigns a string variant based on the provided flag configuration.

  ## Parameters
    - flag_key: Identifies which set of configuration rules to use
    - subject_key: Unique identifier for the subject (usually a user ID)
    - subject_attributes: Optional key-value pairs for rule evaluation
    - default: Fallback value if assignment fails
  """
  def get_string_assignment(flag_key, subject_key, subject_attributes, default) do
    get_assignment(flag_key, subject_key, subject_attributes, default, :string)
  end

  @doc """
  Like get_string_assignment/4 but returns additional evaluation details.
  Returns {value, details} tuple.
  """
  def get_string_assignment_details(flag_key, subject_key, subject_attributes, default) do
    get_assignment_details(flag_key, subject_key, subject_attributes, default, :string)
  end

  @doc """
  Assigns a boolean variant based on the provided flag configuration.

  ## Parameters
    - flag_key: Identifies which set of configuration rules to use
    - subject_key: Unique identifier for the subject (usually a user ID)
    - subject_attributes: Optional key-value pairs for rule evaluation
    - default: Fallback value if assignment fails
  """
  def get_boolean_assignment(flag_key, subject_key, subject_attributes, default) do
    get_assignment(flag_key, subject_key, subject_attributes, default, :boolean)
  end

  @doc """
  Like get_boolean_assignment/4 but returns additional evaluation details.
  Returns {value, details} tuple.
  """
  def get_boolean_assignment_details(flag_key, subject_key, subject_attributes, default) do
    get_assignment_details(flag_key, subject_key, subject_attributes, default, :boolean)
  end

  @doc """
  Assigns an integer variant based on the provided flag configuration.

  ## Parameters
    - flag_key: Identifies which set of configuration rules to use
    - subject_key: Unique identifier for the subject (usually a user ID)
    - subject_attributes: Optional key-value pairs for rule evaluation
    - default: Fallback value if assignment fails
  """
  def get_integer_assignment(flag_key, subject_key, subject_attributes, default) do
    get_assignment(flag_key, subject_key, subject_attributes, default, :integer)
  end

  @doc """
  Like get_integer_assignment/4 but returns additional evaluation details.
  Returns {value, details} tuple.
  """
  def get_integer_assignment_details(flag_key, subject_key, subject_attributes, default) do
    get_assignment_details(flag_key, subject_key, subject_attributes, default, :integer)
  end

  @doc """
  Assigns a numeric (float) variant based on the provided flag configuration.

  ## Parameters
    - flag_key: Identifies which set of configuration rules to use
    - subject_key: Unique identifier for the subject (usually a user ID)
    - subject_attributes: Optional key-value pairs for rule evaluation
    - default: Fallback value if assignment fails
  """
  def get_numeric_assignment(flag_key, subject_key, subject_attributes, default) do
    get_assignment(flag_key, subject_key, subject_attributes, default, :numeric)
  end

  @doc """
  Like get_numeric_assignment/4 but returns additional evaluation details.
  Returns {value, details} tuple.
  """
  def get_numeric_assignment_details(flag_key, subject_key, subject_attributes, default) do
    get_assignment_details(flag_key, subject_key, subject_attributes, default, :numeric)
  end

  @doc """
  Assigns a JSON variant based on the provided flag configuration.
  Returns a Map

  ## Parameters
    - flag_key: Identifies which set of configuration rules to use
    - subject_key: Unique identifier for the subject (usually a user ID)
    - subject_attributes: Optional key-value pairs for rule evaluation
    - default: Fallback Map if assignment fails
  """
  def get_json_assignment(flag_key, subject_key, subject_attributes, default) do
    value_json = get_assignment(flag_key, subject_key, subject_attributes, default, :json)
    if {:ok, value} = Jason.decode(value_json), do: value, else: default
  end

  @doc """
  Like get_json_assignment/4 but returns additional evaluation details.
  Returns {value, details} tuple.
  """
  def get_json_assignment_details(flag_key, subject_key, subject_attributes, default) do
    {value_json, details} =
      get_assignment_details(flag_key, subject_key, subject_attributes, default, :json)

    if {:ok, value} = Jason.decode(value_json), do: {value, details}, else: {default, details}
  end

  defp get_assignment(flag_key, subject_key, subject_attributes, default, expected_type) do
    assignment =
      Core.get_assignment(flag_key, subject_key, subject_attributes, expected_type)

    IO.inspect(assignment, label: "assignment")

    case assignment do
      {:error, error} ->
        Logger.error("Error getting assignment", %{
          flag: flag_key,
          subject: subject_key,
          error: error
        })

        default

      {value, event_json} ->
        Logger.info("Assignment", %{
          flag: flag_key,
          subject: subject_key,
          value: value
        })

        case Jason.decode(event_json) do
          {:ok, event} ->
            log_assignment(event)

          {:error, _} ->
            Logger.error("Failed to decode assignment event #{event_json}", %{
              event_json: event_json
            })
        end

        value
    end
  end

  defp get_assignment_details(flag_key, subject_key, subject_attributes, default, expected_type) do
    assignment =
      Core.get_assignment_details(flag_key, subject_key, subject_attributes, expected_type)

    case assignment do
      :error ->
        Logger.error("Error getting assignment details", %{
          flag: flag_key,
          subject: subject_key
        })

        {default, nil}

      {result, event_json} ->
        value = Map.get(result, "variation")

        Logger.info("Assignment details", %{
          flag: flag_key,
          subject: subject_key,
          value: value
        })

        if {:ok, event} = Jason.decode(event_json), do: log_assignment(event)

        if {:ok, details} = Jason.decode(Map.get(result, "details")),
          do: {value, details},
          else: {value, nil}
    end
  end

  defp log_assignment(event) do
    sdk_logger = Process.get(:eppo_sdk_logger)
    sdk_logger.log_assignment(event)
  end
end
