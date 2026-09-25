import sys

n = int(sys.argv[1])

def step(i):
    return i * 3 - 1

total = 0
for i in range(n):
    total += step(i)
print(f"checksum:{total}")
