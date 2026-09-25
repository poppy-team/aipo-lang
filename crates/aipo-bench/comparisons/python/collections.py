import sys

n = int(sys.argv[1])
values = []
for i in range(n):
    values.append(i)
total = 0
for value in values:
    total += value
print(f"checksum:{total}")
