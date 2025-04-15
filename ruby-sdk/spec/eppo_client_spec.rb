# frozen_string_literal: true

require 'json'

RSpec.describe EppoClient do
  it "has a version number" do
    expect(EppoClient::VERSION).not_to be nil
  end

  describe "wait_for_initialization()" do
    before :each do
      EppoClient::Client.instance.init(EppoClient::Config.new("test-api-key", base_url: "http://127.0.0.1:8378/ufc/api"))
      @client = EppoClient::Client.instance
    end

    it "has default timeout" do
      @client.wait_for_initialization()

      expect(@client.configuration).not_to be_nil
    end

    it "allows 0 timeout" do
      @client.wait_for_initialization(0)

      # Local configuration fetching is so fast that sometimes it
      # succeeds before we call wait_for_initialization(). Therefore,
      # we treat absense of errors as a sign of success here.
    end

    it "allows fractional timeout" do
      @client.wait_for_initialization(0.5)

      expect(@client.configuration).not_to be_nil
    end

    it "timeouts with bad URL" do
      EppoClient::Client.instance.init(EppoClient::Config.new("test-api-key", base_url: "http://127.0.0.1:8378/undefined/api"))
      @client = EppoClient::Client.instance

      start = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      @client.wait_for_initialization(0.2)
      stop = Process.clock_gettime(Process::CLOCK_MONOTONIC)

      expect(@client.configuration).to be_nil
      # shall wait at least for timeout
      expect(stop - start).to be >= 0.2
    end
  end

  describe "configuration()" do
    it "allows getting configuration" do
      init_client_for "ufc"

      configuration = EppoClient::Client.instance.configuration

      expect(configuration).not_to be_nil
    end

    it "returns nil when not initialized" do
      init_client_for "offline"

      configuration = EppoClient::Client.instance.configuration

      expect(configuration).to be_nil
    end
  end

  describe "configuration=()" do
    it "allows setting configuration on offline client" do
      init_client_for "offline"

      configuration = EppoClient::Configuration.new(flags_configuration: File.read("../sdk-test-data/ufc/flags-v1.json"))

      EppoClient::Client.instance.configuration = configuration

      expect(EppoClient::Client.instance.configuration).not_to be_nil
    end
  end

  describe "UFC flag evaluation", :flags do
    before :all do
      init_client_for "ufc"
    end

    Dir["../sdk-test-data/ufc/tests/*.json"].each do |file|
      basename = File.basename(file)
      context "with test file #{basename}", :file => basename do
        data = JSON.parse(File.read(file))

        flag_key = data["flag"]
        variation_type = data["variationType"]
        default_value = data["defaultValue"]

        data["subjects"].each do |subject|
          subject_key = subject["subjectKey"]
          subject_attributes = subject["subjectAttributes"]

          it "#{subject_key}", :subject => subject_key do
            result =
              case variation_type
              when "STRING"
                EppoClient::Client.instance.get_string_assignment(flag_key, subject_key, subject_attributes, default_value)
              when "NUMERIC"
                EppoClient::Client.instance.get_numeric_assignment(flag_key, subject_key, subject_attributes, default_value)
              when "INTEGER"
                EppoClient::Client.instance.get_integer_assignment(flag_key, subject_key, subject_attributes, default_value)
              when "BOOLEAN"
                EppoClient::Client.instance.get_boolean_assignment(flag_key, subject_key, subject_attributes, default_value)
              when "JSON"
                EppoClient::Client.instance.get_json_assignment(flag_key, subject_key, subject_attributes, default_value)
              else raise "unexpected variationType: #{variation_type}"
              end

            expect(result).to eq(subject["assignment"])
          end
        end
      end
    end
  end

  describe "UFC flag evaluation with details", :flags do
    before :all do
      init_client_for "ufc"
    end

    Dir["../sdk-test-data/ufc/tests/*.json"].each do |file|
      basename = File.basename(file)
      context "with test file #{basename}", :file => basename do
        data = JSON.parse(File.read(file))

        flag_key = data["flag"]
        variation_type = data["variationType"]
        default_value = data["defaultValue"]

        data["subjects"].each do |subject|
          subject_key = subject["subjectKey"]
          subject_attributes = subject["subjectAttributes"]

          it "#{subject_key}", :subject => subject_key do
            result =
              case variation_type
              when "STRING"
                EppoClient::Client.instance.get_string_assignment_details(flag_key, subject_key, subject_attributes, default_value)
              when "NUMERIC"
                EppoClient::Client.instance.get_numeric_assignment_details(flag_key, subject_key, subject_attributes, default_value)
              when "INTEGER"
                EppoClient::Client.instance.get_integer_assignment_details(flag_key, subject_key, subject_attributes, default_value)
              when "BOOLEAN"
                EppoClient::Client.instance.get_boolean_assignment_details(flag_key, subject_key, subject_attributes, default_value)
              when "JSON"
                EppoClient::Client.instance.get_json_assignment_details(flag_key, subject_key, subject_attributes, default_value)
              else raise "unexpected variationType: #{variation_type}"
              end

            expect(result[:variation]).to eq(subject["assignment"])
            expect(result[:action]).to be_nil

            expect(result[:evaluationDetails][:flagKey]).to eq(flag_key)
            expect(result[:evaluationDetails][:subjectKey]).to eq(subject_key)
          end
        end
      end
    end
  end

  describe "Bandits evaluation", :bandits do
    before :all do
      init_client_for "bandit"
    end

    Dir["../sdk-test-data/ufc/bandit-tests/*.json"].each do |file|
      basename = File.basename(file)
      context "with test file #{basename}", :file => basename do
        data = JSON.parse(File.read(file))

        flag_key = data["flag"]
        default_value = data["defaultValue"]

        data["subjects"].each do |subject|
          subject_key = subject["subjectKey"]
          subject_attributes = subject["subjectAttributes"]

          actions = subject["actions"].map { |action| [action["actionKey"], { "numericAttributes" => action["numericAttributes"], "categoricalAttributes" => action["categoricalAttributes"] }] }.to_h

          it "#{subject_key}", :subject => subject_key do
            expected = {
              :variation=> subject["assignment"]["variation"],
              :action=>subject["assignment"]["action"],
            }

            result =
                EppoClient::Client.instance.get_bandit_action(flag_key, subject_key, subject_attributes, actions, default_value)

            expect(result).to eq(expected)
          end
        end
      end
    end
  end
end
