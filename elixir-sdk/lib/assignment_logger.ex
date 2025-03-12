defmodule EppoSdk.AssignmentLogger do
  @moduledoc """
  Behaviour for logging experiment assignments.
  """

  @callback log_assignment(event :: map()) :: any()

  @doc """
  Default implementation that does nothing.
  """
  def log_assignment(_event), do: nil
end
