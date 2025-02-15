defmodule Client do
  alias SdkCore, as: Core

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
    assignment = Core.get_string_assignment(flag_key, subject_key, subject_attributes)

    case assignment do
      nil -> default
      assignment -> assignment
    end
  end

  def get_boolean_assignment(flag_key, subject_key, subject_attributes, default) do
    assignment = Core.get_boolean_assignment(flag_key, subject_key, subject_attributes)

    case assignment do
      nil -> default
      assignment -> assignment
    end
  end
end
