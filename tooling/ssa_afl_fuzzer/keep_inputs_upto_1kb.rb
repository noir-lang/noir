#!/bin/env ruby

require 'fileutils'

num_removed = 0
Dir['./inputs/*.ssa'].each do |input_path|
  if File.read(input_path).length > 1024
    puts "removing #{input_path}"
    FileUtils.rm input_path
    num_removed += 1
  end
end

puts "removed #{num_removed} files"
