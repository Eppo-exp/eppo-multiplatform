# frozen_string_literal: true

require "singleton"
require "logger"

require_relative "config"

# Tries to require the extension for the current Ruby version first
begin
  RUBY_VERSION =~ /(\d+\.\d+)/
  require_relative "#{Regexp.last_match(1)}/eppo_client"
rescue LoadError
  require_relative "eppo_client"
end

module EppoClient
  # The main client singleton
  class Client
    include Singleton
    attr_accessor :assignment_logger

    def init(config)
      config.validate

      if @core
        STDERR.puts "Eppo Warning: multiple initialization of the client"
        @core.shutdown
      end

      @assignment_logger = config.assignment_logger
      @core = EppoClient::Core::Client.new(config)
    end

    ##
    # Waits for client to fetch configuration and get ready to serve
    # assignments.
    #
    # This method blocks the current thread until configuration is
    # successfully fetched or +timeout+ seconds passed.
    #
    # Note: this method returns immediately if configuration poller
    # has been disabled.
    #
    # @param timeout [Numeric] Maximum time to wait in seconds
    # @return [nil]
    def wait_for_initialization(timeout=1)
      return unless @core
      @core.wait_for_initialization(timeout)
    end

    ##
    # Returns the currently active configuration.
    def configuration
      @core.configuration
    end

    ##
    # Sets the currently active configuration.
    def configuration=(configuration)
      @core.configuration = configuration
    end

    ##
    # Prepare the client for shutdown.
    #
    # This method stops the configuration poller and any other background threads.
    #
    # @return [nil]
    def shutdown
      @core.shutdown
    end

    ##
    # Tracks an arbitrary event. Events must have a type and a payload.
    #
    # @note This method is considered unstable and may change in future versions.
    #
    # @param event_type [String] The type of the event to track.
    # @param payload [Hash] The payload of the event to track.
    def unstable_track(event_type, payload)
      @core.track(event_type, payload)
    end

    ##
    # Returns a string assignment for the given flag key and subject.
    #
    # @param flag_key [String] The key of the flag to get an assignment for.
    # @param subject_key [String] The key of the subject to get an assignment for.
    # @param subject_attributes [Hash] The attributes of the subject to get an assignment for.
    # @param default_value [String] The default value to return if the flag is not found or no assignment can be made.
    # @return [String] The assignment for the given flag key and subject.
    def get_string_assignment(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_inner(flag_key, subject_key, subject_attributes, "STRING", default_value)
    end

    ##
    # Returns a numeric assignment for the given flag key and subject.
    #
    # @param flag_key [String] The key of the flag to get an assignment for.
    # @param subject_key [String] The key of the subject to get an assignment for.
    # @param subject_attributes [Hash] The attributes of the subject to get an assignment for.
    # @param default_value [Numeric] The default value to return if the flag is not found or no assignment can be made.
    # @return [Numeric] The assignment for the given flag key and subject.
    def get_numeric_assignment(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_inner(flag_key, subject_key, subject_attributes, "NUMERIC", default_value)
    end

    ##
    # Returns an integer assignment for the given flag key and subject.
    #
    # @param flag_key [String] The key of the flag to get an assignment for.
    # @param subject_key [String] The key of the subject to get an assignment for.
    # @param subject_attributes [Hash] The attributes of the subject to get an assignment for.
    # @param default_value [Integer] The default value to return if the flag is not found or no assignment can be made.
    # @return [Integer] The assignment for the given flag key and subject.
    def get_integer_assignment(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_inner(flag_key, subject_key, subject_attributes, "INTEGER", default_value)
    end

    ##
    # Returns a boolean assignment for the given flag key and subject.
    #
    # @param flag_key [String] The key of the flag to get an assignment for.
    # @param subject_key [String] The key of the subject to get an assignment for.
    # @param subject_attributes [Hash] The attributes of the subject to get an assignment for.
    # @param default_value [Boolean] The default value to return if the flag is not found or no assignment can be made.
    # @return [Boolean] The assignment for the given flag key and subject.
    def get_boolean_assignment(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_inner(flag_key, subject_key, subject_attributes, "BOOLEAN", default_value)
    end

    ##
    # Returns a JSON assignment for the given flag key and subject.
    #
    # @param flag_key [String] The key of the flag to get an assignment for.
    # @param subject_key [String] The key of the subject to get an assignment for.
    # @param subject_attributes [Hash] The attributes of the subject to get an assignment for.
    # @param default_value [Hash] The default value to return if the flag is not found or no assignment can be made.
    # @return [Hash] The assignment for the given flag key and subject.
    def get_json_assignment(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_inner(flag_key, subject_key, subject_attributes, "JSON", default_value)
    end

    ##
    # Returns detailed information about a string assignment for the given flag key and subject.
    #
    # @note This method is intended for debugging purposes and is discouraged from use in
    #   production. It is a couple of times slower than the non-detail methods. The evaluation
    #   details format is primarily designed for human consumption (debugging) and is therefore
    #   unstable and may change between SDK versions.
    #
    # @param flag_key [String] The key of the flag to get an assignment for.
    # @param subject_key [String] The key of the subject to get an assignment for.
    # @param subject_attributes [Hash] The attributes of the subject to get an assignment for.
    # @param default_value [String] The default value to return if the flag is not found or no assignment can be made.
    # @return [Hash] A hash containing {:variation => assigned_value, :action => nil, :evaluationDetails => {detailed_evaluation_info}}
    def get_string_assignment_details(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_details_inner(flag_key, subject_key, subject_attributes, "STRING", default_value)
    end

    ##
    # Returns detailed information about a numeric assignment for the given flag key and subject.
    #
    # @note This method is intended for debugging purposes and is discouraged from use in
    #   production. It is a couple of times slower than the non-detail methods. The evaluation
    #   details format is primarily designed for human consumption (debugging) and is therefore
    #   unstable.
    #
    # @param flag_key [String] The key of the flag to get an assignment for.
    # @param subject_key [String] The key of the subject to get an assignment for.
    # @param subject_attributes [Hash] The attributes of the subject to get an assignment for.
    # @param default_value [Numeric] The default value to return if the flag is not found or no assignment can be made.
    # @return [Hash] A hash containing {:variation => assigned_value, :action => nil, :evaluationDetails => {detailed_evaluation_info}}
    def get_numeric_assignment_details(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_details_inner(flag_key, subject_key, subject_attributes, "NUMERIC", default_value)
    end

    ##
    # Returns detailed information about an integer assignment for the given flag key and subject.
    #
    # @note This method is intended for debugging purposes and is discouraged from use in
    #   production. It is a couple of times slower than the non-detail methods. The evaluation
    #   details format is primarily designed for human consumption (debugging) and is therefore
    #   unstable and may change between SDK versions.
    #
    # @param flag_key [String] The key of the flag to get an assignment for.
    # @param subject_key [String] The key of the subject to get an assignment for.
    # @param subject_attributes [Hash] The attributes of the subject to get an assignment for.
    # @param default_value [Integer] The default value to return if the flag is not found or no assignment can be made.
    # @return [Hash] A hash containing {:variation => assigned_value, :action => nil, :evaluationDetails => {detailed_evaluation_info}}
    def get_integer_assignment_details(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_details_inner(flag_key, subject_key, subject_attributes, "INTEGER", default_value)
    end

    ##
    # Returns detailed information about a boolean assignment for the given flag key and subject.
    #
    # @note This method is intended for debugging purposes and is discouraged from use in
    #   production. It is a couple of times slower than the non-detail methods. The evaluation
    #   details format is primarily designed for human consumption (debugging) and is therefore
    #   unstable and may change between SDK versions.
    #
    # @param flag_key [String] The key of the flag to get an assignment for.
    # @param subject_key [String] The key of the subject to get an assignment for.
    # @param subject_attributes [Hash] The attributes of the subject to get an assignment for.
    # @param default_value [Boolean] The default value to return if the flag is not found or no assignment can be made.
    # @return [Hash] A hash containing {:variation => assigned_value, :action => nil, :evaluationDetails => {detailed_evaluation_info}}
    def get_boolean_assignment_details(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_details_inner(flag_key, subject_key, subject_attributes, "BOOLEAN", default_value)
    end

    ##
    # Returns detailed information about a JSON assignment for the given flag key and subject.
    #
    # @note This method is intended for debugging purposes and is discouraged from use in
    #   production. It is a couple of times slower than the non-detail methods. The evaluation
    #   details format is primarily designed for human consumption (debugging) and is therefore
    #   unstable and may change between SDK versions.
    #
    # @param flag_key [String] The key of the flag to get an assignment for.
    # @param subject_key [String] The key of the subject to get an assignment for.
    # @param subject_attributes [Hash] The attributes of the subject to get an assignment for.
    # @param default_value [Hash] The default value to return if the flag is not found or no assignment can be made.
    # @return [Hash] A hash containing {:variation => assigned_value, :action => nil, :evaluationDetails => {detailed_evaluation_info}}
    def get_json_assignment_details(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_details_inner(flag_key, subject_key, subject_attributes, "JSON", default_value)
    end

    ##
    # Returns a bandit action based on the flag key, subject, and available actions.
    #
    # @param flag_key [String] The key of the flag to get an action for.
    # @param subject_key [String] The key of the subject to get an action for.
    # @param subject_attributes [Hash] The attributes of the subject.
    # @param actions [Hash] A map of available actions and their attributes.
    # @param default_variation [String] The default variation to return if no assignment can be made.
    # @return [Hash] A hash containing the assigned variation and action.
    def get_bandit_action(flag_key, subject_key, subject_attributes, actions, default_variation)
      attributes = coerce_context_attributes(subject_attributes)
      actions = actions.to_h { |action, attributes| [action, coerce_context_attributes(attributes)] }
      result = @core.get_bandit_action(flag_key, subject_key, attributes, actions, default_variation)

      log_assignment(result[:assignment_event])
      log_bandit_action(result[:bandit_event])

      {
        :variation => result[:variation],
        :action => result[:action]
      }
    end

    ##
    # Returns detailed information about a bandit action based on the flag key, subject, and available actions.
    #
    # @note This method is intended for debugging purposes and is discouraged from use in
    #   production. It is a couple of times slower than the non-detail methods. The evaluation
    #   details format is primarily designed for human consumption (debugging) and is therefore
    #   unstable and may change between SDK versions.
    #
    # @param flag_key [String] The key of the flag to get an action for.
    # @param subject_key [String] The key of the subject to get an action for.
    # @param subject_attributes [Hash] The attributes of the subject.
    # @param actions [Hash] A map of available actions and their attributes.
    # @param default_variation [String] The default variation to return if no assignment can be made.
    # @return [Hash] A hash containing {:variation => assigned_variation, :action => assigned_action, :evaluationDetails => {detailed_evaluation_info}}
    def get_bandit_action_details(flag_key, subject_key, subject_attributes, actions, default_variation)
      attributes = coerce_context_attributes(subject_attributes)
      actions = actions.to_h { |action, attributes| [action, coerce_context_attributes(attributes)] }
      result, details = @core.get_bandit_action_details(flag_key, subject_key, attributes, actions, default_variation)

      log_assignment(result[:assignment_event])
      log_bandit_action(result[:bandit_event])

      {
        :variation => result[:variation],
        :action => result[:action],
        :evaluationDetails => details
      }
    end

    private

    def get_assignment_inner(flag_key, subject_key, subject_attributes, expected_type, default_value)
      logger = Logger.new($stdout)
      begin
        assignment = @core.get_assignment(flag_key, subject_key, subject_attributes, expected_type)
        return default_value unless assignment

        log_assignment(assignment[:event])

        return assignment[:value]
      rescue StandardError => error
        logger.debug("[Eppo SDK] Failed to get assignment: #{error}")

        # TODO: non-graceful mode?
        default_value
      end
    end
    # rubocop:enable Metrics/MethodLength

    # rubocop:disable Metrics/MethodLength
    def get_assignment_details_inner(flag_key, subject_key, subject_attributes, expected_type, default_value)
      result, event = @core.get_assignment_details(flag_key, subject_key, subject_attributes, expected_type)
      log_assignment(event)

      if !result[:variation]
        result[:variation] = default_value
      end

      result
    end
    # rubocop:enable Metrics/MethodLength

    def log_assignment(event)
      return unless event

      # Because rust's AssignmentEvent has a #[flatten] extra_logging
      # field, serde_magnus serializes it as a normal HashMap with
      # string keys.
      #
      # Convert keys to symbols here, so that logger sees symbol-keyed
      # events for both flag assignment and bandit actions.
      event = event.to_h { |key, value| [key.to_sym, value]}

      begin
        @assignment_logger.log_assignment(event)
      rescue EppoClient::AssignmentLoggerError
      # Error means log_assignment was not set up. This is okay to ignore.
      rescue StandardError => error
        logger = Logger.new($stdout)
        logger.error("[Eppo SDK] Error logging assignment event: #{error}")
      end
    end

    def log_bandit_action(event)
      return unless event

      begin
        @assignment_logger.log_bandit_action(event)
      rescue EppoClient::AssignmentLoggerError
      # Error means log_assignment was not set up. This is okay to ignore.
      rescue StandardError => error
        logger = Logger.new($stdout)
        logger.error("[Eppo SDK] Error logging bandit action event: #{error}")
      end
    end

    def coerce_context_attributes(attributes)
      numeric_attributes = attributes[:numeric_attributes] || attributes["numericAttributes"]
      categorical_attributes = attributes[:categorical_attributes] || attributes["categoricalAttributes"]
      if numeric_attributes || categorical_attributes then
        {
          numericAttributes: numeric_attributes.to_h do |key, value|
            value.is_a?(Numeric) ? [key, value] : [nil, nil]
          end.compact,
          categoricalAttributes: categorical_attributes.to_h do |key, value|
            value.nil? ? [nil, nil] : [key, value.to_s]
          end.compact,
        }
      end
    end
  end
end
