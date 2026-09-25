const n = Number(process.argv[2]);
let value = "";
for (let i = 0; i < n; i += 1) {
  value += "ab";
}
console.log(`checksum:${value.length}`);
