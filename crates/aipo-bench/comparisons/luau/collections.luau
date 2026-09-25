local n = tonumber((...))
local values = {}
for i = 0, n - 1 do
  table.insert(values, i)
end
local total = 0
for _, value in ipairs(values) do
  total = total + value
end
print("checksum:" .. total)
