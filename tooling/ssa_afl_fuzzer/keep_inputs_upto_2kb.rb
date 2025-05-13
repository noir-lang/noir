#!/bin/env ruby

require 'fileutils'

input_dirs = [
  'defunctionalize_inputs',
  'inline_simple_inputs',
  'remove_unreachable_functions_inputs',
]

num_removed = 0
Dir['./defunctionalize_inputs/*.ssa'].each do |input_path|
  if File.read(input_path).length > 2048
    puts "removing #{input_path}"
    FileUtils.rm input_path
    num_removed += 1
  end
end

puts "removed #{num_removed} files"
