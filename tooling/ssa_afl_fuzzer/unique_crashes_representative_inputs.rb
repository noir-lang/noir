#!/bin/env ruby

require 'fileutils'

# this script finds unique locations where crashes occur and
# copies the smallest input file (along with its output)
# to './unique_crash_location_representatives/'

unique_crash_path_and_locations = Dir['unique_crashes/*.out'].flat_map do |unique_crash_path|
  File.read(unique_crash_path).lines.flat_map do |unique_crash_line|
    unique_crash_line.match(/thread 'main' panicked at (?<unique_panic_location>.*):$/) || []
  end.map do |matched_line|
    matched_line['unique_panic_location']
  end.map do |unique_panic_location|
    [unique_crash_path, unique_panic_location]
  end
end

# just the locations
unique_panic_locations = unique_crash_path_and_locations.map do |path, panic_location|
  panic_location
end.sort.uniq

unless Dir.exist? './unique_crash_location_representatives'
  FileUtils.mkdir './unique_crash_location_representatives'
end

# cleanup previous results
Dir['./unique_crash_location_representatives/*'].each do |unique_crash_location_representative|
  FileUtils.rm_rf unique_crash_location_representative
end

unique_panic_locations.each do |unique_panic_location|
  smallest_representative_output = unique_crash_path_and_locations.select do |path, panic_location|
    panic_location == unique_panic_location
  end.sort_by do |output_path, panic_location|
    input_path = output_path.sub '.out', ''
    File.read(input_path).length
  end.map do |output_path, panic_location|
    output_path
  end.first

  smallest_representative_input = smallest_representative_output.sub '.out', ''

  puts "#{unique_panic_location} has: \n#{smallest_representative_input}"
  target_input_filename = unique_panic_location.gsub('/', '__').tr(':', '_') + '.ssa'
  target_output_filename = target_input_filename.sub '.ssa', '.out'

  puts "copying #{smallest_representative_input} to ./unique_crash_location_representatives/#{target_input_filename}"
  FileUtils.cp smallest_representative_input, "./unique_crash_location_representatives/#{target_input_filename}"

  puts "copying #{smallest_representative_output} to ./unique_crash_location_representatives/#{target_output_filename}"
  FileUtils.cp smallest_representative_output, "./unique_crash_location_representatives/#{target_output_filename}"

  puts "#{'-' * 80}"
end
