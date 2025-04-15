# frozen_string_literal: true

require_relative "eppo_client/client"
require_relative "eppo_client/version"

# EppoClient is the main module for initializing the Eppo client.
# It provides a method to initialize the client with a given configuration.
module EppoClient
  ##
  # Initializes the Eppo client singleton.
  #
  # @note The client returned by this method may still be in the process of initializing.
  #       Use the `#wait_for_initialization` method to wait for the client to be ready.
  #
  # @param config [EppoClient::Config] The configuration for the client.
  # @return [EppoClient::Client] The client.
  def init(config)
    client = EppoClient::Client.instance
    client.init(config)
  end

  module_function :init
end
