// Cham interpreter — a TypeScript port of the Rust engine.
// Parses .cham files and runs them against an in-memory store.

export interface ChamError {
  line: number;
  text: string;
}

export type Expr =
  | { t: "text"; v: string }
  | { t: "num"; v: number }
  | { t: "now" }
  | { t: "not"; inner: Expr }
  | { t: "all"; coll: string }
  | { t: "var"; path: string[] }
  | { t: "bin"; op: string; l: Expr; r: Expr };

export type Action =
  | { t: "value"; e: Expr }
  | { t: "get"; coll: string; fields: string[] }
  | { t: "save"; fields: string[]; coll: string }
  | { t: "delete"; coll: string; fields: string[] };

export type Step =
  | { t: "need"; fields: string[] }
  | { t: "set"; name: string | null; action: Action }
  | { t: "if"; cond: Expr; body: Line[] }
  | { t: "answer"; status: number; main: Expr; extra: Expr | null };

export interface Line {
  n: number;
  step: Step;
}

export interface Route {
  method: string;
  path: string;
  body: Line[];
}

export interface ChamApp {
  settings: Map<string, Expr>;
  routes: Route[];
}

type Json = string | number | boolean | null | Json[] | { [k: string]: Json };
type Vars = Map<string, Json>;

const WORDS = ["when", "need", "if", "not", "get", "all", "save", "delete", "answer", "now"];

// ---------- tokenizer ----------

type Tok =
  | { k: "text"; v: string }
  | { k: "num"; v: number }
  | { k: "word"; v: string }
  | { k: "sym"; v: string };

function lex(src: string): Tok[] | string {
  const out: Tok[] = [];
  let i = 0;
  const chars = [...src];
  while (i < chars.length) {
    const c = chars[i];
    if (c === " " || c === "\t" || c === "\n" || c === "\r") { i++; continue; }
    if (c === '"') {
      i++;
      let s = "";
      while (i < chars.length && chars[i] !== '"') { s += chars[i]; i++; }
      if (i >= chars.length) return "unclosed_quote";
      i++;
      out.push({ k: "text", v: s });
      continue;
    }
    if (c >= "0" && c <= "9") {
      let s = "";
      while (i < chars.length && ((chars[i] >= "0" && chars[i] <= "9") || chars[i] === ".")) { s += chars[i]; i++; }
      out.push({ k: "num", v: parseFloat(s) || 0 });
      continue;
    }
    if (/[a-zA-Z_]/.test(c)) {
      let s = "";
      while (i < chars.length && /[a-zA-Z0-9_.]/.test(chars[i])) { s += chars[i]; i++; }
      out.push({ k: "word", v: s });
      continue;
    }
    const two = chars.slice(i, i + 2).join("");
    if (["==", "!=", ">=", "<="].includes(two)) { out.push({ k: "sym", v: two }); i += 2; continue; }
    if ("+-=<>,:()".includes(c)) { out.push({ k: "sym", v: c }); i++; continue; }
    return `leftover:${c}`;
  }
  return out;
}

// ---------- parser ----------

class Parser {
  i = 0;
  constructor(public toks: Tok[]) {}

  peek(): Tok | undefined { return this.toks[this.i]; }
  isSym(s: string): boolean { const t = this.peek(); return !!t && t.k === "sym" && t.v === s; }
  isWord(s: string): boolean { const t = this.peek(); return !!t && t.k === "word" && t.v === s; }

  word(): string | null {
    const t = this.peek();
    if (t && t.k === "word") { this.i++; return t.v; }
    return null;
  }

  names(stop?: string): string[] | null {
    const first = this.word();
    if (!first) return null;
    const out = [first];
    while (this.isSym(",")) {
      this.i++;
      const w = this.word();
      if (!w) return null;
      out.push(w);
    }
    if (stop && !this.isWord(stop)) return null;
    return out;
  }

  done(): boolean { return this.i >= this.toks.length; }

