import "os" for Process

var n = Num.fromString(Process.arguments[0])
var values = []
for (i in 0...n) {
  values.add(i)
}
var total = 0
for (value in values) {
  total = total + value
}
System.print("checksum:%(total)")
