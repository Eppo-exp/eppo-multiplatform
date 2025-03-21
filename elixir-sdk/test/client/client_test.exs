defmodule EppoSdk.ClientTest do
  use ExUnit.Case
  import ExUnit.CaptureLog
  import TestHelper

  alias EppoSdk.Client
  alias EppoSdk.Server

  setup_all do
    init_client_for("ufc")
    :ok
  end

  test "incorrect subject attributes logs a warning for get_string_assignment" do
    client = Server.get_instance()

    log =
      capture_log(fn ->
        assignment =
          Client.get_string_assignment(
            client,
            "test-flag",
            "test-subject",
            "invalid",
            "default"
          )

        assert assignment == "default"
      end)

    assert log =~ "warning"
  end

  test "incorrect subject attributes logs a warning for get_string_assignment_details" do
    client = Server.get_instance()

    log =
      capture_log(fn ->
        {assignment, _} =
          Client.get_string_assignment_details(
            client,
            "test-flag",
            "test-subject",
            "invalid",
            "default"
          )

        assert assignment == "default"
      end)

    assert log =~ "warning"
  end
end
