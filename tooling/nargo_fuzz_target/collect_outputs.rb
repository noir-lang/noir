#!/usr/bin/env ruby

require 'fileutils'

output_dirs = [
  "outputs",
  "outputs-big",
  "outputs-parallel",
  "outputs-ramdisk",
  "outputs-release",
]

collected_output_dir = ARGV[0]
if collected_output_dir.nil?
  raise "the first argument is required: [dir-for-all-outputs]"
end

num_crashes = 0
puts "copying crashing inputs to #{collected_output_dir}"

# [output_dir]/[worker_dir]/crashes/id:[..]
output_dirs.each do |output_dir|
  Dir["#{output_dir}/*"].filter do |worker_output_dir|
    if Dir.exist? worker_output_dir
      Dir["#{worker_output_dir}/crashes/id:*"].each do |crash_path|
        puts "copying #{crash_path}"
        FileUtils.cp crash_path, ARGV[0]
        num_crashes += 1
      end
    end
  end
end

puts "copied #{num_crashes} crashes."
