import sys

n = int(sys.argv[1])
value = ""
for _ in range(n):
    value += "ab"
print(f"checksum:{len(value)}")
