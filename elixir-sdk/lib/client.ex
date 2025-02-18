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
      :is_graceful_mode,
      :poll_interval_seconds,
      :poll_jitter_seconds,
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

  ## Parameters
    - flag_key: Identifies which set of configuration rules to use
    - subject_key: Unique identifier for the subject (usually a user ID)
    - subject_attributes: Optional key-value pairs for rule evaluation
    - default: Fallback value if assignment fails
  """
  def get_json_assignment(flag_key, subject_key, subject_attributes, default) do
    get_assignment(flag_key, subject_key, subject_attributes, default, :json)
  end

  @doc """
  Like get_json_assignment/4 but returns additional evaluation details.
  Returns {value, details} tuple.
  """
  def get_json_assignment_details(flag_key, subject_key, subject_attributes, default) do
    get_assignment_details(flag_key, subject_key, subject_attributes, default, :json)
  end

  defp get_assignment(flag_key, subject_key, subject_attributes, default, expected_type) do
    {value, event} = Core.get_assignment(flag_key, subject_key, subject_attributes, expected_type)

    case value do
      :error ->
        Logger.error("Error getting assignment", %{
          flag: flag_key,
          subject: subject_key
        })

        default

      value ->
        Logger.info("Assignment", %{
          flag: flag_key,
          subject: subject_key,
          value: value
        })

        log_assignment(event)
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

        {default, nil, nil}

      {result, event} ->
        value = Map.get(result, "variation")
        details = Map.get(result, "details")

        Logger.info("Assignment details", %{
          flag: flag_key,
          subject: subject_key,
          value: value
        })

        log_assignment(event)
        {value, details}
    end
  end

  defp log_assignment(event) do
    sdk_logger = Process.get(:eppo_sdk_logger)
    sdk_logger.log_assignment(event)
  end
end
