#!/bin/env ruby

# this script prints the unique locations where crashes occur

Dir['unique_crashes/*.out'].flat_map do |unique_crash_path|
  File.read(unique_crash_path).lines.flat_map do |unique_crash_line|
    unique_crash_line.match(/thread 'main' panicked at (?<unique_panic_location>.*):$/) || []
  end.map do |matched_line|
    matched_line['unique_panic_location']
  end
end.sort.uniq.each do |unique_panic_location|
  puts unique_panic_location
end
