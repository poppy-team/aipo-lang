// aipo-runtime.js — versioned JavaScript semantics shim for Aipo (Wave 2).
// ESM. Mirrors the Rust VM value model and Core IR interpreter semantics:
// divergence between VM and JS backends is a bug.
// RUNTIME_VERSION must match aipo-js RUNTIME_VERSION in src/lib.rs.
export const RUNTIME_VERSION = '1.0.0';

const MAX_SAFE_INT = 9007199254740991;
const MIN_SAFE_INT = -9007199254740991;
const BYTES_MAX_ALLOCATION = 67108864;

let __nextId = 1;
function freshId() { return __nextId++; }

// ---- output capture (io.print/println write here; harness overrides) ----
export let outputBuffer = [];
let useBuffer = false;
export function setOutputSink(arr) { outputBuffer = arr; useBuffer = true; }
export function resetOutputSink() { outputBuffer = []; useBuffer = false; }
function emitText(s) {
  if (useBuffer) { outputBuffer.push(String(s)); return; }
  if (typeof process !== 'undefined' && process.stdout && process.stdout.write) {
    process.stdout.write(String(s));
  } else {
    outputBuffer.push(String(s));
  }
}

// ---- faults (never capturable by attempt) ----
export class AipoFault extends Error {
  constructor(code, message) {
    super(`[${code}] ${message}`);
    this.code = code;
  }
}
function fault(code, message) { throw new AipoFault(code, message); }
function typeMismatch(expected, actual) {
  fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: expected ${expected}, got ${actual}`);
}

// ---- value constructors (tagged objects) ----
export function vNone() { return { t: 'none' }; }
export function vBool(b) { return { t: 'bool', v: !!b }; }
export function vInt(n) {
  if (!Number.isInteger(n)) fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: expected Int, got ${n}`);
  return checkSafeInt(n);
}
export function vFloat(f) { return checkFiniteFloat(f); }
export function vByte(b) {
  if (!Number.isInteger(b) || b < 0 || b > 255) fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: expected Byte, got ${b}`);
  return { t: 'byte', v: b };
}
export function vStr(s) { return { t: 'str', v: String(s).normalize('NFC') }; }
export function vList(items) { return { t: 'list', items: items || [], id: freshId() }; }
function buildStrIdx(entries) {
  const idx = new Map();
  entries.forEach(([k], i) => {
    if (k.t === 'str' && !idx.has(k.v)) idx.set(k.v, i);
  });
  return idx;
}
function rebuildStrIdx(d) { d.strIdx = buildStrIdx(d.entries); }
export function vDict(entries) {
  const list = entries || [];
  return { t: 'dict', entries: list, strIdx: buildStrIdx(list), id: freshId() };
}
export function dictGet(d, key) {
  if (key.t === 'str') {
    const at = d.strIdx.get(key.v);
    if (at === undefined) return null;
    return d.entries[at][1];
  }
  const found = d.entries.find(([k]) => valuesEqual(k, key));
  return found ? found[1] : null;
}
export function dictUpsert(d, key, value) {
  if (key.t === 'str') {
    const at = d.strIdx.get(key.v);
    if (at !== undefined && d.entries[at]) {
      d.entries[at][1] = value;
      return;
    }
  } else {
    const found = d.entries.find(([k]) => valuesEqual(k, key));
    if (found) {
      found[1] = value;
      return;
    }
  }
  if (key.t === 'str' && !d.strIdx.has(key.v)) d.strIdx.set(key.v, d.entries.length);
  d.entries.push([key, value]);
}
export function dictRemove(d, key) {
  let at = -1;
  if (key.t === 'str') {
    const found = d.strIdx.get(key.v);
    if (found !== undefined) at = found;
  } else {
    at = d.entries.findIndex(([k]) => valuesEqual(k, key));
  }
  if (at < 0) return false;
  d.entries.splice(at, 1);
  rebuildStrIdx(d);
  return true;
}
export function dictClear(d) { d.entries.length = 0; d.strIdx.clear(); }
export function vBytes(data) { return { t: 'bytes', data: data instanceof Uint8Array ? data : new Uint8Array(data || []) }; }
export function vRange(start, end) { return { t: 'range', start, end }; }
export function vType(name) { return { t: 'type', name }; }
export function vFail(msg) { return { t: 'fail', msg: String(msg) }; }
export function vUnset() { return { t: 'unset' }; }
function vStruct(type, fields, fixed, constructing) {
  return { t: 'struct', type, fields: fields || [], fixed: new Set(fixed || []), constructing: !!constructing, id: freshId() };
}

export function isFailure(v) { return v && v.t === 'fail'; }
export function isUnset(v) { return v && v.t === 'unset'; }

export function typeName(v) {
  switch (v.t) {
    case 'none': return 'none';
    case 'bool': return 'Bool';
    case 'int': return 'Int';
    case 'float': return 'Float';
    case 'str': return 'String';
    case 'list': return 'List';
    case 'dict': return 'Dict';
    case 'struct': return 'struct';
    case 'func': case 'closure': case 'native': case 'bound': return 'Function';
    case 'byte': return 'Byte';
    case 'bytes': return 'Bytes';
    case 'type': return 'Type';
    case 'range': return 'Range';
    case 'fail': return 'Failure';
    default: return '<omitted argument>';
  }
}

function structDisplayName(v) {
  if (v && v.t === 'struct') return v.type;
  return typeName(v);
}

export function display(v) {
  switch (v.t) {
    case 'none': return 'none';
    case 'bool': return v.v ? 'true' : 'false';
    case 'int': return String(v.v);
    case 'float': return Number.isInteger(v.v) ? `${v.v}.0` : String(v.v);
    case 'byte': return String(v.v);
    case 'str': return v.v;
    case 'list': return `[${v.items.map(display).join(', ')}]`;
    case 'dict': return `#{${v.entries.map(([k, val]) => `${display(k)}: ${display(val)}`).join(', ')}}`;
    case 'struct': return `${v.type}{${v.fields.map(([k, val]) => `${k} = ${display(val)}`).join(', ')}}`;
    case 'bytes': return '<bytes>';
    case 'type': return v.name;
    case 'range': return `${v.start}..${v.end}`;
    case 'fail': return `fail("${v.msg}")`;
    case 'func': case 'closure': case 'native': case 'bound': return '<function>';
    default: return '<omitted argument>';
  }
}

// ---- numeric guards ----
function checkSafeInt(n) {
  if (!Number.isInteger(n) || n < MIN_SAFE_INT || n > MAX_SAFE_INT) {
    fault('AIPO_RT_OVERFLOW', `${n} exceeds integer range ±(2^53 - 1)`);
  }
  return { t: 'int', v: n };
}
function checkFiniteFloat(f) {
  if (typeof f !== 'number' || !Number.isFinite(f)) fault('AIPO_RT_NON_FINITE_FLOAT', 'non-finite float');
  return { t: 'float', v: f };
}
function widen(v) { return v.t === 'byte' ? { t: 'int', v: v.v } : v; }
function needsWiden(a, b) { return a.t === 'byte' || b.t === 'byte'; }
function toF64(v) {
  if (v.t === 'int' || v.t === 'byte') return v.v;
  if (v.t === 'float') return v.v;
  return NaN;
}

// ---- arithmetic (Failure propagates, left priority) ----
function arithPre(a, b) {
  if (isFailure(a)) return a;
  if (isFailure(b)) return b;
  return null;
}
export function valAdd(a, b) {
  const p = arithPre(a, b); if (p) return p;
  if (needsWiden(a, b)) return valAdd(widen(a), widen(b));
  if (a.t === 'int' && b.t === 'int') return checkSafeInt(a.v + b.v);
  if (a.t === 'float' && b.t === 'float') return checkFiniteFloat(a.v + b.v);
  if (a.t === 'int' && b.t === 'float') return checkFiniteFloat(a.v + b.v);
  if (a.t === 'float' && b.t === 'int') return checkFiniteFloat(a.v + b.v);
  if (a.t === 'str' && b.t === 'str') return vStr(a.v + b.v);
  if (a.t === 'list' && b.t === 'list') return vList([...a.items, ...b.items]);
  return typeMismatch('Int, Float, String, or List', `${typeName(a)} and ${typeName(b)}`);
}
export function valSub(a, b) {
  const p = arithPre(a, b); if (p) return p;
  if (needsWiden(a, b)) return valSub(widen(a), widen(b));
  if (a.t === 'int' && b.t === 'int') return checkSafeInt(a.v - b.v);
  if (a.t === 'float' && b.t === 'float') return checkFiniteFloat(a.v - b.v);
  if (a.t === 'int' && b.t === 'float') return checkFiniteFloat(a.v - b.v);
  if (a.t === 'float' && b.t === 'int') return checkFiniteFloat(a.v - b.v);
  return typeMismatch('Int or Float', `${typeName(a)} and ${typeName(b)}`);
}
export function valMul(a, b) {
  const p = arithPre(a, b); if (p) return p;
  if (needsWiden(a, b)) return valMul(widen(a), widen(b));
  if (a.t === 'int' && b.t === 'int') return checkSafeInt(a.v * b.v);
  if (a.t === 'float' && b.t === 'float') return checkFiniteFloat(a.v * b.v);
  if (a.t === 'int' && b.t === 'float') return checkFiniteFloat(a.v * b.v);
  if (a.t === 'float' && b.t === 'int') return checkFiniteFloat(a.v * b.v);
  return typeMismatch('Int or Float', `${typeName(a)} and ${typeName(b)}`);
}
export function valDiv(a, b) {
  const p = arithPre(a, b); if (p) return p;
  const x = toF64(widen(a)), y = toF64(widen(b));
  if (Number.isNaN(x) || Number.isNaN(y)) return typeMismatch('Int or Float', `${typeName(a)} and ${typeName(b)}`);
  if (y === 0) fault('AIPO_RT_DIV_ZERO', 'division by zero');
  return checkFiniteFloat(x / y);
}
export function valIntDiv(a, b) {
  const p = arithPre(a, b); if (p) return p;
  const x = widen(a), y = widen(b);
  if (x.t !== 'int' || y.t !== 'int') return typeMismatch('Int', `${typeName(a)} and ${typeName(b)}`);
  if (y.v === 0) fault('AIPO_RT_DIV_ZERO', 'division by zero');
  return checkSafeInt(Math.trunc(x.v / y.v));
}
export function valMod(a, b) {
  const p = arithPre(a, b); if (p) return p;
  const x = widen(a), y = widen(b);
  if (x.t !== 'int' || y.t !== 'int') return typeMismatch('Int', `${typeName(a)} and ${typeName(b)}`);
  if (y.v === 0) fault('AIPO_RT_DIV_ZERO', 'division by zero');
  return checkSafeInt(x.v % y.v);
}
export function valNeg(a) {
  if (isFailure(a)) return a;
  const x = widen(a);
  if (x.t === 'int') return checkSafeInt(-x.v);
  if (x.t === 'float') return checkFiniteFloat(-x.v);
  return typeMismatch('Int or Float', typeName(a));
}
export function valPos(a) {
  if (isFailure(a)) return a;
  const x = widen(a);
  if (x.t === 'int' || x.t === 'float') return a;
  return typeMismatch('Int or Float', typeName(a));
}
export function valNot(a) {
  if (isFailure(a)) return a;
  if (a.t !== 'bool') return typeMismatch('Bool', typeName(a));
  return vBool(!a.v);
}

