local n = tonumber(arg[1])
local Box = {}
Box.__index = Box

function Box.advance(self, i)
  self.a = (self.b + i) % 1000
  self.b = (self.c + self.a) % 1000
  self.c = (self.d + self.b) % 1000
  self.d = (self.e + self.c) % 1000
  self.e = (self.f + self.d) % 1000
  self.f = (self.a + 1) % 1000
end

function Box.score(self)
  return self.a + 2 * self.b + 3 * self.c + 4 * self.d + 5 * self.e + 6 * self.f
end

local box = setmetatable({ a = 0, b = 0, c = 0, d = 0, e = 0, f = 0 }, Box)
local i = 0
local total = 0
while i < n do
  box:advance(i)
  total = total + box:score()
  i = i + 1
end
print("checksum:" .. total)
