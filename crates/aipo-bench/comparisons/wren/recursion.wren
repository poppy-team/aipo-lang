class Fibonacci {
  static compute(n) {
    if (n < 2) return n
    return Fibonacci.compute(n - 1) + Fibonacci.compute(n - 2)
  }
}

System.print("checksum:%(Fibonacci.compute(24))")
