import "os" for Process

var n = Num.fromString(Process.arguments[0])
var value = ""
for (i in 0...n) {
  value = value + "ab"
}
System.print("checksum:%(value.count)")