// ---- equality & comparison ----
export function valEqual(a, b) {
  if (isFailure(a)) return a;
  if (isFailure(b)) return b;
  return vBool(valuesEqual(a, b));
}
export function valuesEqual(a, b) {
  if (a.t !== b.t) {
    if ((a.t === 'int' || a.t === 'float' || a.t === 'byte') && (b.t === 'int' || b.t === 'float' || b.t === 'byte')) {
      return toF64(a) === toF64(b);
    }
    return false;
  }
  switch (a.t) {
    case 'none': return true;
    case 'bool': return a.v === b.v;
    case 'int': case 'byte': case 'float': return a.v === b.v;
    case 'str': return a.v === b.v;
    case 'bytes': { if (a.data.length !== b.data.length) return false; for (let i = 0; i < a.data.length; i++) if (a.data[i] !== b.data[i]) return false; return true; }
    case 'type': return a.name === b.name;
    case 'range': return a.start === b.start && a.end === b.end;
    case 'list': { if (a.items.length !== b.items.length) return false; for (let i = 0; i < a.items.length; i++) if (!valuesEqual(a.items[i], b.items[i])) return false; return true; }
    case 'dict': { if (a.entries.length !== b.entries.length) return false; for (let i = 0; i < a.entries.length; i++) if (!valuesEqual(a.entries[i][0], b.entries[i][0]) || !valuesEqual(a.entries[i][1], b.entries[i][1])) return false; return true; }
    case 'struct': { if (a.type !== b.type || a.fields.length !== b.fields.length) return false; for (let i = 0; i < a.fields.length; i++) if (a.fields[i][0] !== b.fields[i][0] || !valuesEqual(a.fields[i][1], b.fields[i][1])) return false; return true; }
    case 'fail': return a.msg === b.msg;
    case 'func': return a.idx === b.idx;
    case 'closure': return a.idx === b.idx && a.cells === b.cells;
    case 'native': return a.name === b.name;
    case 'bound': return a.name === b.name && a.arity === b.arity && valuesEqual(a.recv, b.recv);
    default: return false;
  }
}
function cmpOrder(a, b) {
  const an = (a.t === 'int' || a.t === 'byte' || a.t === 'float') ? toF64(a) : null;
  const bn = (b.t === 'int' || b.t === 'byte' || b.t === 'float') ? toF64(b) : null;
  if (an !== null && bn !== null) return an < bn ? -1 : an > bn ? 1 : 0;
  if (a.t === 'str' && b.t === 'str') return a.v < b.v ? -1 : a.v > b.v ? 1 : 0;
  if (a.t === 'bool' && b.t === 'bool') return a.v === b.v ? 0 : !a.v && b.v ? -1 : 1;
  return null;
}
export function valLess(a, b) {
  if (isFailure(a)) return a;
  if (isFailure(b)) return b;
  const c = cmpOrder(widen(a), widen(b));
  if (c === null) return typeMismatch('comparable Int, Float, or String', `${typeName(a)} and ${typeName(b)}`);
  return vBool(c < 0);
}
export function valLessEqual(a, b) {
  if (isFailure(a)) return a;
  if (isFailure(b)) return b;
  const c = cmpOrder(widen(a), widen(b));
  if (c === null) return typeMismatch('comparable Int, Float, or String', `${typeName(a)} and ${typeName(b)}`);
  return vBool(c <= 0);
}
export function valGreater(a, b) { return valLess(b, a); }
export function valGreaterEqual(a, b) { return valLessEqual(b, a); }
export function valNotEqual(a, b) {
  const e = valEqual(a, b);
  if (isFailure(e)) return e;
  return vBool(!e.v);
}

// ---- strings: all code-point based; NFC invariant at construction boundaries ----
function chars(s) { return Array.from(s); }
function nfc(s) { return s.normalize('NFC'); }
function reqStr(v, what) {
  if (isFailure(v)) return v;
  if (v.t !== 'str') typeMismatch('String', typeName(v));
  return null;
}
function sLen(s) { return chars(s.v).length; }
function sByteLen(s) { return new TextEncoder().encode(s.v).length; }
function strSlice(text, start, end) {
  const c = chars(text);
  const len = c.length;
  let a = start < 0 ? len + start : start;
  let b = end < 0 ? len + end : end;
  a = Math.max(0, Math.min(len, a));
  b = Math.max(0, Math.min(len, b));
  if (a >= b) return '';
  return c.slice(a, b).join('');
}
export function std_string_len(a) {
  if (isFailure(a)) return a;
  if (a.t !== 'str') return typeMismatch('String', typeName(a));
  return vInt(sLen(a));
}
export function std_string_byte_len(a) {
  if (isFailure(a)) return a;
  if (a.t !== 'str') return typeMismatch('String', typeName(a));
  return vInt(sByteLen(a));
}
export function std_string_contains(t, s) {
  if (isFailure(t)) return t;
  if (isFailure(s)) return s;
  reqStr(t); reqStr(s);
  return vBool(t.v.includes(s.v));
}
export function std_string_starts_with(t, p) {
  if (isFailure(t)) return t;
  if (isFailure(p)) return p;
  reqStr(t); reqStr(p);
  return vBool(t.v.startsWith(p.v));
}
export function std_string_ends_with(t, s) {
  if (isFailure(t)) return t;
  if (isFailure(s)) return s;
  reqStr(t); reqStr(s);
  return vBool(t.v.endsWith(s.v));
}
export function std_string_find(t, sub) {
  if (isFailure(t)) return t;
  if (isFailure(sub)) return sub;
  reqStr(t); reqStr(sub);
  if (sub.v === '') return vInt(0);
  const idx = t.v.indexOf(sub.v);
  if (idx < 0) return vNone();
  return vInt(chars(t.v.slice(0, idx)).length);
}
export function std_string_lower(a) {
  if (isFailure(a)) return a;
  reqStr(a);
  return vStr(nfc(a.v.toLowerCase()));
}
export function std_string_upper(a) {
  if (isFailure(a)) return a;
  reqStr(a);
  return vStr(nfc(a.v.toUpperCase()));
}
export function std_string_capitalize(a) {
  if (isFailure(a)) return a;
  reqStr(a);
  const c = chars(a.v);
  let out = '';
  let done = false;
  for (const ch of c) {
    const up = ch.toUpperCase(), lo = ch.toLowerCase();
    const cased = up !== lo;
    if (!done && cased) { out += up; done = true; }
    else if (done && cased) { out += lo; }
    else out += ch;
  }
  return vStr(nfc(out));
}
export function std_string_reverse(a) {
  if (isFailure(a)) return a;
  reqStr(a);
  let parts;
  if (typeof Intl !== 'undefined' && Intl.Segmenter) {
    const seg = new Intl.Segmenter(undefined, { granularity: 'grapheme' });
    parts = [...seg.segment(a.v)].map(s => s.segment);
  } else {
    parts = chars(a.v);
  }
  return vStr(nfc(parts.reverse().join('')));
}
export function std_string_trim(a) {
  if (isFailure(a)) return a;
  reqStr(a);
  return vStr(a.v.trim());
}
export function std_string_split(t, d) {
  if (isFailure(t)) return t;
  if (isFailure(d)) return d;
  reqStr(t); reqStr(d);
  if (d.v === '') fault('AIPO_RT_TYPE_MISMATCH', 'type mismatch: expected non-empty String separator');
  const parts = t.v.split(d.v);
  return vList(parts.map(p => vStr(p)));
}
export function std_string_join(sep, list) {
  if (isFailure(sep)) return sep;
  if (isFailure(list)) return list;
  reqStr(sep);
  if (list.t !== 'list') return typeMismatch('List', typeName(list));
  if (list.items.length === 0) return vStr('');
  for (const el of list.items) {
    if (isFailure(el)) return el;
    if (el.t !== 'str') fault('AIPO_RT_TYPE_MISMATCH', 'type mismatch: expected List of String (use String(value) to convert)');
  }
  return vStr(nfc(list.items.map(e => e.v).join(sep.v)));
}
export function std_string_replace(t, from, to) {
  if (isFailure(t)) return t;
  if (isFailure(from)) return from;
  if (isFailure(to)) return to;
  reqStr(t); reqStr(from); reqStr(to);
  if (from.v === '') fault('AIPO_RT_TYPE_MISMATCH', 'type mismatch: expected non-empty String `from`');
  return vStr(nfc(t.v.split(from.v).join(to.v)));
}
export function std_string_slice(t, s, e) {
  if (isFailure(t)) return t;
  if (isFailure(s)) return s;
  if (isFailure(e)) return e;
  reqStr(t);
  const a = widen(s), b = widen(e);
  if (a.t !== 'int' || b.t !== 'int') return typeMismatch('Int', 'slice bounds');
  return vStr(strSlice(t.v, a.v, b.v));
}
export function std_string_format(tpl, vals) {
  if (isFailure(tpl)) return tpl;
  if (isFailure(vals)) return vals;
  reqStr(tpl);
  if (vals.t !== 'dict') return typeMismatch('Dict', typeName(vals));
  const src = tpl.v;
  let out = '';
  let i = 0;
  const cps = chars(src);
  const isName = nm => /^[A-Za-z_][A-Za-z0-9_]*$/.test(nm);
  while (i < cps.length) {
    const ch = cps[i];
    if (ch === '{') {
      if (cps[i + 1] === '{') { out += '{'; i += 2; continue; }
      let name = '';
      let closed = false;
      let j = i + 1;
      while (j < cps.length) {
        if (cps[j] === '}') { closed = true; break; }
        if (cps[j] === '{') return vFail("malformed format template: nested '{' is not allowed");
        name += cps[j];
        j++;
      }
      if (!closed) return vFail("malformed format template: unclosed '{'");
      if (!isName(name)) return vFail(`unsupported placeholder {${name}}: only simple names are allowed`);
      const found = dictGet(vals, { t: 'str', v: name });
      if (found === null) return vFail(`missing format value for placeholder {${name}}`);
      if (isFailure(found)) return found;
      out += display(found);
      i = j + 1;
    } else if (ch === '}') {
      if (cps[i + 1] === '}') { out += '}'; i += 2; continue; }
      return vFail("malformed format template: unmatched '}'");
    } else {
      out += ch; i++;
    }
  }
  return vStr(nfc(out));
}

