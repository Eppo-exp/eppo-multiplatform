defmodule SdkCore do
  use Rustler, otp_app: :eppo_sdk, crate: "sdk_core"

  # Client configuration struct
  defmodule Config do
    @type t :: %__MODULE__{
            api_key: String.t(),
            base_url: String.t(),
            assignment_logger: module(),
            is_graceful_mode: boolean(),
            poll_interval_seconds: pos_integer() | nil,
            poll_jitter_seconds: non_neg_integer()
          }

    defstruct [
      :api_key,
      :base_url,
      :assignment_logger,
      is_graceful_mode: true,
      poll_interval_seconds: 30,
      poll_jitter_seconds: 3
    ]
  end

  # Native implemented functions
  def init(_config), do: error()
  def get_instance(), do: error()
  def shutdown(), do: error()

  # Helper function for NIF not loaded errors
  defp error, do: :erlang.nif_error(:nif_not_loaded)
end
