#!/usr/bin/env ruby

require 'fileutils'
require 'open3'
require 'timeout'


# TODO: timeout? this appears to have been caused by rebuilding 'ssa_afl_fuzzer'
#
# attempting reproduction on ./collected_outputs/id:000255,sig:09,src:003970,time:82662864,execs:226910830,op:colorization,rep:2
# ^C#<Thread:0x00007ffff78da848 /usr/lib/ruby/3.2.0/open3.rb:404 run> terminated with exception (report_on_exception is true):
# /usr/lib/ruby/3.2.0/open3.rb:404:in `read': stream closed in another thread (IOError)
# 	from /usr/lib/ruby/3.2.0/open3.rb:404:in `block (2 levels) in capture2e'
# /usr/lib/ruby/3.2.0/open3.rb:416:in `value': Interrupt
# 	from /usr/lib/ruby/3.2.0/open3.rb:416:in `block in capture2e'
# 	from /usr/lib/ruby/3.2.0/open3.rb:228:in `popen_run'
# 	from /usr/lib/ruby/3.2.0/open3.rb:210:in `popen2e'
# 	from /usr/lib/ruby/3.2.0/open3.rb:399:in `capture2e'
# 	from ./collect_unique_crashes.rb:11:in `run_nargo'
# 	from ./collect_unique_crashes.rb:37:in `block in <main>'
# 	from ./collect_unique_crashes.rb:35:in `map'
# 	from ./collect_unique_crashes.rb:35:in `<main>'


def run_nargo(program_path, program_str)
  target_path = "../../target/debug/ssa_afl_fuzzer"
  cmd_str = "cargo afl run #{target_path} < #{program_path}"

  stdout_and_stderr_str = nil
  status = nil
  # timeout after 5 seconds
  begin
    Timeout::timeout(5) do
      # If opts[:stdin_data] is specified, it is sent to the command’s standard input.
      stdout_and_stderr_str, status = Open3.capture2e(cmd_str) # , stdin_data: program_str)
    end
  rescue Timeout::Error => e
    stdout_and_stderr_str = "#{e}"
  end

  if !status.nil? && status.success?
    puts "unexpected successful input: #{program_path}"
    nil
  else
    stdout_and_stderr_str
  end
end

# make output dir's
unless Dir.exist? './collected_outputs'
  FileUtils.mkdir './collected_outputs'
else
  # clear previous results
  Dir['./collected_outputs/*'].each do |collected_output|
    FileUtils.rm_rf collected_output
  end
end
unless Dir.exist? './unique_crashes'
  FileUtils.mkdir './unique_crashes'
else
  # clear previous results
  Dir['./unique_crashes/*'].each do |unique_crash|
    FileUtils.rm_rf unique_crash
  end
end

# update ./collected_outputs
system "./collect_outputs.rb ./collected_outputs"
puts

# rebuild fuzz target (in case it updated)
system "cargo afl build"

num_crashes = 0
Dir['./collected_outputs/*'].map do |collected_output_path|
  [collected_output_path, File.read(collected_output_path)]
end.flat_map do |collected_output_path, collected_output_str|
  puts "attempting reproduction on #{collected_output_path}"
  nargo_output = run_nargo collected_output_path, collected_output_str
  if nargo_output.nil?
    []
  else
    [[collected_output_path, collected_output_str, nargo_output]]
  end
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