  expr(): Expr | null {
    let l = this.add();
    if (!l) return null;
    for (const op of ["==", "!=", ">=", "<=", ">", "<"]) {
      if (this.isSym(op)) {
        this.i++;
        const r = this.add();
        if (!r) return null;
        return { t: "bin", op, l, r };
      }
    }
    return l;
  }

  add(): Expr | null {
    let l = this.unary();
    if (!l) return null;
    while (this.isSym("+") || this.isSym("-")) {
      const op = this.isSym("+") ? "+" : "-";
      this.i++;
      const r = this.unary();
      if (!r) return null;
      l = { t: "bin", op, l, r };
    }
    return l;
  }

  unary(): Expr | null {
    if (this.isWord("not")) {
      this.i++;
      const inner = this.unary();
      if (!inner) return null;
      return { t: "not", inner };
    }
    return this.primary();
  }

  primary(): Expr | null {
    const t = this.peek();
    if (!t) return null;
    this.i++;
    switch (t.k) {
      case "text": return { t: "text", v: t.v };
      case "num": return { t: "num", v: t.v };
      case "sym":
        if (t.v === "(") {
          const e = this.expr();
          if (!e || !this.isSym(")")) return null;
          this.i++;
          return e;
        }
        return null;
      case "word":
        if (t.v === "now") return { t: "now" };
        if (t.v === "all") {
          const coll = this.word();
          if (!coll) return null;
          return { t: "all", coll };
        }
        // detect "foo bar" pattern (call to unknown function)
        const next = this.peek();
        if (next && (next.k === "word" || next.k === "text" || next.k === "num")) return null;
        return { t: "var", path: t.v.split(".") };
    }
  }

  action(): Action | null {
    if (this.isWord("get")) {
      this.i++;
      const coll = this.word();
      if (!coll) return null;
      const fields = this.names();
      if (!fields) return null;
      return { t: "get", coll, fields };
    }
    if (this.isWord("delete")) {
      this.i++;
      const coll = this.word();
      if (!coll) return null;
      const fields = this.names();
      if (!fields) return null;
      return { t: "delete", coll, fields };
    }
    if (this.isWord("save")) {
      this.i++;
      const fields = this.names("to");
      if (!fields) return null;
      this.i++; // skip "to"
      const coll = this.word();
      if (!coll) return null;
      return { t: "save", fields, coll };
    }
    const e = this.expr();
    if (!e) return null;
    return { t: "value", e };
  }
}

function stripComment(line: string): string {
  let quoted = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (c === '"') quoted = !quoted;
    if (c === "#" && !quoted) return line.slice(0, i);
  }
  return line;
}

function distance(a: string, b: string): number {
  const m = a.length, n = b.length;
  if (m === 0) return n;
  if (n === 0) return m;
  const dp: number[] = Array.from({ length: n + 1 }, (_, j) => j);
  for (let i = 1; i <= m; i++) {
    let prev = dp[0];
    dp[0] = i;
    for (let j = 1; j <= n; j++) {
      const tmp = dp[j];
      dp[j] = Math.min(dp[j] + 1, dp[j - 1] + 1, prev + (a[i - 1] === b[j - 1] ? 0 : 1));
      prev = tmp;
    }
  }
  return dp[n];
}

function closest(word: string): string | null {
  let best: string | null = null;
  let bestD = 3;
  for (const w of WORDS) {
    const d = distance(word, w);
    if (d <= 2 && d < bestD) { bestD = d; best = w; }
  }
  return best;
}

interface RawLine { n: number; indent: number; text: string; }

