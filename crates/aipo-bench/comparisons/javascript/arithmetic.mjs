const n = Number(process.argv[2]);
function step(i) {
  return i * 3 - 1;
}
let total = 0;
for (let i = 0; i < n; i += 1) {
  total += step(i);
}
console.log(`checksum:${total}`);
