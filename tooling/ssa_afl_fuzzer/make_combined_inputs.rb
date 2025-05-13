#!/bin/env ruby

require 'fileutils'
require 'pathname'

input_dirs = [
  'remove_unreachable_functions_inputs',
  'defunctionalize_inputs',
  'inline_simple_inputs',
]

unless Dir.exist? './combined_inputs/'
  FileUtils.mkdir './combined_inputs'
end

Dir["./combined_inputs/*.ssa"].each do |combined_input_path|
  FileUtils.rm_f combined_input_path
end

input_dirs.each_with_index do |input_dir, input_dir_index|
  Dir["./#{input_dir}/*.ssa"].each do |input_ssa|
    new_path = Pathname.new('./combined_inputs') + Pathname.new(input_ssa).basename
    pow2_index = 2 ** input_dir_index
    new_contents = "// #{pow2_index}\n" + File.read(input_ssa)
    puts "copying to #{new_path}"
    File.write(new_path, new_contents)
  end
end
