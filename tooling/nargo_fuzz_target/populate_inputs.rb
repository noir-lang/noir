#!/usr/bin/env ruby

require 'fileutils'

# NOTE: current files are pared down from this one-liner
# Dir['../../test_programs/compile_success_empty/**/*.nr'].select{|x|`stat -f%z #{x}`.chomp.to_i <= 120}.sort.join(' ')

target_path = 'test_programs'
# target_path = 'noir_stdlib'

puts "Copying all #{target_path} to inputs (renamed).."
num_inputs = 0

# create target directory unless it already exists
unless Dir.exist? 'draft_inputs'
  FileUtils.mkdir 'draft_inputs'
end

Dir["../../#{target_path}/**/*.nr"].each do |path|
  new_filename = path
    .sub(/^\.\.\/\.\.\/test_programs\//, '')
    .gsub(/\//, "__")

  unless path.match?(/benchmark/) || File.read(path).length > 512
    puts path
    FileUtils.cp path, "draft_inputs/#{new_filename}"
    num_inputs += 1
  end
end

puts "#{num_inputs} inputs collected"