function parseStep(toks: Tok[]): { step: Step; wantsBlock: boolean } | string {
  const p = new Parser(toks);
  const first = p.peek();
  if (!first || first.k !== "word") {
    if (!first) return "needs_arg";
    return `leftover:${showTok(first)}`;
  }
  const name = first.v;

  switch (name) {
    case "need": {
      p.i++;
      const fields = p.names();
      if (!fields) return "needs_arg:need";
      if (!p.done()) return `leftover:${showTok(p.peek()!)}`;
      return { step: { t: "need", fields }, wantsBlock: false };
    }
    case "if": {
      p.i++;
      const cond = p.expr();
      if (!cond) return "needs_arg:if";
      if (!p.isSym(":")) return "need_colon";
      p.i++;
      if (!p.peek()) return { step: { t: "if", cond, body: [] }, wantsBlock: true };
      const inner = parseStep(toks.slice(p.i));
      if (typeof inner === "string") return inner;
      return { step: { t: "if", cond, body: [{ n: 0, step: inner.step }] }, wantsBlock: false };
    }
    case "answer": {
      p.i++;
      let status = 200;
      const a = p.toks[p.i], b = p.toks[p.i + 1];
      if (a && a.k === "num" && b && !(b.k === "sym" && (b.v === "+" || b.v === "-"))) {
        status = a.v as number;
        if (status < 100 || status >= 600) return "bad_status";
        p.i++;
      }
      const main = p.expr();
      if (!main) return "needs_arg:answer";
      let extra: Expr | null = null;
      if (p.isSym(",")) { p.i++; extra = p.expr(); if (!extra) return "needs_arg:answer"; }
      if (!p.done()) return `leftover:${showTok(p.peek()!)}`;
      return { step: { t: "answer", status, main, extra }, wantsBlock: false };
    }
    case "get":
    case "save":
    case "delete": {
      const action = p.action();
      if (!action) return `needs_arg:${name}`;
      if (!p.done()) return `leftover:${showTok(p.peek()!)}`;
      return { step: { t: "set", name: null, action }, wantsBlock: false };
    }
  }

  // assignment: name = ...
  if (p.toks[1] && p.toks[1].k === "sym" && p.toks[1].v === "=") {
    p.i += 2;
    const action = p.action();
    if (!action) return `needs_arg:${name}`;
    if (!p.done()) return `leftover:${showTok(p.peek()!)}`;
    return { step: { t: "set", name, action }, wantsBlock: false };
  }

  const hint = closest(name);
  return hint ? `unknown:${name}:${hint}` : `unknown:${name}`;
}

function showTok(t: Tok): string {
  switch (t.k) {
    case "text": return `"${t.v}"`;
    case "num": return String(t.v);
    case "word":
    case "sym": return t.v;
  }
}

function routeHead(text: string): { method: string; path: string } | string {
  if (!text.endsWith(":")) return "need_colon";
  const head = text.slice(0, -1);
  const words = head.split(/\s+/).filter(w => w !== "someone");
  const method = words[1];
  let m = "";
  if (method === "posts") m = "POST";
  else if (method === "opens") m = "GET";
  else if (method === "deletes") m = "DELETE";
  else return "bad_when";
  const p = words[2];
  if (!p || !p.startsWith("/") || words.length !== 3) return "bad_when";
  return { method: m, path: p };
}

export function parse(src: string): { app: ChamApp } | { error: ChamError } {
  const lines: RawLine[] = [];
  src.split("\n").forEach((raw, idx) => {
    const stripped = stripComment(raw).trimEnd();
    if (stripped.trim().length === 0) return;
    const indent = stripped.length - stripped.trimStart().length;
    lines.push({ n: idx + 1, indent, text: stripped.trim() });
  });

  const app: ChamApp = { settings: new Map(), routes: [] };
  let i = 0;

  function fail(n: number, text: string): { error: ChamError } {
    return { error: { line: n, text } };
  }

  while (i < lines.length) {
    const l = lines[i];
    if (l.indent > 0) return fail(l.n, "bad_indent");

    if (l.text.startsWith("when")) {
      const head = routeHead(l.text);
      if (typeof head === "string") return fail(l.n, head);
      i++;
      const body = parseBlock(lines, i, l.indent);
      if ("error" in body) return fail(body.error.line, body.error.text);
      i = body.nextI;
      app.routes.push({ method: head.method, path: head.path, body: body.lines });
      continue;
    }

    const toks = lex(l.text);
    if (typeof toks === "string") return fail(l.n, toks);
    const result = parseStep(toks);
    if (typeof result === "string") return fail(l.n, result);
    if (result.step.t === "set" && result.step.name && result.step.action.t === "value") {
      app.settings.set(result.step.name, result.step.action.e);
    } else {
      const w = l.text.split(/\s+/)[0];
      return fail(l.n, `not_indented:${w}`);
    }
    i++;
  }

  return { app };
}

