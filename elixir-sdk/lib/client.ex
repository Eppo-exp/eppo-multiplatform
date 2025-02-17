defmodule Client do
  alias SdkCore, as: Core
  require Logger

  defmodule Config do
    defstruct [
      :api_key,
      :base_url,
      :assignment_logger,
      :is_graceful_mode,
      :poll_interval_seconds,
      :poll_jitter_seconds
    ]
  end

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

  def get_string_assignment(flag_key, subject_key, subject_attributes, default) do
    get_assignment(flag_key, subject_key, subject_attributes, default, :string)
  end

  def get_boolean_assignment(flag_key, subject_key, subject_attributes, default) do
    get_assignment(flag_key, subject_key, subject_attributes, default, :boolean)
  end

  def get_integer_assignment(flag_key, subject_key, subject_attributes, default) do
    get_assignment(flag_key, subject_key, subject_attributes, default, :integer)
  end

  def get_numeric_assignment(flag_key, subject_key, subject_attributes, default) do
    get_assignment(flag_key, subject_key, subject_attributes, default, :numeric)
  end

  def get_json_assignment(flag_key, subject_key, subject_attributes, default) do
    get_assignment(flag_key, subject_key, subject_attributes, default, :json)
  end

  defp get_assignment(flag_key, subject_key, subject_attributes, default, expected_type) do
    {value, event} = Core.get_assignment(flag_key, subject_key, subject_attributes, expected_type)
    sdk_logger = Process.get(:eppo_sdk_logger)

    case value do
      nil ->
        Logger.warning("No assignment", %{
          flag: flag_key,
          subject: subject_key
        })

        default

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

        sdk_logger.log_assignment(event)
        value
    end
  end
end
