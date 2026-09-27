# What the Ruby engine asked and wrote on each step of a sweep script, for
# parity/records/ (see parity/README.md). Run in a scratch copy of the Ruby
# engine's repository, never a working checkout, with both provider keys
# unset and its test database's schema loaded:
#
#   RAILS_ENV=test bin/rails runner records.rb <output directory> <script>...
#
# Each named script is played the way `bin/rails engine:parity` plays it (the
# walk rolled back, no model), and two files are written for it:
# `goldens/<script>.json`, the same file that task writes, and
# `records/<script>.json`, which holds for every step each request a browser
# step handed a provider and every row of the tables below that the walk
# wrote, changed or removed, measured against the world as it was loaded.
module Capture
  SENT = []
  TABLES = %w[locations location_connections characters items quest_steps scenes characters_scenes
              interactions playthrough_endings playthrough_beats playthrough_npc_states world_events].freeze
  SKIPPED = %w[created_at updated_at].freeze

  def self.schema_of(schema)
    return nil if schema.nil?

    JSON.parse(JSON.generate(schema.new.to_json_schema))
  end

  def self.rows
    TABLES.to_h do |table|
      rows = ActiveRecord::Base.connection.select_all("SELECT * FROM #{table} ORDER BY #{table == 'characters_scenes' ? 'scene_id, character_id' : 'id'}").to_a
      [ table, JSON.parse(rows.map { |row| row.except(*SKIPPED) }.to_json) ]
    end
  end

  # What the walk wrote: every row that is not in the world as it was loaded,
  # and the id of every loaded row that is gone (a row that only changed is
  # listed as it is now, and not as gone).
  def self.written(loaded)
    rows.to_h do |table, now|
      before = loaded.fetch(table)
      [ table, { "rows" => now - before, "removed" => (before - now).map { |row| row["id"] || row } - (now - before).map { |row| row["id"] } } ]
    end
  end

  class << self
    attr_accessor :loaded
  end

  module Loaded
    def load_world!(...)
      super.tap { Capture.loaded = Capture.rows }
    end
  end
end
EngineSweep::Walk.prepend(Capture::Loaded)

class EngineSweep::BrowserTurn::Agent
  attr_accessor :instructions, :schema

  def with_instructions(text, *) = tap { self.instructions = text }
  def with_schema(schema, *) = tap { self.schema = schema }

  alias_method :uncaptured_ask, :ask
  def ask(prompt, verify: nil, **rest, &block)
    Capture::SENT << { "purpose" => @purpose, "system" => instructions, "user" => prompt,
                       "schema" => Capture.schema_of(schema) }
    uncaptured_ask(prompt, verify: verify, **rest, &block)
  end
end

class EngineSweep::BrowserTurn::TypedAgent
  alias_method :uncaptured_ask_questions, :ask_questions
  def ask_questions(state:, questions:)
    Capture::SENT << { "purpose" => PURPOSE, "state" => JSON.parse(JSON.generate(state)),
                       "questions" => JSON.parse(JSON.generate(questions)) }
    uncaptured_ask_questions(state: state, questions: questions)
  end
end

# The browser step's own stand-in for the providers, keeping the
# instructions and the schema each agent was built with.
class EngineSweep::BrowserTurn
  private

  def without_provider(calls, replies:, prompt_failures:)
    original = BaseAgent.method(:new)
    typed = SystemOneAgent.method(:new)
    switch = SystemOneAgent.method(:configured?)
    keyed = replies.any? { |reply| reply["purpose"] == TypedAgent::PURPOSE }

    BaseAgent.singleton_class.send(:define_method, :new) do |*args, **options|
      Agent.new(options[:purpose], calls, replies, prompt_failures).tap do |agent|
        agent.instructions = args[0]
        agent.schema = args[1]
      end
    end
    if keyed
      SystemOneAgent.singleton_class.send(:define_method, :new) do |*_args, **_options|
        TypedAgent.new(calls, replies, prompt_failures)
      end
      SystemOneAgent.singleton_class.send(:define_method, :configured?) { true }
    end
    yield
  ensure
    BaseAgent.singleton_class.send(:define_method, :new, original)
    SystemOneAgent.singleton_class.send(:define_method, :new, typed)
    SystemOneAgent.singleton_class.send(:define_method, :configured?, switch)
  end
end

out = Pathname(ARGV.fetch(0))
%w[goldens records].each { |dir| FileUtils.mkdir_p(out.join(dir)) }
ARGV.drop(1).each do |name|
  script = EngineSweep.scripts.find { |s| s.name == name } or abort "no script #{name}"
  dumps = []
  steps = []
  result = EngineSweep.without_a_model do
    EngineSweep::Walk.new(script, on_step: lambda { |step, dump|
      dumps << JSON.parse(dump.to_h.to_json)
      steps << { "step" => step.label, "requests" => Capture::SENT.dup, "written" => Capture.written(Capture.loaded) }
      Capture::SENT.clear
    }).play
  end
  puts result.line
  puts result.report unless result.passed?
  out.join("goldens", "#{name}.json").write(EngineSweep::Parity.render(EngineSweep::Parity.document(script, dumps)))
  out.join("records", "#{name}.json").write("#{JSON.pretty_generate({ "script" => name, "steps" => steps })}\n")
end