function parseBlock(lines: RawLine[], startI: number, parentIndent: number):
  { lines: Line[]; nextI: number } | { error: ChamError } {
  const out: Line[] = [];
  const first = lines[startI];
  if (!first || first.indent <= parentIndent) return { lines: out, nextI: startI };
  const indent = first.indent;
  let i = startI;

  while (i < lines.length) {
    const l = lines[i];
    if (l.indent < indent) {
      if (l.indent > parentIndent) return { error: { line: l.n, text: "bad_indent" } };
      break;
    }
    if (l.indent > indent) return { error: { line: l.n, text: "bad_indent" } };
    if (l.text.startsWith("when ")) return { error: { line: l.n, text: "bad_indent" } };

    const toks = lex(l.text);
    if (typeof toks === "string") return { error: { line: l.n, text: toks } };
    const result = parseStep(toks);
    if (typeof result === "string") return { error: { line: l.n, text: result } };

    const n = l.n;
    i++;

    if (result.wantsBlock) {
      const inner = parseBlock(lines, i, indent);
      if ("error" in inner) return inner;
      if (inner.lines.length === 0) return { error: { line: n, text: "empty_if" } };
      if (result.step.t === "if") result.step.body = inner.lines;
      i = inner.nextI;
    } else if (result.step.t === "if" && result.step.body.length > 0) {
      result.step.body[0].n = n;
    }

    out.push({ n, step: result.step });
  }

  return { lines: out, nextI: i };
}

// ---------- store ----------

export class Store {
  colls: Map<string, Json[]> = new Map();

  all(name: string): Json[] {
    if (!this.colls.has(name)) this.colls.set(name, []);
    return this.colls.get(name)!;
  }

  insert(name: string, rec: Record<string, Json>): Json {
    const rows = this.all(name);
    const ids = rows.filter(r => r && typeof r === "object" && !Array.isArray(r))
      .map(r => (r as { [k: string]: Json })["id"])
      .filter(v => typeof v === "number") as number[];
    const id = (ids.length ? Math.max(...ids) : 0) + 1;
    const full = { ...rec, id } as Json;
    rows.push(full);
    return full;
  }

  find(name: string, want: Record<string, Json>): Json | null {
    const rows = this.all(name);
    return rows.find(r => matches(r, want)) ?? null;
  }

  remove(name: string, want: Record<string, Json>): number {
    const rows = this.all(name);
    const before = rows.length;
    const kept = rows.filter(r => !matches(r, want));
    this.colls.set(name, kept);
    return before - kept.length;
  }

  newFields(name: string, rec: Record<string, Json>): string[] {
    const rows = this.all(name);
    const known = new Set<string>();
    for (const r of rows) {
      if (r && typeof r === "object" && !Array.isArray(r)) {
        Object.keys(r as object).forEach(k => known.add(k));
      }
    }
    if (known.size === 0) return [];
    return Object.keys(rec).filter(k => !known.has(k));
  }
}

function matches(row: Json, want: Record<string, Json>): boolean {
  if (!row || typeof row !== "object" || Array.isArray(row)) return false;
  const r = row as { [k: string]: Json };
  return Object.entries(want).every(([k, v]) => textEq(r[k], v));
}

function textEq(a: Json, b: Json): boolean {
  return toText(a) === toText(b);
}

// ---------- evaluator ----------

type Flow = { done: false } | { done: true; reply: Reply };

export interface Reply {
  status: number;
  body: Json;
}

