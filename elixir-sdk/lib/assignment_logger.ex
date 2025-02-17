defmodule AssignmentLogger do
  # Override with your own implementation
  def log_assignment(event) do
    IO.inspect(event, label: "assignment_event")
  end
end
