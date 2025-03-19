defmodule EppoSdk.Core do
  @moduledoc """
  Defines the interface with the Rust core SDK for evaluating feature flags.

  This module provides the core functionality for feature flag evaluation by interfacing
  with the Rust implementation via NIFs. It handles initialization of the client and
  assignment evaluation while abstracting away the underlying Rust implementation details.
  """
  use Rustler, otp_app: :eppo_sdk, crate: "sdk_core"

  defmodule Config do
    defstruct api_key: "",
              base_url: "",
              is_graceful_mode: true,
              poll_interval_seconds: 30,
              poll_jitter_seconds: 3
  end

  @opaque client :: reference()

  def init(_config), do: error()

  def get_assignment(_client, _flag_key, _subject_key, _subject_attributes, _expected_type),
    do: error()

  def get_assignment_details(
        _client,
        _flag_key,
        _subject_key,
        _subject_attributes,
        _expected_type
      ),
      do: error()
      
  def wait_for_initialization(_client, _timeout_secs \\ 1.0), do: error()

  # Helper function for NIF not loaded errors
  defp error, do: :erlang.nif_error(:nif_not_loaded)
end