// ---- math ----
function mathNum(v) {
  if (isFailure(v)) return v;
  const x = widen(v);
  if (x.t === 'int' || x.t === 'float') return x;
  typeMismatch('Int or Float', typeName(v));
}
export function std_math_abs(x) {
  if (isFailure(x)) return x;
  const a = widen(x);
  if (a.t === 'int') {
    if (a.v === MIN_SAFE_INT) fault('AIPO_RT_OVERFLOW', 'abs overflow');
    return checkSafeInt(Math.abs(a.v));
  }
  if (a.t === 'float') return checkFiniteFloat(Math.abs(a.v));
  return typeMismatch('Int or Float', typeName(x));
}
function mathPromote(a, b) {
  const x = widen(a), y = widen(b);
  if ((x.t === 'int' || x.t === 'float') && (y.t === 'int' || y.t === 'float')) {
    if (x.t === 'float' || y.t === 'float') return [toF64(x), toF64(y), true];
    return [x.v, y.v, false];
  }
  return typeMismatch('Int or Float', `${typeName(a)} and ${typeName(b)}`);
}
export function std_math_min(a, b) {
  if (isFailure(a)) return a;
  if (isFailure(b)) return b;
  const [x, y, fl] = mathPromote(a, b);
  if (fl) return checkFiniteFloat(Math.min(x, y));
  return checkSafeInt(Math.min(x, y));
}
export function std_math_max(a, b) {
  if (isFailure(a)) return a;
  if (isFailure(b)) return b;
  const [x, y, fl] = mathPromote(a, b);
  if (fl) return checkFiniteFloat(Math.max(x, y));
  return checkSafeInt(Math.max(x, y));
}
function mathToInt(v, name) {
  const r = toF64(mathNum(v));
  if (!Number.isFinite(r)) fault('AIPO_RT_NON_FINITE_FLOAT', 'non-finite float');
  return checkSafeInt(Math[name](r));
}
export function std_math_floor(x) { if (isFailure(x)) return x; return mathToInt(x, 'floor'); }
export function std_math_ceil(x) { if (isFailure(x)) return x; return mathToInt(x, 'ceil'); }
export function std_math_truncate(x) { if (isFailure(x)) return x; return mathToInt(x, 'trunc'); }
export function std_math_round(x) {
  if (isFailure(x)) return x;
  const r = toF64(mathNum(x));
  if (!Number.isFinite(r)) fault('AIPO_RT_NON_FINITE_FLOAT', 'non-finite float');
  const q = r >= 0 ? Math.floor(r + 0.5) : Math.ceil(r - 0.5);
  return checkSafeInt(q);
}
export function std_math_sqrt(x) {
  if (isFailure(x)) return x;
  const r = toF64(mathNum(x));
  if (r < 0) return vFail('cannot compute square root of negative number');
  return checkFiniteFloat(Math.sqrt(r));
}
export function std_math_pow(b, e) {
  if (isFailure(b)) return b;
  if (isFailure(e)) return e;
  const bb = widen(b), ee = widen(e);
  if (bb.t === 'int' && ee.t === 'int' && ee.v >= 0) {
    let r = 1;
    for (let i = 0; i < ee.v; i++) {
      r *= bb.v;
      if (!Number.isSafeInteger(r) || r < MIN_SAFE_INT || r > MAX_SAFE_INT) {
        const f = Math.pow(bb.v, ee.v);
        return checkFiniteFloat(f);
      }
    }
    return checkSafeInt(r);
  }
  const f = Math.pow(toF64(bb), toF64(ee));
  return checkFiniteFloat(f);
}
export function std_math_clamp(v, mn, mx) {
  if (isFailure(v)) return v;
  if (isFailure(mn)) return mn;
  if (isFailure(mx)) return mx;
  const a = widen(v), lo = widen(mn), hi = widen(mx);
  const an = toF64(a), ln = toF64(lo), hn = toF64(hi);
  if (Number.isNaN(an) || Number.isNaN(ln) || Number.isNaN(hn)) return typeMismatch('Int or Float', 'clamp bounds');
  const fl = a.t === 'float' || lo.t === 'float' || hi.t === 'float';
  if (ln > hn) return vFail(`math.clamp bounds are inverted: ${display(mn)} > ${display(mx)}`);
  const c = Math.min(Math.max(an, ln), hn);
  return fl ? checkFiniteFloat(c) : checkSafeInt(Math.trunc(c));
}

// ---- prelude & conversions ----
export function std_len(v) {
  if (isFailure(v)) return v;
  if (v.t === 'str') return vInt(chars(v.v).length);
  if (v.t === 'list') return vInt(v.items.length);
  if (v.t === 'dict') return vInt(v.entries.length);
  if (v.t === 'bytes') return vInt(v.data.length);
  return typeMismatch('String, List, Dict, or Bytes', typeName(v));
}
export function std_copy(v) {
  if (isFailure(v)) return v;
  if (v.t === 'list') return vList([...v.items]);
  if (v.t === 'dict') return vDict(v.entries.map(([k, x]) => [k, x]));
  if (v.t === 'struct') return vStruct(v.type, v.fields.map(([k, x]) => [k, x]), [...v.fixed], v.constructing);
  return v;
}
export function std_same(a, b) {
  if (isFailure(a) || isFailure(b)) return vBool(false);
  if ((a.t === 'list' || a.t === 'dict' || a.t === 'struct') && a === b) return vBool(true);
  if (a.t === 'str' && b.t === 'str') return vBool(a.v === b.v);
  if ((a.t === 'list' || a.t === 'dict' || a.t === 'struct') || (b.t === 'list' || b.t === 'dict' || b.t === 'struct')) return vBool(false);
  return vBool(valuesEqual(a, b));
}
export function std_some(v) { return vBool(!(v.t === 'none')); }
export function std_fail(x) {
  if (isFailure(x)) return x;
  return vFail(display(x));
}
function dq(s) { return JSON.stringify(s); }
function parseIntWhole(s) {
  if (!/^[+-]?\d+$/.test(s)) return null;
  let n;
  try { n = BigInt(s); } catch { return null; }
  if (n < BigInt(Number.MIN_SAFE_INTEGER) - BigInt(1) || n > BigInt(Number.MAX_SAFE_INTEGER) + BigInt(1)) {
    // Outside i64-representable fast path: Rust parse::<i64> fails -> invalid text.
    if (n < BigInt('-9223372036854775808') || n > BigInt('9223372036854775807')) return null;
  }
  return Number(n);
}
export function convInt(v) {
  if (isFailure(v)) return v;
  const x = widen(v);
  if (x.t === 'int') return x;
  if (x.t === 'float') {
    if (!Number.isFinite(x.v)) return vFail('Int cannot represent a non-finite Float');
    const t = Math.trunc(x.v);
    if (t < MIN_SAFE_INT || t > MAX_SAFE_INT) return vFail(`Float ${String(x.v)} is outside the Int range`);
    return { t: 'int', v: t };
  }
  if (x.t === 'str') {
    const p = parseIntWhole(x.v);
    if (p === null || !Number.isSafeInteger(p)) return vFail(`invalid integer text: ${dq(x.v)}`);
    if (p < MIN_SAFE_INT || p > MAX_SAFE_INT) return vFail(`integer text out of range: ${dq(x.v)}`);
    return { t: 'int', v: p };
  }
  return typeMismatch('String, Int, Float, or Byte', typeName(v));
}
function parseFloatWhole(s) {
  if (/^[+-]?(inf|infinity|nan)$/i.test(s)) return Number(s);
  if (!/^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$/.test(s)) return null;
  return Number(s);
}
export function convFloat(v) {
  if (isFailure(v)) return v;
  const x = widen(v);
  if (x.t === 'float') return x;
  if (x.t === 'int') return checkFiniteFloat(x.v);
  if (x.t === 'str') {
    const n = parseFloatWhole(x.v);
    if (n === null || Number.isNaN(n)) return vFail(`invalid float text: ${dq(x.v)}`);
    if (!Number.isFinite(n)) return vFail(`non-finite float text: ${dq(x.v)}`);
    return { t: 'float', v: n };
  }
  return typeMismatch('String, Int, Float, or Byte', typeName(v));
}
export function convByte(v) {
  if (isFailure(v)) return v;
  const x = widen(v);
  let n = null;
  if (x.t === 'byte') return x;
  if (x.t === 'int') n = x.v;
  else if (x.t === 'float') {
    if (!Number.isFinite(x.v) || !Number.isInteger(x.v)) return vFail(`Byte requires an integral value in 0..=255, got ${String(x.v)}`);
    n = x.v;
  } else if (x.t === 'str') {
    const p = parseIntWhole(x.v);
    if (p === null || !Number.isSafeInteger(p)) return vFail(`invalid Byte text: ${dq(x.v)}`);
    n = p;
  } else return typeMismatch('String, Int, Float, or Byte', typeName(v));
  if (n < 0 || n > 255) return vFail(`Byte value ${n} is outside the range 0..=255`);
  return { t: 'byte', v: n };
}
export function convString(v) {
  if (isFailure(v)) return v;
  switch (v.t) {
    case 'str': return vStr(v.v);
    case 'int': case 'byte': return vStr(String(v.v));
    case 'float': return vStr(Number.isInteger(v.v) ? `${v.v}.0` : String(v.v));
    case 'bool': return vStr(v.v ? 'true' : 'false');
    case 'none': return vStr('none');
    case 'type': return vStr(v.name);
    default: return typeMismatch('String-convertible value', typeName(v));
  }
}
export function convBytes(v) {
  if (isFailure(v)) return v;
  if (v.t !== 'int') return typeMismatch('Int for Bytes(count)', typeName(v));
  if (v.v < 0 || v.v > BYTES_MAX_ALLOCATION) return vFail(`Bytes(${v.v}) is outside the constructible range 0..=${BYTES_MAX_ALLOCATION}`);
  return vBytes(new Uint8Array(v.v));
}

