defmodule EppoSdk.ClientUFCDetailsTest do
  use ExUnit.Case
  import TestHelper
  alias EppoSdk.{Client, Server}

  # Disable logging for tests
  @moduletag :capture_log

  # Move setup_all outside of describe block
  setup_all do
    init_client_for("ufc")
    :ok
  end

  describe "UFC flag evaluation with details" do
    # Get all JSON test files and create a test for each one
    for file <- Path.wildcard("../sdk-test-data/ufc/tests/*.json") do
      basename = Path.basename(file)

      test "with test file #{basename}" do
        data = File.read!(unquote(file)) |> Jason.decode!()

        client = Server.get_instance()
        flag_key = data["flag"]
        variation_type = data["variationType"]
        default_value = data["defaultValue"]

        # Test each subject in the test file
        Enum.each(data["subjects"], fn subject ->
          subject_key = subject["subjectKey"]
          subject_attributes = subject["subjectAttributes"]

          {value, _details} =
            case variation_type do
              "STRING" ->
                Client.get_string_assignment_details(
                  client,
                  flag_key,
                  subject_key,
                  subject_attributes,
                  default_value
                )

              "NUMERIC" ->
                Client.get_numeric_assignment_details(
                  client,
                  flag_key,
                  subject_key,
                  subject_attributes,
                  default_value
                )

              "INTEGER" ->
                Client.get_integer_assignment_details(
                  client,
                  flag_key,
                  subject_key,
                  subject_attributes,
                  default_value
                )

              "BOOLEAN" ->
                Client.get_boolean_assignment_details(
                  client,
                  flag_key,
                  subject_key,
                  subject_attributes,
                  default_value
                )

              "JSON" ->
                Client.get_json_assignment_details(
                  client,
                  flag_key,
                  subject_key,
                  subject_attributes,
                  default_value
                )

              _ ->
                raise "unexpected variationType: #{variation_type}"
            end

          expected_value = subject["assignment"]

          assert value == expected_value,
                 "Failed for subject #{subject_key} in #{unquote(basename)}. Found value #{inspect(value)} but expected #{inspect(expected_value)}"
        end)
      end
    end
  end
end
