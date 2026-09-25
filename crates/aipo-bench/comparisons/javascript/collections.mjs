const n = Number(process.argv[2]);
const values = [];
for (let i = 0; i < n; i += 1) {
  values.push(i);
}
let total = 0;
for (const value of values) {
  total += value;
}
console.log(`checksum:${total}`);
