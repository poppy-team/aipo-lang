n = Integer(ARGV.fetch(0))
value = +""
n.times { value << "ab" }
puts "checksum:#{value.length}"