// ---- indexing / slicing ----
function rangeLen(r) { return Math.max(0, r.end - r.start); }
function normalizeSlice(len, a, b) {
  let from = a < 0 ? len + a : a;
  let to = b < 0 ? len + b : b;
  from = Math.max(0, Math.min(len, from));
  to = Math.max(0, Math.min(len, to));
  if (from >= to) return null;
  return [from, to];
}
function resolveIndex(len, idx) {
  let i = idx < 0 ? len + idx : idx;
  return i;
}
export function valGetIndex(target, index, activeIterations) {
  if (isFailure(target)) return target;
  if (isFailure(index)) return index;
  if (target.t === 'list') {
    const xi = widen(index);
    if (xi.t === 'range') {
      const r = normalizeSlice(target.items.length, xi.start, xi.end);
      if (!r) return vList([]);
      return vList(target.items.slice(r[0], r[1]));
    }
    if (xi.t !== 'int') return typeMismatch('Int index', typeName(index));
    const i = resolveIndex(target.items.length, xi.v);
    if (i < 0 || i >= target.items.length) fault('AIPO_RT_INDEX_OUT_OF_RANGE', `index ${xi.v} out of range (len ${target.items.length})`);
    return target.items[i];
  }
  if (target.t === 'str') {
    const c = chars(target.v);
    const xi = widen(index);
    if (xi.t === 'range') {
      const r = normalizeSlice(c.length, xi.start, xi.end);
      if (!r) return vStr('');
      return vStr(c.slice(r[0], r[1]).join(''));
    }
    if (xi.t !== 'int') return typeMismatch('Int index', typeName(index));
    const i = resolveIndex(c.length, xi.v);
    if (i < 0 || i >= c.length) fault('AIPO_RT_INDEX_OUT_OF_RANGE', `index ${xi.v} out of range (len ${c.length})`);
    return vStr(c[i]);
  }
  if (target.t === 'bytes') {
    const xi = widen(index);
    if (xi.t === 'range') {
      const r = normalizeSlice(target.data.length, xi.start, xi.end);
      if (!r) return vBytes(new Uint8Array(0));
      return vBytes(target.data.slice(r[0], r[1]));
    }
    if (xi.t !== 'int') return typeMismatch('Int index', typeName(index));
    const i = resolveIndex(target.data.length, xi.v);
    if (i < 0 || i >= target.data.length) fault('AIPO_RT_INDEX_OUT_OF_RANGE', `index ${xi.v} out of range (len ${target.data.length})`);
    return { t: 'byte', v: target.data[i] };
  }
  if (target.t === 'range') {
    const len = rangeLen(target);
    const xi = widen(index);
    if (xi.t !== 'int') return typeMismatch('Int index', typeName(index));
    const i = resolveIndex(len, xi.v);
    if (i < 0 || i >= len) fault('AIPO_RT_INDEX_OUT_OF_RANGE', `index ${xi.v} out of range (len ${len})`);
    return vInt(target.start + i);
  }
  if (target.t === 'dict') {
    const found = dictGet(target, index);
    if (found === null) fault('AIPO_RT_KEY_NOT_FOUND', `key ${display(index)} not found`);
    return found;
  }
  return typeMismatch('indexable collection or string', typeName(target));
}
export function valSetIndex(target, index, value) {
  if (isFailure(target)) return target;
  if (isFailure(index)) return index;
  if (isFailure(value)) return value;
  if (target.t === 'list') {
    const xi = widen(index);
    if (xi.t !== 'int') return typeMismatch('Int index', typeName(index));
    const i = resolveIndex(target.items.length, xi.v);
    if (i < 0 || i >= target.items.length) fault('AIPO_RT_INDEX_OUT_OF_RANGE', `index ${xi.v} out of range (len ${target.items.length})`);
    target.items[i] = value;
    return vNone();
  }
  if (target.t === 'dict') {
    dictUpsert(target, index, value);
    return vNone();
  }
  return typeMismatch('mutable collection', typeName(target));
}
export function valLen(v) {
  if (isFailure(v)) return v;
  if (v.t === 'str') return vInt(chars(v.v).length);
  if (v.t === 'list') return vInt(v.items.length);
  if (v.t === 'dict') return vInt(v.entries.length);
  if (v.t === 'bytes') return vInt(v.data.length);
  if (v.t === 'range') return vInt(rangeLen(v));
  return typeMismatch('String, List, Dict, Bytes, or Range', typeName(v));
}

// ---- List / Dict natives (receiver-first) ----
const MUTATING = new Set(['add', 'insert', 'remove', 'remove_at', 'remove_last', 'clear']);
export function checkMutationAllowed(name, recv, active) {
  if (!MUTATING.has(name)) return;
  if (recv && (recv.t === 'list' || recv.t === 'dict') && active.includes(recv.id)) {
    fault('AIPO_RT_MUTATION_DURING_ITERATION', 'mutation during iteration');
  }
}
function reqList(v) { if (v.t !== 'list') return typeMismatch('List', typeName(v)); }
function reqDict(v) { if (v.t !== 'dict') return typeMismatch('Dict', typeName(v)); }
function intIndex(v) {
  const x = widen(v);
  if (x.t !== 'int') return typeMismatch('Int index', typeName(v));
  return x.v;
}
export const listNatives = {
  add(r, a) { reqList(r); r.items.push(a[0]); return vNone(); },
  insert(r, a) { reqList(r); const i0 = intIndex(a[0]); const idx = i0 < 0 ? r.items.length + i0 : i0; if (idx < 0 || idx > r.items.length) fault('AIPO_RT_INDEX_OUT_OF_RANGE', `index ${i0} out of range (len ${r.items.length})`); r.items.splice(idx, 0, a[1]); return vNone(); },
  remove(r, a) { reqList(r); const at = r.items.findIndex(x => valuesEqual(x, a[0])); if (at >= 0) r.items.splice(at, 1); return vBool(at >= 0); },
  remove_at(r, a) { reqList(r); if (r.items.length === 0) fault('AIPO_RT_INDEX_OUT_OF_RANGE', 'index 0 out of range (len 0)'); const i0 = intIndex(a[0]); const idx = i0 < 0 ? r.items.length + i0 : i0; if (idx < 0 || idx >= r.items.length) fault('AIPO_RT_INDEX_OUT_OF_RANGE', `index ${i0} out of range (len ${r.items.length})`); return r.items.splice(idx, 1)[0]; },
  remove_last(r) { reqList(r); if (r.items.length === 0) fault('AIPO_RT_INDEX_OUT_OF_RANGE', 'index -1 out of range (len 0)'); return r.items.pop(); },
  clear(r) { reqList(r); r.items.length = 0; return vNone(); },
  contains(r, a) { reqList(r); return vBool(r.items.some(x => valuesEqual(x, a[0]))); },
  find(r, a) { reqList(r); const at = r.items.findIndex(x => valuesEqual(x, a[0])); return at < 0 ? vNone() : vInt(at); },
  count(r, a) { reqList(r); return vInt(r.items.filter(x => valuesEqual(x, a[0])).length); },
  first(r) { reqList(r); if (r.items.length === 0) fault('AIPO_RT_INDEX_OUT_OF_RANGE', 'index 0 out of range (len 0)'); return r.items[0]; },
  last(r) { reqList(r); if (r.items.length === 0) fault('AIPO_RT_INDEX_OUT_OF_RANGE', 'index -1 out of range (len 0)'); return r.items[r.items.length - 1]; },
  is_empty(r) { reqList(r); return vBool(r.items.length === 0); },
  len(r) { reqList(r); return vInt(r.items.length); },
  reverse(r) { reqList(r); return vList([...r.items].reverse()); },
  sort(r) { reqList(r); const cp = [...r.items]; cp.sort(compareValues); return vList(cp); },
};
export const dictNatives = {
  has(r, a) { reqDict(r); return vBool(dictGet(r, a[0]) !== null); },
  get(r, a) { reqDict(r); const f = dictGet(r, a[0]); return f === null ? vNone() : f; },
  keys(r) { reqDict(r); return vList(r.entries.map(([k]) => k)); },
  values(r) { reqDict(r); return vList(r.entries.map(([, v]) => v)); },
  remove(r, a) { reqDict(r); return vBool(dictRemove(r, a[0])); },
  clear(r) { reqDict(r); dictClear(r); return vNone(); },
  is_empty(r) { reqDict(r); return vBool(r.entries.length === 0); },
  len(r) { reqDict(r); return vInt(r.entries.length); },
};
function numKey(v) {
  if (v.t === 'int' || v.t === 'byte') return v.v;
  if (v.t === 'float') return v.v;
  return null;
}
function compareValues(a, b) {
  const an = numKey(a), bn = numKey(b);
  if (an !== null && bn !== null) return an - bn;
  if (a.t === 'str' && b.t === 'str') return a.v < b.v ? -1 : a.v > b.v ? 1 : 0;
  if (a.t === 'bool' && b.t === 'bool') return a.v === b.v ? 0 : !a.v ? -1 : 1;
  throw new AipoFault('AIPO_RT_TYPE_MISMATCH', 'type mismatch: expected comparable sort values');
}

