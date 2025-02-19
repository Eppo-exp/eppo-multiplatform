defmodule Eppo.AssignmentLogger do
  # Override with your own implementation
  def log_assignment(event) do
    IO.inspect(event, label: "log_assignment")
  end
end
