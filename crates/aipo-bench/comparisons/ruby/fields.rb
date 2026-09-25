n = Integer(ARGV.fetch(0))

class Box
  def initialize
    @a = 0
    @b = 0
    @c = 0
    @d = 0
    @e = 0
    @f = 0
  end

  def advance(i)
    @a = (@b + i) % 1000
    @b = (@c + @a) % 1000
    @c = (@d + @b) % 1000
    @d = (@e + @c) % 1000
    @e = (@f + @d) % 1000
    @f = (@a + 1) % 1000
  end

  def score
    @a + 2 * @b + 3 * @c + 4 * @d + 5 * @e + 6 * @f
  end
end

box = Box.new
i = 0
total = 0
while i < n
  box.advance(i)
  total += box.score
  i += 1
end
puts("checksum:#{total}")