export function runRoute(
  route: Route,
  settings: Map<string, Expr>,
  params: Record<string, string>,
  rawBody: string,
  store: Store
): Reply {
  const vars: Vars = new Map();

  // settings
  for (const [k, e] of settings) {
    const v = evalExpr(e, vars, store);
    if (v !== null) vars.set(k, v);
  }

  // body
  if (rawBody.trim()) {
    try {
      const parsed = JSON.parse(rawBody);
      if (typeof parsed === "object" && parsed !== null && !Array.isArray(parsed)) {
        for (const [k, v] of Object.entries(parsed)) vars.set(k, v as Json);
      } else {
        return { status: 400, body: { error: `Expected a JSON object, got ${JSON.stringify(parsed)}` } };
      }
    } catch (e) {
      return { status: 400, body: { error: `Request body is not valid JSON (${(e as Error).message})` } };
    }
  }

  // path params
  for (const [k, v] of Object.entries(params)) vars.set(k, v);

  const flow = runSteps(route.body, vars, store);
  if (flow.done) return flow.reply;
  return { status: 200, body: {} };
}

function runSteps(lines: Line[], vars: Vars, store: Store): Flow {
  for (const line of lines) {
    const step = line.step;
    switch (step.t) {
      case "need": {
        for (const f of step.fields) {
          const v = vars.get(f);
          if (v === undefined || v === null) return done(422, { error: `Field "${f}" is missing or empty. This route needs: ${step.fields.join(", ")}.` });
          if (typeof v === "string" && v.trim() === "") return done(422, { error: `Field "${f}" is missing or empty. This route needs: ${step.fields.join(", ")}.` });
        }
        break;
      }
      case "set": {
        const v = doAction(step.action, vars, store, line.n);
        if ("error" in v) return done(v.error.status, { error: v.error.text });
        if (step.name) vars.set(step.name, v.value);
        break;
      }
      case "if": {
        const cond = evalExpr(step.cond, vars, store);
        if (truthy(cond)) {
          const inner = runSteps(step.body, vars, store);
          if (inner.done) return inner;
        }
        break;
      }
      case "answer": {
        const main = evalExpr(step.main, vars, store);
        const extra = step.extra ? evalExpr(step.extra, vars, store) : null;
        let body: Json;
        if (typeof main === "string" && step.status >= 400) {
          body = { error: main };
        } else if (typeof main === "string" && extra !== null) {
          body = { message: main, data: extra };
        } else if (typeof main === "string") {
          body = { message: main };
        } else {
          body = main;
        }
        return done(step.status, body);
      }
    }
  }
  return { done: false };
}

function done(status: number, body: Json): Flow {
  return { done: true, reply: { status, body } };
}

function doAction(action: Action, vars: Vars, store: Store, lineN: number):
  { value: Json } | { error: { status: number; text: string } } {
  switch (action.t) {
    case "value":
      return { value: evalExpr(action.e, vars, store) };
    case "get": {
      const want = pickFields(action.fields, vars, lineN);
      if ("error" in want) return want;
      return { value: store.find(action.coll, want.value) };
    }
    case "save": {
      const rec = pickFields(action.fields, vars, lineN);
      if ("error" in rec) return rec;
      for (const f of store.newFields(action.coll, rec.value)) {
        console.log(`note: "${action.coll}" got a new field "${f}". Stored it, no migration needed.`);
      }
      return { value: store.insert(action.coll, rec.value) };
    }
    case "delete": {
      const want = pickFields(action.fields, vars, lineN);
      if ("error" in want) return want;
      const count = store.remove(action.coll, want.value);
      return { value: count };
    }
  }
}

function pickFields(fields: string[], vars: Vars, lineN: number):
  { value: Record<string, Json> } | { error: { status: number; text: string } } {
  const out: Record<string, Json> = {};
  for (const f of fields) {
    const v = vars.get(f);
    if (v === undefined) return { error: { status: 400, text: `You used "${f}", but the request didn't send it and nothing set it before. Send "${f}" with the request or write ${f} = ... above this line (line ${lineN}).` } };
    out[f] = v;
  }
  return { value: out };
}

