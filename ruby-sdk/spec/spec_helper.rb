# frozen_string_literal: true

require "eppo_client"

def init_client_for(test_name)
  if test_name == "offline" then
    EppoClient::Client.instance.init(EppoClient::Config.new("test-api-key", poll_interval_seconds: nil))
  else
    config = EppoClient::Config.new("test-api-key", base_url: "http://127.0.0.1:8378/#{test_name}/api")
    EppoClient::Client.instance.init(config)
    EppoClient::Client.instance.wait_for_initialization()
  end
end

RSpec.configure do |config|
  # Enable flags like --only-failures and --next-failure
  config.example_status_persistence_file_path = ".rspec_status"

  # Disable RSpec exposing methods globally on `Module` and `main`
  config.disable_monkey_patching!

  config.expect_with :rspec do |c|
    c.syntax = :expect
  end
end
