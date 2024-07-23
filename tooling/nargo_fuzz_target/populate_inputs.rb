#!/usr/bin/env ruby

require 'fileutils'

# target_path = 'test_programs'
target_path = 'noir_stdlib'

puts "Copying all #{target_path} to inputs (renamed).."
num_inputs = 0

Dir["../../#{target_path}/**/*.nr"].each do |path|
  new_filename = path
    .sub(/^\.\.\/\.\.\/test_programs\//, '')
    .gsub(/\//, "__")

  unless path.match?(/benchmark/)
    puts path
    FileUtils.cp path, "inputs/#{new_filename}"
    num_inputs += 1
  end
end

puts "#{num_inputs} inputs collected"
