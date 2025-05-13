#!/usr/bin/env ruby

# drop header line
`ps -ef`.lines.drop(1).map do |ps_line|
  ps_columns = ps_line.split(' ');
  [ps_columns[-1], ps_columns[1]]
end.select do |process_name, pid|
  process_name.match?(/afl-fuzz/) || process_name.match?(/ssa_afl_fuzzer/)
end.each do |process_name, pid|
  puts "kill -9 #{pid} # #{process_name}"
  system("kill -9 #{pid}")
end

