// Shim self-test: unit properties of the pure value/stdlib layer.
// Run with `node selftest.mjs` (requires Node >= 20). Machine-level behavior
// (frames, handlers, journals) is covered by the VM<->JS differential suite.
import assert from 'node:assert/strict';
import * as R from './aipo-runtime.js';

function throwsCode(fn, code) {
  try {
    fn();
  } catch (e) {
    assert.equal(e.code, code, `expected ${code}, got ${e.code}: ${e.message}`);
    return;
  }
  assert.fail(`expected fault ${code}, but nothing threw`);
}

// --- numerics: range, finiteness, div-zero ---
assert.equal(R.vInt(9007199254740991).v, 9007199254740991);
throwsCode(() => R.vInt(9007199254740992), 'AIPO_RT_OVERFLOW');
throwsCode(() => R.valAdd(R.vInt(9007199254740991), R.vInt(1)), 'AIPO_RT_OVERFLOW');
throwsCode(() => R.valDiv(R.vInt(1), R.vInt(0)), 'AIPO_RT_DIV_ZERO');
throwsCode(() => R.valIntDiv(R.vInt(1), R.vInt(0)), 'AIPO_RT_DIV_ZERO');
assert.equal(R.valDiv(R.vInt(1), R.vInt(2)).v, 0.5);
assert.equal(R.display(R.valDiv(R.vInt(2), R.vInt(1))), '2.0');
assert.equal(R.valIntDiv(R.vInt(7), R.vInt(2)).v, 3);
assert.equal(R.valMod(R.vInt(7), R.vInt(3)).v, 1);
// Byte widens: Byte(200) + Byte(100) exceeds Byte but fits Int.
assert.equal(R.valAdd(R.vByte(200), R.vByte(100)).v, 300);

// --- strings: code points, NFC, tolerant slices ---
assert.equal(R.std_string_len(R.vStr('é')).v, 1);
assert.equal(R.std_string_byte_len(R.vStr('é')).v, 2);
assert.equal(R.vStr('é').v, 'é', 'decomposed input normalizes at construction');
assert.equal(R.valAdd(R.vStr('e'), R.vStr('́')).v, 'é', 'concat normalizes');
assert.equal(R.std_string_reverse(R.vStr('́e')).v, 'é');
assert.equal(R.std_string_slice(R.vStr('hello'), R.vInt(-10), R.vInt(99)).v, 'hello');
assert.equal(R.std_string_slice(R.vStr('hello'), R.vInt(3), R.vInt(1)).v, '');
throwsCode(() => R.std_string_split(R.vStr('a'), R.vStr('')), 'AIPO_RT_TYPE_MISMATCH');
throwsCode(() => R.std_string_join(R.vStr(','), R.vList([R.vInt(1)])), 'AIPO_RT_TYPE_MISMATCH');
assert.equal(R.std_string_find(R.vStr('abc'), R.vStr('')).v, 0);
assert.ok(R.std_string_find(R.vStr('abc'), R.vStr('z')).t === 'none');
assert.equal(R.std_string_capitalize(R.vStr('123 abc')).v, '123 Abc');

// --- format: recoverable, never a fault ---
assert.equal(R.std_string_format(R.vStr('hi {x}'), R.vDict([])).msg, 'missing format value for placeholder {x}');
assert.equal(R.std_string_format(R.vStr('hi {0}'), R.vDict([])).msg, 'unsupported placeholder {0}: only simple names are allowed');
assert.equal(R.std_string_format(R.vStr('hi {x'), R.vDict([])).msg, "malformed format template: unclosed '{'");

// --- math: Failure channel ---
assert.equal(R.std_math_sqrt(R.vInt(-1)).msg, 'cannot compute square root of negative number');
assert.ok(R.std_math_clamp(R.vInt(5), R.vInt(3), R.vInt(0)).msg.startsWith('math.clamp bounds are inverted'));
assert.equal(R.std_math_round(R.vFloat(2.5)).v, 3);
assert.equal(R.std_math_round(R.vFloat(-2.5)).v, -3);

// --- conversions: exact messages ---
assert.equal(R.convInt(R.vStr('not a number')).msg, 'invalid integer text: "not a number"');
assert.equal(R.convFloat(R.vStr('xx')).msg, 'invalid float text: "xx"');
assert.equal(R.convByte(R.vInt(256)).msg, 'Byte value 256 is outside the range 0..=255');
assert.equal(R.convBytes(R.vInt(-1)).msg, 'Bytes(-1) is outside the constructible range 0..=67108864');

// --- equality: numeric cross-type, structural collections ---
assert.ok(R.valuesEqual(R.vInt(1), R.vFloat(1.0)));
assert.ok(R.valuesEqual(R.vByte(2), R.vInt(2)));
assert.ok(!R.valuesEqual(R.vInt(1), R.vStr('1')));
assert.ok(R.valuesEqual(R.vList([R.vInt(1)]), R.vList([R.vInt(1)])));
assert.ok(!R.valuesEqual(R.vList([R.vInt(1)]), R.vList([R.vInt(2)])));

// --- dict: insertion order preserved ---
const d = R.vDict([[R.vStr('b'), R.vInt(2)], [R.vStr('a'), R.vInt(1)]]);
assert.deepEqual(R.dictNatives.keys(d, []).items.map(k => k.v), ['b', 'a']);
assert.equal(R.dictNatives.get(d, [R.vStr('a')]).v, 1);
assert.ok(R.dictNatives.get(d, [R.vStr('zzz')]).t === 'none');

// --- list: negative indices, tolerant ranges via valGetIndex ---
const l = R.vList([R.vInt(1), R.vInt(2), R.vInt(3)]);
assert.equal(R.valGetIndex(l, R.vInt(-1), []).v, 3);
throwsCode(() => R.valGetIndex(l, R.vInt(9), []), 'AIPO_RT_INDEX_OUT_OF_RANGE');
assert.equal(R.valGetIndex(l, R.vRange(1, 99), []).items.length, 2);
assert.equal(R.valLen(R.vRange(3, 3)).v, 0);

// --- Failure propagation vs faults ---
const f = R.vFail('x');
assert.ok(R.valAdd(f, R.vInt(1)) === f);
assert.ok(R.valNot(f) === f);
assert.equal(R.structGetField(f, 'message').v, 'x');
console.log('shim selftest: all assertions passed');
