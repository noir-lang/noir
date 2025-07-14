#!/usr/bin/env ruby

require 'fileutils'
require 'open3'

def run_nargo(program_path, program_str)
  nargo_path = "../../target/release/nargo"
  cmd_str = "#{nargo_path} compile --debug-compile-stdin --pedantic-solving"

  # If opts[:stdin_data] is specified, it is sent to the command’s standard input.
  stdout_and_stderr_str, status = Open3.capture2e(cmd_str, stdin_data: program_str)

  if status.success?
    raise "unexpected successful input: #{program_path}"
  end

  stdout_and_stderr_str
end

# update ./collected_outputs
system "./collect_outputs.rb ./collected_outputs"
puts

num_crashes = 0
Dir['./collected_outputs/*'].map do |collected_output_path|
  [collected_output_path, File.read(collected_output_path)]
end.map do |collected_output_path, collected_output_str|
  puts "running 'nargo' on #{collected_output_path}"
  nargo_output = run_nargo collected_output_path, collected_output_str
  [collected_output_path, collected_output_str, nargo_output]
end.group_by do |collected_output_path, collected_output_str, nargo_output|
  nargo_output
end.map do |nargo_output, collected_outputs|
  smallest_input_path_and_str = collected_outputs.min_by do |_, collected_output_str, _|
    collected_output_str.length
  end
  puts "#{collected_outputs.length} non-unique crashes"
  smallest_input_path = smallest_input_path_and_str[0]
  smallest_input_str = smallest_input_path_and_str[1]

  puts "copying #{smallest_input_path.inspect}, (input size: #{smallest_input_str.length})"
  FileUtils.cp smallest_input_path, './unique_crashes'

  smallest_input_basename = File.basename smallest_input_path
  File.write "./unique_crashes/#{smallest_input_basename}.out", nargo_output

  num_crashes += 1
end
puts

puts "copied #{num_crashes} output-unique crashes."
