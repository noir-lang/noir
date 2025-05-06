#!/bin/env ruby

require 'open3'

# TODO: migrate rust repo clone + dir setup to this file and run on the 'test(s)' dir

def run_nargo(input_path)
  # example:
  # program_str = "fn main() { }"
  program_str = File.read(input_path)
  nargo_path = "/home/michael/coding/rust/noir/target/release/nargo"
  nargo_panic_str = "The application panicked (crashed)."
  cmd_str = "#{nargo_path} compile --debug-compile-stdin"

  start_time = Time.now

  # If opts[:stdin_data] is specified, it is sent to the command’s standard input.
  stdout_and_stderr_str, status = Open3.capture2e(cmd_str, stdin_data: program_str)

  end_time = Time.now
  total_seconds = end_time - start_time
  bytes_per_second = program_str.size / total_seconds

  failed = 0
  succeeded = 0
  if status.success?
    puts "success @#{bytes_per_second}"
    succeeded += 1
  else
    puts "failed  @#{bytes_per_second}"
    failed += 1
  end

  if stdout_and_stderr_str.match?(nargo_panic_str)
    puts
    puts
    puts "------------------------------------------------------------------------------------"
    p input_path
    puts
    p program_str
    puts
    p stdout_and_stderr_str
    puts
    puts "------------------------------------------------------------------------------------"
    raise "#{input_path} resulted in a panic!"
  end

  [program_str.size, total_seconds, failed, succeeded]
end

total_size = 0
total_seconds = 0.0
total_failed = 0
total_succeeded = 0
Dir["./**/*.rs"].each do |input_path|
  next_size, next_seconds, next_failed, next_succeeded = run_nargo input_path
  total_size += next_size
  total_seconds += next_seconds
  total_failed += next_failed
  total_succeeded += next_succeeded
end

bytes_per_second = total_size / total_seconds
puts "total_size: #{total_size}"
puts "total_seconds: #{total_seconds}"
puts "bytes_per_second: #{bytes_per_second}"
puts "total_failed: #{total_failed}"
puts "total_succeeded: #{total_succeeded}"

