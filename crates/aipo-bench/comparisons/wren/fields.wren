import "os" for Process

class Box {
  construct new() {
    _a = 0
    _b = 0
    _c = 0
    _d = 0
    _e = 0
    _f = 0
  }

  advance(i) {
    _a = (_b + i) % 1000
    _b = (_c + _a) % 1000
    _c = (_d + _b) % 1000
    _d = (_e + _c) % 1000
    _e = (_f + _d) % 1000
    _f = (_a + 1) % 1000
  }

  score() {
    return _a + 2 * _b + 3 * _c + 4 * _d + 5 * _e + 6 * _f
  }
}

var n = Num.fromString(Process.arguments[0])
var box = Box.new()
var i = 0
var total = 0
while (i < n) {
  box.advance(i)
  total = total + box.score()
  i = i + 1
}
System.print("checksum:%(total)")