// ---- structs ----
export function structGetField(obj, field) {
  if (isFailure(obj)) {
    if (field === 'message') return vStr(obj.msg);
    return obj;
  }
  if (obj.t !== 'struct') return typeMismatch('struct instance', typeName(obj));
  const f = obj.fields.find(([k]) => k === field);
  if (!f) fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: ${obj.type} has no field '${field}'`);
  return f[1];
}
export function structSetField(obj, field, value, journal) {
  if (isFailure(obj)) return obj;
  if (isFailure(value)) return value;
  if (obj.t !== 'struct') return typeMismatch('struct instance', typeName(obj));
  const at = obj.fields.findIndex(([k]) => k === field);
  if (at < 0) fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: ${obj.type} has no field '${field}'`);
  if (!obj.constructing && obj.fixed.has(field)) fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: fixed field '${obj.type}.${field}' cannot be reassigned`);
  if (journal && !obj.constructing && hasInvariant(obj.type)) {
    const seen = journal.some(e => e.inst === obj && e.field === field);
    if (!seen) journal.push({ inst: obj, field, prev: obj.fields[at][1] });
  }
  obj.fields[at][1] = value;
  return vNone();
}
const structDefs = new Map();
export function registerStructDef(name, fields) { structDefs.set(name, fields); }
export function getStructDef(name) { return structDefs.get(name); }
function hasInvariant(type) {
  return typeof __invariantEntries !== 'undefined' && __invariantEntries && __invariantEntries.has(type);
}
// __invariantEntries is installed per run (see interpreter part 4).
export let __invariantEntries = null;
export function setInvariantEntries(m) { __invariantEntries = m; }

// ---- machine ----
function constToValue(c) {
  if (typeof c === 'string') {
    if (c === 'None') return vNone();
    fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: bad constant ${c}`);
  }
  if (c && typeof c === 'object') {
    if ('None' in c) return vNone();
    if ('Bool' in c) return vBool(c.Bool);
    if ('Int' in c) return checkSafeInt(c.Int);
    if ('Float' in c) return checkFiniteFloat(c.Float);
    if ('String' in c) return vStr(c.String);
  }
  fault('AIPO_RT_TYPE_MISMATCH', 'type mismatch: bad constant');
}

function makeMachine(module) {
  const funcIndex = new Map();
  module.functions.forEach((f, i) => funcIndex.set(f.name, i));
  const invEntries = new Map();
  module.functions.forEach((f, i) => {
    const dot = f.name.lastIndexOf('.');
    if (dot > 0 && f.name.slice(dot + 1) === 'invariant') invEntries.set(f.name.slice(0, dot), i);
  });
  setInvariantEntries(invEntries);
  (module.structs || []).forEach(s => registerStructDef(s.name, s.fields.map(([n, fx]) => [n, !!fx])));
  const structMethods = new Map();
  module.functions.forEach((f, i) => {
    const dot = f.name.indexOf('.');
    if (dot > 0) {
      const tn = f.name.slice(0, dot), m = f.name.slice(dot + 1);
      if (m !== 'invariant') structMethods.set(`${tn}.${m}`, { idx: i, total: f.params.length });
    }
  });
  return {
    module, funcIndex, structMethods,
    stack: [], frames: [], handlers: [], journal: [], active: [],
    globals: makeGlobals(), halted: null, done: false, result: null,
  };
}

function makeGlobals() {
  const g = new Map();
  g.set('none', vNone());
  g.set('true', vBool(true));
  g.set('false', vBool(false));
  const nat = (name, arity, fn) => ({ t: 'native', name, arity, fn });
  g.set('len', nat('len', 1, a => std_len(a[0])));
  g.set('copy', nat('copy', 1, a => std_copy(a[0])));
  g.set('same', nat('same', 2, a => std_same(a[0], a[1])));
  g.set('some', nat('some', 1, a => std_some(a[0])));
  g.set('fail', nat('fail', 1, a => std_fail(a[0])));
  g.set('Int', vType('Int'));
  g.set('Float', vType('Float'));
  g.set('Byte', vType('Byte'));
  g.set('String', vType('String'));
  g.set('Bool', vType('Bool'));
  g.set('List', vType('List'));
  g.set('Dict', vType('Dict'));
  g.set('Bytes', vType('Bytes'));
  // math module
  const mathEntries = [
    ['abs', nat('math.abs', 1, a => std_math_abs(a[0]))],
    ['min', nat('math.min', 2, a => std_math_min(a[0], a[1]))],
    ['max', nat('math.max', 2, a => std_math_max(a[0], a[1]))],
    ['floor', nat('math.floor', 1, a => std_math_floor(a[0]))],
    ['ceil', nat('math.ceil', 1, a => std_math_ceil(a[0]))],
    ['round', nat('math.round', 1, a => std_math_round(a[0]))],
    ['truncate', nat('math.truncate', 1, a => std_math_truncate(a[0]))],
    ['sqrt', nat('math.sqrt', 1, a => std_math_sqrt(a[0]))],
    ['pow', nat('math.pow', 2, a => std_math_pow(a[0], a[1]))],
    ['clamp', nat('math.clamp', 3, a => std_math_clamp(a[0], a[1], a[2]))],
    ['pi', checkFiniteFloat(Math.PI)],
    ['e', checkFiniteFloat(Math.E)],
  ];
  g.set('math', vDict(mathEntries.map(([k, v]) => [vStr(k), v])));
  // string module
  const sm = (n, arity, fn) => [vStr(n), nat(`string.${n}`, arity, fn)];
  g.set('string', vDict([
    sm('len', 1, a => std_string_len(a[0])),
    sm('byte_len', 1, a => std_string_byte_len(a[0])),
    sm('contains', 2, a => std_string_contains(a[0], a[1])),
    sm('starts_with', 2, a => std_string_starts_with(a[0], a[1])),
    sm('ends_with', 2, a => std_string_ends_with(a[0], a[1])),
    sm('find', 2, a => std_string_find(a[0], a[1])),
    sm('lower', 1, a => std_string_lower(a[0])),
    sm('upper', 1, a => std_string_upper(a[0])),
    sm('capitalize', 1, a => std_string_capitalize(a[0])),
    sm('reverse', 1, a => std_string_reverse(a[0])),
    sm('trim', 1, a => std_string_trim(a[0])),
    sm('split', 2, a => std_string_split(a[0], a[1])),
    sm('join', 2, a => std_string_join(a[0], a[1])),
    sm('replace', 3, a => std_string_replace(a[0], a[1], a[2])),
    sm('slice', 3, a => std_string_slice(a[0], a[1], a[2])),
    sm('format', 2, a => std_string_format(a[0], a[1])),
  ]));
  // io module
  const ioText = v => (v.t === 'str' ? v.v : display(v));
  g.set('io', vDict([
    [vStr('print'), nat('io.print', 1, a => { emitText(ioText(a[0])); return vNone(); })],
    [vStr('println'), nat('io.println', 1, a => { emitText(ioText(a[0])); emitText('\n'); return vNone(); })],
  ]));
  return g;
}

function mPush(m, v) {
  if (m.stack.length >= 1024) fault('AIPO_RT_OVERFLOW', `operand stack exceeded depth limit 1024 (${m.frames.length} active call frames; deep or unbounded recursion is the usual cause)`);
  m.stack.push(v);
}
function mPop(m) {
  if (m.stack.length === 0) fault('AIPO_RT_TYPE_MISMATCH', 'type mismatch: stack underflow');
  return m.stack.pop();
}
function mPeek(m) {
  if (m.stack.length === 0) fault('AIPO_RT_TYPE_MISMATCH', 'type mismatch: stack underflow');
  return m.stack[m.stack.length - 1];
}
function curFrame(m) { return m.frames.length ? m.frames[m.frames.length - 1] : null; }