function evalExpr(e: Expr, vars: Vars, store: Store): Json {
  switch (e.t) {
    case "text": return e.v;
    case "num": return num(e.v);
    case "now": return Math.floor(Date.now() / 1000);
    case "all": return store.all(e.coll);
    case "not": return !truthy(evalExpr(e.inner, vars, store));
    case "var": {
      let v: Json | undefined = vars.get(e.path[0]);
      if (v === undefined) v = null;
      for (const key of e.path.slice(1)) {
        if (v && typeof v === "object" && !Array.isArray(v)) {
          v = (v as { [k: string]: Json })[key] ?? null;
        } else {
          v = null;
        }
      }
      return v ?? null;
    }
    case "bin": {
      const a = evalExpr(e.l, vars, store);
      const b = evalExpr(e.r, vars, store);
      const af = toNum(a), bf = toNum(b);
      switch (e.op) {
        case "+":
          if (af !== null && bf !== null) return num(af + bf);
          return toText(a) + toText(b);
        case "-":
          if (af !== null && bf !== null) return num(af - bf);
          return null;
        case "==": return toText(a) === toText(b);
        case "!=": return toText(a) !== toText(b);
        case ">": return af !== null && bf !== null ? af > bf : false;
        case "<": return af !== null && bf !== null ? af < bf : false;
        case ">=": return af !== null && bf !== null ? af >= bf : false;
        case "<=": return af !== null && bf !== null ? af <= bf : false;
        default: return null;
      }
    }
  }
}

function num(x: number): Json {
  if (Number.isInteger(x) && Math.abs(x) < 1e15) return x;
  return x;
}

function toNum(v: Json): number | null {
  if (typeof v === "number") return v;
  if (typeof v === "string") {
    const n = parseFloat(v);
    return isNaN(n) ? null : n;
  }
  if (typeof v === "boolean") return v ? 1 : 0;
  return null;
}

function truthy(v: Json): boolean {
  if (v === null) return false;
  if (typeof v === "boolean") return v;
  if (typeof v === "number") return v !== 0;
  if (typeof v === "string") return v.length > 0;
  if (Array.isArray(v)) return v.length > 0;
  return true;
}

function toText(v: Json): string {
  if (v === null) return "null";
  if (typeof v === "string") return v;
  return JSON.stringify(v);
}

// ---------- HTTP matching ----------

export function matchPath(pattern: string, path: string): Record<string, string> | null {
  const a = pattern.replace(/^\/|\/$/g, "").split("/");
  const b = path.replace(/^\/|\/$/g, "").split("/");
  if (a.length !== b.length) return null;
  const params: Record<string, string> = {};
  for (let i = 0; i < a.length; i++) {
    if (a[i].startsWith(":")) {
      params[a[i].slice(1)] = b[i];
    } else if (a[i] !== b[i]) {
      return null;
    }
  }
  return params;
}

export interface RequestResult {
  status: number;
  body: Json;
  matchedRoute: string | null;
}

export function handleRequest(
  app: ChamApp,
  method: string,
  path: string,
  body: string,
  store: Store
): RequestResult {
  for (const route of app.routes) {
    if (route.method !== method.toUpperCase()) continue;
    const params = matchPath(route.path, path);
    if (params) {
      const reply = runRoute(route, app.settings, params, body, store);
      return { status: reply.status, body: reply.body, matchedRoute: `${route.method} ${route.path}` };
    }
  }
  const known = app.routes.map(r => `${r.method} ${r.path}`).join(", ");
  return {
    status: 404,
    body: { error: `Nothing answers ${method} ${path}. Routes in this app: ${known}` },
    matchedRoute: null,
  };
}

// ---------- error messages ----------

