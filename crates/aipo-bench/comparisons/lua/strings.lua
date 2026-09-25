local n = tonumber(arg[1])
local value = ""
for _ = 1, n do
    value = value .. "ab"
end
print("checksum:" .. #value)