function bindMethod(m, recv, name) {
  const tn = recv.t === 'struct' ? 'struct' : typeName(recv);
  const key = `${tn}.${name}`;
  const table = {
    'String.len': [0, a => std_string_len(a[0])],
    'String.byte_len': [0, a => std_string_byte_len(a[0])],
    'String.contains': [1, a => std_string_contains(a[0], a[1])],
    'String.starts_with': [1, a => std_string_starts_with(a[0], a[1])],
    'String.ends_with': [1, a => std_string_ends_with(a[0], a[1])],
    'String.find': [1, a => std_string_find(a[0], a[1])],
    'String.lower': [0, a => std_string_lower(a[0])],
    'String.upper': [0, a => std_string_upper(a[0])],
    'String.capitalize': [0, a => std_string_capitalize(a[0])],
    'String.reverse': [0, a => std_string_reverse(a[0])],
    'String.trim': [0, a => std_string_trim(a[0])],
    'String.split': [1, a => std_string_split(a[0], a[1])],
    'String.join': [1, a => std_string_join(a[0], a[1])],
    'String.replace': [2, a => std_string_replace(a[0], a[1], a[2])],
    'String.slice': [2, a => std_string_slice(a[0], a[1], a[2])],
    'String.format': [1, a => std_string_format(a[0], a[1])],
    'List.add': [1, a => listNatives.add(a[0], a.slice(1))],
    'List.insert': [2, a => listNatives.insert(a[0], a.slice(1))],
    'List.remove': [1, a => listNatives.remove(a[0], a.slice(1))],
    'List.remove_at': [1, a => listNatives.remove_at(a[0], a.slice(1))],
    'List.remove_last': [0, a => listNatives.remove_last(a[0], a.slice(1))],
    'List.clear': [0, a => listNatives.clear(a[0], a.slice(1))],
    'List.contains': [1, a => listNatives.contains(a[0], a.slice(1))],
    'List.find': [1, a => listNatives.find(a[0], a.slice(1))],
    'List.count': [1, a => listNatives.count(a[0], a.slice(1))],
    'List.first': [0, a => listNatives.first(a[0], a.slice(1))],
    'List.last': [0, a => listNatives.last(a[0], a.slice(1))],
    'List.is_empty': [0, a => listNatives.is_empty(a[0], a.slice(1))],
    'List.reverse': [0, a => listNatives.reverse(a[0], a.slice(1))],
    'List.sort': [0, a => listNatives.sort(a[0], a.slice(1))],
    'List.len': [0, a => listNatives.len(a[0], a.slice(1))],
    'Dict.has': [1, a => dictNatives.has(a[0], a.slice(1))],
    'Dict.get': [1, a => dictNatives.get(a[0], a.slice(1))],
    'Dict.keys': [0, a => dictNatives.keys(a[0], a.slice(1))],
    'Dict.values': [0, a => dictNatives.values(a[0], a.slice(1))],
    'Dict.remove': [1, a => dictNatives.remove(a[0], a.slice(1))],
    'Dict.clear': [0, a => dictNatives.clear(a[0], a.slice(1))],
    'Dict.is_empty': [0, a => dictNatives.is_empty(a[0], a.slice(1))],
    'Dict.len': [0, a => dictNatives.len(a[0], a.slice(1))],
  };
  if (table[key]) {
    const [arity, fn] = table[key];
    return { t: 'bound', name, arity, recv, kind: 'native', fn };
  }
  if ((name === 'filter' || name === 'transform' || name === 'sort_by') &&
      (recv.t === 'list' || recv.t === 'dict' || recv.t === 'str' || recv.t === 'range')) {
    return { t: 'bound', name, arity: 1, recv, kind: 'higher' };
  }
  return null;
}

function doGetField(m, target, field) {
  if (isFailure(target)) {
    if (field === 'message') return vStr(target.msg);
    return target;
  }
  if (target.t === 'struct') {
    const f = target.fields.find(([k]) => k === field);
    if (f) return f[1];
    const sm = m.structMethods.get(`${target.type}.${field}`);
    if (sm) {
      return { t: 'bound', name: `${target.type}.${field}`, arity: Math.max(0, m.module.functions[sm.idx].params.length - 1), recv: target, kind: 'ufunc', idx: sm.idx, total: sm.total };
    }
    fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: ${target.type} has no field '${field}'`);
  }
  if (target.t === 'dict') {
    const found = dictGet(target, { t: 'str', v: field });
    if (found !== null) return found;
    const b = bindMethod(m, target, field);
    if (b) return b;
    fault('AIPO_RT_KEY_NOT_FOUND', `key ${field} not found`);
  }
  const b = bindMethod(m, target, field);
  if (b) return b;
  return typeMismatch('struct, collection, or type with that member', `${typeName(target)}.${field}`);
}

function convertViaType(tag, args) {
  if (args.length !== 1) return typeMismatch('1 argument', `${args.length} arguments`);
  const a = args[0];
  switch (tag) {
    case 'Int': return convInt(a);
    case 'Float': return convFloat(a);
    case 'Byte': return convByte(a);
    case 'String': return convString(a);
    case 'Bytes': return convBytes(a);
    default: fault('AIPO_RT_NOT_CALLABLE', `${tag} (no conversion form in V1)`);
  }
}

function checkArity(got, want, what) {
  if (got !== want) fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: expected ${want} arguments${what ? ` for ${what}` : ''}, got ${got}`);
}

function beginCall(m, argc) {
  if (m.stack.length < argc + 1) fault('AIPO_RT_TYPE_MISMATCH', 'type mismatch: stack underflow');
  const calleeIdx = m.stack.length - 1 - argc;
  const callee = m.stack[calleeIdx];
  if (isFailure(callee)) {
    const f = m.stack.splice(calleeIdx);
    mPush(m, f[0]);
    return;
  }
  for (let i = 0; i < argc; i++) {
    if (isFailure(m.stack[calleeIdx + 1 + i])) {
      const f = m.stack[calleeIdx + 1 + i];
      m.stack.length = calleeIdx;
      mPush(m, f);
      return;
    }
  }
  const args = m.stack.slice(calleeIdx + 1, calleeIdx + 1 + argc);
  const callFn = (idx, vars, cells) => {
    const fn = m.module.functions[idx];
    checkArity(argc, fn.params.length);
    const v = {};
    fn.params.forEach((p, i) => { v[p] = args[i]; });
    m.frames.push({ fn, ip: 0, vars: v, cells, base: calleeIdx, journalStart: m.journal.length });
  };
  if (callee.t === 'func') {
    callFn(callee.idx, {}, null);
    m.stack.length = calleeIdx + 1 + argc;
  } else if (callee.t === 'closure') {
    callFn(callee.idx, {}, callee.cells);
    m.stack.length = calleeIdx + 1 + argc;
  } else if (callee.t === 'native') {
    checkArity(argc, callee.arity);
    const r = callee.fn(args);
    m.stack.length = calleeIdx;
    mPush(m, r);
  } else if (callee.t === 'type') {
    const r = convertViaType(callee.name, args);
    m.stack.length = calleeIdx;
    mPush(m, r);
  } else if (callee.t === 'bound') {
    checkArity(argc, callee.arity);
    if (callee.kind === 'native') {
      checkMutationAllowed(callee.name, callee.recv, m.active);
      const r = callee.fn([callee.recv, ...args]);
      m.stack.length = calleeIdx;
      mPush(m, r);
    } else if (callee.kind === 'ufunc') {
      if (argc + 1 !== callee.total) fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: expected ${callee.total - 1} arguments for ${callee.name}, got ${argc}`);
      m.stack[calleeIdx] = callee.recv;
      const fn = m.module.functions[callee.idx];
      const v = {};
      v[fn.params[0]] = callee.recv;
      args.forEach((a, i) => { v[fn.params[i + 1]] = a; });
      m.frames.push({ fn, ip: 0, vars: v, cells: null, base: calleeIdx, journalStart: m.journal.length });
      m.stack.length = calleeIdx + 1 + argc;
    } else if (callee.kind === 'higher') {
      const callable = args[0] !== undefined ? args[0] : vNone();
      m.stack.length = calleeIdx;
      const r = higherOrder(m, callee.name, callee.recv, callable);
      mPush(m, r);
    }
  } else {
    fault('AIPO_RT_NOT_CALLABLE', `${typeName(callee)} is not callable`);
  }
}

function iterableItems(m, recv) {
  if (recv.t === 'list') return [...recv.items];
  if (recv.t === 'dict') return recv.entries.map(([, v]) => v);
  if (recv.t === 'str') return chars(recv.v).map(c => vStr(c));
  if (recv.t === 'range') {
    const out = [];
    for (let i = recv.start; i < recv.end; i++) out.push(vInt(i));
    return out;
  }
  return typeMismatch('iterable List, Dict, String, or Range', typeName(recv));
}

function invokeSame(m, callee, args) {
  const stackBase = m.stack.length;
  const frameBase = m.frames.length;
  const handlerBase = m.handlers.length;
  const journalBase = m.journal.length;
  mPush(m, callee);
  args.forEach(a => mPush(m, a));
  beginCall(m, args.length);
  const called = m.frames.length > frameBase;
  if (!called) {
    const r = mPop(m);
    m.stack.length = stackBase;
    m.handlers.length = handlerBase;
    m.journal.length = journalBase;
    if (isFailure(r)) throw { uncaught: r.msg };
    return r;
  }
  while (m.frames.length > frameBase) {
    stepFn(m);
    if (m.done) break;
  }
  if (m.done) {
    const r = m.result;
    m.done = false;
    m.stack.length = stackBase;
    m.frames.length = frameBase;
    m.handlers.length = handlerBase;
    m.journal.length = journalBase;
    if (isFailure(r)) throw { uncaught: r.msg };
    return r;
  }
  const r = mPop(m);
  m.stack.length = stackBase;
  m.handlers.length = handlerBase;
  m.journal.length = journalBase;
  if (isFailure(r)) throw { uncaught: r.msg };
  return r;
}

function higherOrder(m, name, recv, callable) {
  const items = iterableItems(m, recv);
  const guardId = (recv.t === 'list' || recv.t === 'dict') ? recv.id : null;
  if (guardId !== null) m.active.push(guardId);
  try {
    if (name === 'filter') {
      const kept = [];
      for (const it of items) {
        let r;
        try { r = invokeSame(m, callable, [it]); }
        catch (e) { if (e instanceof AipoFault) throw e; throw new AipoFault('AIPO_RT_TYPE_MISMATCH', `type mismatch: ${e && e.uncaught ? e.uncaught : 'failure in predicate'}`); }
        if (r.t !== 'bool') fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: expected Bool predicate result, got ${typeName(r)}`);
        if (r.v) kept.push(it);
      }
      return vList(kept);
    }
    if (name === 'transform') {
      const out = [];
      for (const it of items) {
        try { out.push(invokeSame(m, callable, [it])); }
        catch (e) { if (e instanceof AipoFault) throw e; out.push(vFail(e.uncaught)); }
      }
      return vList(out);
    }
    if (name === 'sort_by') {
      const keyed = [];
      for (const it of items) {
        let k;
        try { k = invokeSame(m, callable, [it]); }
        catch (e) { if (e instanceof AipoFault) throw e; throw new AipoFault('AIPO_RT_TYPE_MISMATCH', 'type mismatch: failure in sort key'); }
        keyed.push([k, it]);
      }
      keyed.sort((a, b) => sortKeyCompare(a[0], b[0]));
      return vList(keyed.map(([, it]) => it));
    }
    return typeMismatch('filter, transform, or sort_by', name);
  } finally {
    if (guardId !== null) {
      const at = m.active.lastIndexOf(guardId);
      if (at >= 0) m.active.splice(at, 1);
    }
  }
}

