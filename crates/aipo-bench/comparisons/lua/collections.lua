local n = tonumber(arg[1])
local values = {}
for i = 0, n - 1 do
    values[#values + 1] = i
end
local total = 0
for _, value in ipairs(values) do
    total = total + value
end
print("checksum:" .. total)