export function formatError(err: ChamError, src: string, lang: "en" | "uk" | "ru" = "en"): string {
  const messages: Record<string, Record<string, string>> = {
    en: {
      unclosed_quote: "A quote opens here but never closes. Add \" at the end of the text.",
      leftover: "I understood the start of the line but not something here. Check for a missing + or comma.",
      needs_arg: "Something is missing after a keyword. Check the syntax.",
      need_colon: "This line should end with a colon.",
      bad_status: "That is not a valid HTTP status. Use a number like 200 or 201.",
      bad_when: "I can't read this route. Write: when someone posts /orders:  (or opens, deletes)",
      bad_indent: "The indentation here doesn't match the lines above. Line it up with them, or indent it under an if.",
      not_indented: "This line has to be inside a route. Indent it under a line like: when someone posts /orders:",
      empty_if: "This if has nothing to do. Put a step after the colon or indent the next lines under it.",
      unknown: "I don't know the word \"{0}\".{1}",
      did_you_mean: " Did you mean \"{0}\"?",
    },
    uk: {
      unclosed_quote: "Лапки відкриваються, але не закриваються. Додайте \" в кінці тексту.",
      leftover: "Початок рядка зрозумів, а далі ні. Перевірте, чи не бракує + або коми.",
      needs_arg: "Після слова чогось бракує. Перевірте синтаксис.",
      need_colon: "Цей рядок має закінчуватися двокрапкою.",
      bad_status: "Це не HTTP-статус. Напишіть число, наприклад 200 або 201.",
      bad_when: "Не можу прочитати маршрут. Напишіть: when someone posts /orders:  (або opens, deletes)",
      bad_indent: "Відступ тут не збігається з рядками вище. Вирівняйте його або поставте під if.",
      not_indented: "Цей рядок має бути всередині маршруту. Зробіть відступ під рядком на кшталт: when someone posts /orders:",
      empty_if: "Цей if нічого не робить. Напишіть крок після двокрапки або зробіть відступ для наступних рядків.",
      unknown: "Я не знаю слова \"{0}\".{1}",
      did_you_mean: " Можливо, ви мали на увазі \"{0}\"?",
    },
    ru: {
      unclosed_quote: "Кавычка открыта, но не закрыта. Добавьте \" в конце текста.",
      leftover: "Начало строки понял, а дальше нет. Проверьте, не пропущен ли + или запятая.",
      needs_arg: "После слова чего-то не хватает. Проверьте синтаксис.",
      need_colon: "Эта строка должна заканчиваться двоеточием.",
      bad_status: "Это не HTTP-статус. Напишите число, например 200 или 201.",
      bad_when: "Не могу прочитать маршрут. Напишите: when someone posts /orders:  (или opens, deletes)",
      bad_indent: "Отступ здесь не совпадает со строками выше. Выровняйте его или поставьте под if.",
      not_indented: "Эта строка должна быть внутри маршрута. Сделайте отступ под строкой вроде: when someone posts /orders:",
      empty_if: "Этот if ничего не делает. Напишите шаг после двоеточия или сделайте отступ для следующих строк.",
      unknown: "Я не знаю слова \"{0}\".{1}",
      did_you_mean: " Возможно, вы имели в виду \"{0}\"?",
    },
  };

  const t = messages[lang] || messages.en;
  const parts = err.text.split(":");
  const key = parts[0];
  let msg = t[key] || err.text;

  if (key === "unknown") {
    const word = parts[1] || "";
    const hint = parts[2] || "";
    msg = t.unknown
      .replace("{0}", word)
      .replace("{1}", hint ? t.did_you_mean.replace("{0}", hint) : "");
  } else if (key === "leftover") {
    msg = t.leftover;
  } else if (key === "not_indented") {
    msg = t.not_indented;
  }

  const lineWord = lang === "uk" ? "рядок" : lang === "ru" ? "строка" : "line";
  return `${src.split("\n")[0]?.includes("app.cham") ? "app.cham" : "playground"}, ${lineWord} ${err.line}: ${msg}`;
}