function sortKeyCompare(a, b) {
  const an = numKey(a), bn = numKey(b);
  if (an !== null && bn !== null) return an - bn;
  if (an !== null || bn !== null) throw new AipoFault('AIPO_RT_TYPE_MISMATCH', 'type mismatch: expected comparable sort keys');
  if (a.t === 'str' && b.t === 'str') return a.v < b.v ? -1 : a.v > b.v ? 1 : 0;
  if (a.t === 'bool' && b.t === 'bool') return a.v === b.v ? 0 : !a.v ? -1 : 1;
  throw new AipoFault('AIPO_RT_TYPE_MISMATCH', 'type mismatch: expected comparable sort keys');
}

function tagMatches(tag, v) {
  switch (tag) {
    case 'none': return v.t === 'none';
    case 'Bool': return v.t === 'bool';
    case 'Int': return v.t === 'int';
    case 'Float': return v.t === 'float';
    case 'Byte': return v.t === 'byte';
    case 'String': return v.t === 'str';
    case 'List': return v.t === 'list';
    case 'Dict': return v.t === 'dict';
    case 'Bytes': return v.t === 'bytes';
    case 'Range': return v.t === 'range';
    default: return false;
  }
}

function operationArity(m, value, name) {
  if (value.t === 'struct') {
    const sm = m.structMethods.get(`${value.type}.${name}`);
    if (sm) return Math.max(0, m.module.functions[sm.idx].params.length - 1);
  }
  const b = bindMethod(m, value, name);
  return b ? b.arity : null;
}

function assertContract(m, type, nullable, pos, ops, value) {
  if (isUnset(value) || isFailure(value)) return;
  if (value.t === 'none') {
    if (!nullable) fault('AIPO_RT_TYPE_MISMATCH', `contract violation at ${pos}: expected ${type}, got none`);
    return;
  }
  if (type === 'Function') {
    if (!(value.t === 'func' || value.t === 'closure' || value.t === 'native' || value.t === 'bound')) {
      fault('AIPO_RT_TYPE_MISMATCH', `contract violation at ${pos}: expected Function, got ${typeName(value)}`);
    }
  } else if (['none', 'Bool', 'Int', 'Float', 'Byte', 'String', 'List', 'Dict', 'Bytes', 'Range'].includes(type)) {
    if (!tagMatches(type, value)) {
      fault('AIPO_RT_TYPE_MISMATCH', `contract violation at ${pos}: expected ${nullable ? `${type}?` : type}, got ${structDisplayName(value)}`);
    }
  } else if (getStructDef(type)) {
    if (!(value.t === 'struct' && value.type === type)) {
      fault('AIPO_RT_TYPE_MISMATCH', `contract violation at ${pos}: expected ${nullable ? `${type}?` : type}, got ${structDisplayName(value)}`);
    }
  }
  if (ops && ops.length && value.t !== 'none') {
    for (const [opName, opArity] of ops) {
      const found = operationArity(m, value, opName);
      if (found === null) {
        fault('AIPO_RT_TYPE_MISMATCH', `contract violation at ${pos}: expected ${type}.${opName}/${opArity}, got ${structDisplayName(value)} without '${opName}'`);
      } else if (found !== opArity) {
        fault('AIPO_RT_TYPE_MISMATCH', `contract violation at ${pos}: expected ${type}.${opName}/${opArity}, got ${structDisplayName(value)}.${opName}/${found}`);
      }
    }
  }
}

function handleFailure(m, failVal) {
  if (m.handlers.length) {
    const h = m.handlers.pop();
    m.frames.length = h.frameDepth;
    m.stack.length = h.stackDepth;
    // Match the Rust VM exactly: recovery releases every provisional mutation journaled
    // by frames that did not survive, including entries the surviving frame journaled
    // before the handler was registered. Keeping them would roll back mutations that
    // canon already committed past the handler boundary.
    const fr = curFrame(m);
    m.journal.length = fr ? fr.journalStart : 0;
    mPush(m, failVal);
    if (fr) fr.ip = h.ip;
    return;
  }
  if (m.frames.length) {
    const fr = m.frames.pop();
    m.stack.length = fr.base;
    m.journal.length = fr.journalStart;
    mPush(m, failVal);
    return;
  }
  m.halted = failVal;
  m.done = true;
}

function checkMutations(m) {
  const fr = curFrame(m);
  const start = fr ? fr.journalStart : 0;
  if (m.journal.length <= start) return;
  const involved = [];
  const seen = new Set();
  for (let i = start; i < m.journal.length; i++) {
    const e = m.journal[i];
    if (!seen.has(e.inst.id)) { seen.add(e.inst.id); involved.push(e.inst); }
  }
  let failedType = null;
  for (const inst of involved) {
    const idx = __invariantEntries ? __invariantEntries.get(inst.type) : undefined;
    if (idx === undefined) continue;
    let r;
    try {
      r = invokeSame(m, { t: 'func', idx, arity: 1 }, [inst]);
    } catch (e) {
      if (e instanceof AipoFault) throw e;
      failedType = inst.type;
      break;
    }
    if (!(r.t === 'bool' && r.v === true)) { failedType = inst.type; break; }
  }
  if (failedType) {
    for (let i = m.journal.length - 1; i >= start; i--) {
      const e = m.journal[i];
      const at = e.inst.fields.findIndex(([k]) => k === e.field);
      if (at >= 0) e.inst.fields[at][1] = e.prev;
    }
    m.journal.length = start;
    handleFailure(m, vFail(`invariant() of ${failedType} did not hold after mutation`));
  } else {
    m.journal.length = start;
  }
}

