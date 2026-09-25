const n = Number(process.argv[2]);

class Box {
  constructor() {
    this.a = 0;
    this.b = 0;
    this.c = 0;
    this.d = 0;
    this.e = 0;
    this.f = 0;
  }

  advance(i) {
    this.a = (this.b + i) % 1000;
    this.b = (this.c + this.a) % 1000;
    this.c = (this.d + this.b) % 1000;
    this.d = (this.e + this.c) % 1000;
    this.e = (this.f + this.d) % 1000;
    this.f = (this.a + 1) % 1000;
  }

  score() {
    return this.a + 2 * this.b + 3 * this.c + 4 * this.d + 5 * this.e + 6 * this.f;
  }
}

const box = new Box();
let i = 0;
let total = 0;
while (i < n) {
  box.advance(i);
  total += box.score();
  i += 1;
}
console.log(`checksum:${total}`);
