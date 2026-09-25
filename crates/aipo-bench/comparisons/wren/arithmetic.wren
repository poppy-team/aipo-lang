import "os" for Process

class Step {
  static compute(i) {
    return i * 3 - 1
  }
}

var n = Num.fromString(Process.arguments[0])
var total = 0
for (i in 0...n) {
  total = total + Step.compute(i)
}
System.print("checksum:%(total)")
