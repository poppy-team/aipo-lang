local n = tonumber((...))
local function step(i)
  return i * 3 - 1
end
local total = 0
for i = 0, n - 1 do
  total = total + step(i)
end
print("checksum:" .. total)