// ---- instruction dispatch ----
function binOp(m, op, a, b) {
  switch (op) {
    case 'Add': return valAdd(a, b);
    case 'Sub': return valSub(a, b);
    case 'Mul': return valMul(a, b);
    case 'Div': return valDiv(a, b);
    case 'IntDiv': return valIntDiv(a, b);
    case 'Mod': return valMod(a, b);
    case 'Equal': return valEqual(a, b);
    case 'NotEqual': return valNotEqual(a, b);
    case 'Less': return valLess(a, b);
    case 'LessEqual': return valLessEqual(a, b);
    case 'Greater': return valGreater(a, b);
    case 'GreaterEqual': return valGreaterEqual(a, b);
    case 'Is': {
      if (isFailure(a)) return a;
      if (isFailure(b)) return b;
      if (b.t !== 'type') return typeMismatch('type value on the right of is', typeName(b));
      return vBool(tagMatches(b.name, a));
    }
    case 'And': {
      if (isFailure(a)) return a;
      if (isFailure(b)) return b;
      if (a.t !== 'bool' || b.t !== 'bool') return typeMismatch('Bool', `${typeName(a)} and ${typeName(b)}`);
      return vBool(a.v && b.v);
    }
    case 'Or': {
      if (isFailure(a)) return a;
      if (isFailure(b)) return b;
      if (a.t !== 'bool' || b.t !== 'bool') return typeMismatch('Bool', `${typeName(a)} and ${typeName(b)}`);
      return vBool(a.v || b.v);
    }
    case 'OrElse': {
      if (isFailure(a)) return b;
      return a;
    }
    case 'Pipeline': {
      mPush(m, b);
      mPush(m, a);
      beginCall(m, 1);
      return null;
    }
    case 'Range': {
      if (isFailure(a)) return a;
      if (isFailure(b)) return b;
      const x = widen(a), y = widen(b);
      if (x.t !== 'int' || y.t !== 'int') return typeMismatch('Int range bounds', `${typeName(a)} and ${typeName(b)}`);
      return vRange(x.v, y.v);
    }
    default: fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: unknown operator ${op}`);
  }
}

function stepFn(m) {
  const fr = curFrame(m);
  if (!fr) {
    const r = m.stack.length ? m.stack[m.stack.length - 1] : vNone();
    m.result = r;
    m.done = true;
    return;
  }
  const fn = fr.fn;
  if (fr.ip >= fn.code.length) {
    doReturn(m, vNone());
    return;
  }
  const inst = fn.code[fr.ip++];
  const op = inst.op;
  const localNames = [...fn.params, ...fn.locals];
  switch (op) {
    case 'Constant': mPush(m, constToValue(inst.value)); break;
    case 'Load': {
      const n = inst.name;
      if (fn.params.includes(n) || fn.locals.includes(n)) {
        mPush(m, Object.hasOwn(fr.vars, n) ? fr.vars[n] : vUnset());
      } else if (fr.cells && Object.hasOwn(fr.cells, n)) mPush(m, fr.cells[n].v);
      else if (m.globals.has(n)) mPush(m, m.globals.get(n));
      else fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: undefined global '${n}'`);
      break;
    }
    case 'Store': {
      const n = inst.name;
      const v = mPop(m);
      if (fn.params.includes(n) || fn.locals.includes(n)) fr.vars[n] = v;
      else if (fr.cells && Object.hasOwn(fr.cells, n)) fr.cells[n].v = v;
      else m.globals.set(n, v);
      break;
    }
    case 'GetUpvalue': {
      const n = inst.name;
      if (!fr.cells || !Object.hasOwn(fr.cells, n)) fault('AIPO_RT_TYPE_MISMATCH', 'type mismatch: stack underflow');
      mPush(m, fr.cells[n].v);
      break;
    }
    case 'SetUpvalue': {
      const n = inst.name;
      if (!fr.cells || !Object.hasOwn(fr.cells, n)) fault('AIPO_RT_TYPE_MISMATCH', 'type mismatch: stack underflow');
      fr.cells[n].v = mPop(m);
      break;
    }
    case 'Binary': {
      const b = mPop(m);
      const a = mPop(m);
      const r = binOp(m, inst.op2, a, b);
      if (r !== null && r !== undefined) mPush(m, r);
      break;
    }
    case 'Unary': {
      const a = mPop(m);
      if (inst.op2 === 'Neg') mPush(m, valNeg(a));
      else if (inst.op2 === 'Pos') mPush(m, valPos(a));
      else if (inst.op2 === 'Not') mPush(m, valNot(a));
      else fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: unknown unary ${inst.op2}`);
      break;
    }
    case 'Call': beginCall(m, inst.argc); break;
    case 'Return': {
      const v = inst.has ? mPop(m) : vNone();
      doReturn(m, v);
      break;
    }
    case 'Jump': fr.ip = inst.t; break;
    case 'JumpIfFalse': {
      const c = mPop(m);
      if (c.t !== 'bool') fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: expected Bool, got ${typeName(c)}`);
      if (!c.v) fr.ip = inst.t;
      break;
    }
    case 'JumpIfSetLocal': {
      const nm = localNames[inst.slot];
      const v = nm !== undefined && Object.hasOwn(fr.vars, nm) ? fr.vars[nm] : vUnset();
      if (v.t !== 'unset') fr.ip = inst.t;
      break;
    }
    case 'PushHandler': {
      m.handlers.push({ ip: inst.t, fn, stackDepth: m.stack.length, frameDepth: m.frames.length });
      break;
    }
    case 'PopHandler': m.handlers.pop(); break;
    case 'Pop': mPop(m); break;
    case 'Dup': mPush(m, mPeek(m)); break;
    case 'GetField': {
      const t = mPop(m);
      mPush(m, doGetField(m, t, inst.f));
      break;
    }
    case 'SetField': {
      const nv = mPop(m);
      const t = mPop(m);
      if (t.t !== 'struct') return typeMismatch('struct instance', typeName(t));
      if (isFailure(nv)) { mPush(m, nv); break; }
      const at = t.fields.findIndex(([k]) => k === inst.f);
      if (at < 0) fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: ${t.type} has no field '${inst.f}'`);
      if (!t.constructing && t.fixed.has(inst.f)) fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: fixed field '${t.type}.${inst.f}' cannot be reassigned`);
      if (!t.constructing && __invariantEntries && __invariantEntries.has(t.type)) {
        if (!m.journal.some(e => e.inst === t && e.field === inst.f)) {
          m.journal.push({ inst: t, field: inst.f, prev: t.fields[at][1] });
        }
      }
      t.fields[at][1] = nv;
      mPush(m, vNone());
      break;
    }
    case 'GetIndex': {
      const ix = mPop(m);
      const t = mPop(m);
      mPush(m, valGetIndex(t, ix, m.active));
      break;
    }
    case 'SetIndex': {
      const v = mPop(m);
      const ix = mPop(m);
      const t = mPop(m);
      valSetIndex(t, ix, v);
      mPush(m, vNone());
      break;
    }
    case 'BuildList': {
      const n = inst.n;
      const items = [];
      for (let i = 0; i < n; i++) items.unshift(mPop(m));
      mPush(m, vList(items));
      break;
    }
    case 'BuildDict': {
      const n = inst.n;
      const entries = [];
      for (let i = 0; i < n; i++) {
        const v = mPop(m);
        const k = mPop(m);
        entries.unshift([k, v]);
      }
      mPush(m, vDict(entries));
      break;
    }
    case 'BuildStruct': {
      const n = inst.n;
      const vals = [];
      for (let i = 0; i < n; i++) vals.unshift(mPop(m));
      const def = getStructDef(inst.type);
      const fields = def
        ? def.slice(0, n).map(([fname], i) => [fname, vals[i] !== undefined ? vals[i] : vNone()])
        : vals.map((v, i) => [`field_${i}`, v]);
      const fixed = inst.defer ? [] : (def ? def.filter(([, fx]) => fx).map(([fname]) => fname) : []);
      mPush(m, vStruct(inst.type, fields, fixed, !!inst.defer));
      break;
    }
    case 'SealStruct': {
      const v = mPop(m);
      if (v.t !== 'struct') return typeMismatch('struct instance', typeName(v));
      const def = getStructDef(v.type);
      v.fixed = new Set(def ? def.filter(([, fx]) => fx).map(([fname]) => fname) : []);
      v.constructing = false;
      mPush(m, v);
      break;
    }
    case 'AssertInvariant': {
      const v = mPop(m);
      if (!(v.t === 'bool' && v.v === true)) {
        fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: invariant() of ${inst.type} did not hold after construction`);
      }
      break;
    }
    case 'AssertContract': {
      const v = mPeek(m);
      assertContract(m, inst.type, !!inst.null, inst.pos, inst.ops || [], v);
      break;
    }
    case 'CheckMutations': checkMutations(m); break;
    case 'PropagateFailure': {
      const v = mPeek(m);
      if (isFailure(v)) {
        mPop(m);
        handleFailure(m, v);
      }
      break;
    }
    case 'Fail': {
      const v = mPop(m);
      handleFailure(m, isFailure(v) ? v : vFail(display(v)));
      break;
    }
    case 'MakeFunction': {
      const idx = m.funcIndex.get(inst.name);
      if (idx === undefined) fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: unknown function '${inst.name}'`);
      mPush(m, { t: 'func', idx, arity: m.module.functions[idx].params.length });
      break;
    }
    case 'MakeClosure': {
      const idx = m.funcIndex.get(inst.name);
      if (idx === undefined) fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: unknown function '${inst.name}'`);
      const cells = {};
      for (const cap of inst.ups || []) {
        if (inst.self !== undefined && inst.self !== null && cap === inst.self) {
          cells[cap] = { v: vUnset() };
        } else if (Object.hasOwn(fr.vars, cap)) {
          cells[cap] = { v: fr.vars[cap] };
        } else if (fr.cells && Object.hasOwn(fr.cells, cap)) {
          cells[cap] = fr.cells[cap];
        } else if (m.globals.has(cap)) {
          cells[cap] = { v: m.globals.get(cap) };
        } else {
          fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: undefined global '${cap}'`);
        }
      }
      mPush(m, { t: 'closure', idx, arity: m.module.functions[idx].params.length, cells });
      break;
    }
    case 'FillSelfCapture': {
      const c = mPop(m);
      if (c.t !== 'closure') return typeMismatch('Function', typeName(c));
      const cell = c.cells[inst.name];
      if (!cell || cell.v.t !== 'unset') fault('AIPO_RT_TYPE_MISMATCH', 'type mismatch: corrupted closure (missing self placeholder)');
      cell.v = c;
      mPush(m, c);
      break;
    }
    case 'Range': {
      const b = mPop(m);
      const a = mPop(m);
      if (isFailure(a)) { mPush(m, a); break; }
      if (isFailure(b)) { mPush(m, b); break; }
      const x = widen(a), y = widen(b);
      if (x.t !== 'int' || y.t !== 'int') return typeMismatch('Int range bounds', `${typeName(a)} and ${typeName(b)}`);
      mPush(m, vRange(x.v, y.v));
      break;
    }
    case 'Len': {
      const v = mPop(m);
      mPush(m, valLen(v));
      break;
    }
    case 'TypeIs': {
      const tag = mPop(m);
      const v = mPop(m);
      if (isFailure(v)) { mPush(m, v); break; }
      if (tag.t !== 'type') return typeMismatch('type value on the right of is', typeName(tag));
      mPush(m, vBool(tagMatches(tag.name, v)));
      break;
    }
    case 'IterGuard': {
      const v = mPop(m);
      if (v.t === 'list' || v.t === 'dict' || v.t === 'bytes') m.active.push(v.id);
      break;
    }
    case 'IterGuardEnd': m.active.pop(); break;
    case 'PushUnset': mPush(m, vUnset()); break;
    default: fault('AIPO_RT_TYPE_MISMATCH', `type mismatch: unknown instruction ${op}`);
  }
}

function doReturn(m, v) {
  if (isFailure(v)) {
    const fr0 = curFrame(m);
    if (fr0) {
      m.frames.pop();
      m.stack.length = fr0.base;
      m.journal.length = fr0.journalStart;
    }
    handleFailure(m, v);
    return;
  }
  const fr = m.frames.pop();
  if (!fr) {
    mPush(m, v);
    m.result = v;
    m.done = true;
    return;
  }
  m.stack.length = fr.base;
  m.journal.length = fr.journalStart;
  mPush(m, v);
}

// ---- entry point ----
export function runModule(module) {
  const m = makeMachine(module);
  for (const f of module.functions) {
    if (!f.name.includes('.')) {
      m.globals.set(f.name, { t: 'func', idx: m.funcIndex.get(f.name), arity: f.params.length });
    }
  }
  const top = module.top;
  m.frames.push({ fn: top, ip: 0, vars: {}, cells: null, base: 0, journalStart: 0 });
  try {
    while (!m.done) stepFn(m);
  } catch (e) {
    if (e && e.uncaught !== undefined) {
      printFault('AIPO_RT_FAILURE_UNCAUGHT', `uncaught failure: ${e.uncaught}`);
      return 1;
    }
    if (e instanceof AipoFault) {
      printFault(e.code, e.message.replace(/^\[[^\]]+\] /, ''));
      return 1;
    }
    throw e;
  }
  if (m.halted) {
    printFault('AIPO_RT_FAILURE_UNCAUGHT', `uncaught failure: ${m.halted.msg}`);
    return 1;
  }
  const r = m.result;
  if (r && isFailure(r)) {
    printFault('AIPO_RT_FAILURE_UNCAUGHT', `uncaught failure: ${r.msg}`);
    return 1;
  }
  return 0;
}

function printFault(code, message) {
  const text = `error: [${code}] ${message}\n`;
  if (typeof process !== 'undefined' && process.stderr && process.stderr.write) {
    process.stderr.write(text);
  } else {
    emitText(text);
  }
}
