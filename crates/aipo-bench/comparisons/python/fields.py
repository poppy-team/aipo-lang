import sys

n = int(sys.argv[1])

class Box:
    __slots__ = ("a", "b", "c", "d", "e", "f")

    def __init__(self):
        self.a = 0
        self.b = 0
        self.c = 0
        self.d = 0
        self.e = 0
        self.f = 0

    def advance(self, i):
        self.a = (self.b + i) % 1000
        self.b = (self.c + self.a) % 1000
        self.c = (self.d + self.b) % 1000
        self.d = (self.e + self.c) % 1000
        self.e = (self.f + self.d) % 1000
        self.f = (self.a + 1) % 1000

    def score(self):
        return self.a + 2 * self.b + 3 * self.c + 4 * self.d + 5 * self.e + 6 * self.f

box = Box()
i = 0
total = 0
while i < n:
    box.advance(i)
    total += box.score()
    i += 1
print(f"checksum:{total}")
