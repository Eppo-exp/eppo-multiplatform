defmodule Eppo.Core do
  use Rustler, otp_app: :eppo_sdk, crate: "sdk_core"

  defmodule Config do
    defstruct api_key: "",
              base_url: "",
              is_graceful_mode: true,
              poll_interval_seconds: 30,
              poll_jitter_seconds: 3
  end

  # Client loading functions
  def init(_config), do: error()
  # def get_instance(), do: error()
  def shutdown(), do: error()

  # Feature flag evaluation functions
  def get_assignment(_flag_key, _subject_key, _subject_attributes, _expected_type), do: error()

  def get_assignment_details(_flag_key, _subject_key, _subject_attributes, _expected_type),
    do: error()

  # Helper function for NIF not loaded errors
  defp error, do: :erlang.nif_error(:nif_not_loaded)
end
