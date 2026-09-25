n = Integer(ARGV.fetch(0))

def step(i)
  i * 3 - 1
end

total = 0
n.times do |i|
  total += step(i)
end
puts "checksum:#{total}"
