defmodule ClientUFCTest do
  use ExUnit.Case
  import TestHelper

  # Move setup_all outside of describe block
  setup_all do
    init_client_for("ufc")
    :ok
  end

  describe "UFC flag evaluation" do
    # Get all JSON test files and create a test for each one
    for file <- Path.wildcard("../sdk-test-data/ufc/tests/*.json") do
      basename = Path.basename(file)
      IO.puts("--- #{basename} ---")

      test "with test file #{basename}" do
        data = File.read!(unquote(file)) |> Jason.decode!()

        flag_key = data["flag"]
        variation_type = data["variationType"]
        default_value = data["defaultValue"]

        IO.inspect(flag_key, label: "flag_key")
        # Test each subject in the test file
        Enum.each(data["subjects"], fn subject ->
          subject_key = subject["subjectKey"]
          subject_attributes = subject["subjectAttributes"]
          IO.inspect(subject_key, label: "subject_key")
          IO.inspect(subject_attributes, label: "subject_attributes")

          result =
            case variation_type do
              "STRING" ->
                Client.get_string_assignment(
                  flag_key,
                  subject_key,
                  subject_attributes,
                  default_value
                )

              "NUMERIC" ->
                Client.get_numeric_assignment(
                  flag_key,
                  subject_key,
                  subject_attributes,
                  default_value
                )

              "INTEGER" ->
                Client.get_integer_assignment(
                  flag_key,
                  subject_key,
                  subject_attributes,
                  default_value
                )

              "BOOLEAN" ->
                Client.get_boolean_assignment(
                  flag_key,
                  subject_key,
                  subject_attributes,
                  default_value
                )

              "JSON" ->
                Client.get_json_assignment(
                  flag_key,
                  subject_key,
                  subject_attributes,
                  default_value
                )

              _ ->
                raise "unexpected variationType: #{variation_type}"
            end

          assert result == subject["assignment"],
                 "Failed for subject #{subject_key} in #{unquote(basename)}"
        end)
      end
    end
  end
end
