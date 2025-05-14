#!/bin/env ruby

loop do
  bytes_free = `df -B1 .`.split[10].to_i
  #             302823825408
  if bytes_free < 2823825408
    system("kill -9 2543791")
  end

  puts "bytes_free: #{bytes_free}"

  # sleep for 2s
  sleep(2)
end
