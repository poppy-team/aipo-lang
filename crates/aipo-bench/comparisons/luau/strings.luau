local n = tonumber((...))
local value = ""
for _ = 1, n do
  value = value .. "ab"
end
print("checksum:" .. #value)
