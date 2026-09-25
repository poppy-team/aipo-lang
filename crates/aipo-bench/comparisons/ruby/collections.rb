n = Integer(ARGV.fetch(0))
values = []
n.times { |i| values << i }
total = 0
values.each { |value| total += value }
puts "checksum:#{total}"
