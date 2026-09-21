//! AipoSmith: deterministic generator of small *valid* Aipo programs.
//!
//! The generator is type-directed: an environment tracks every binding's type,
//! so generated programs parse and check cleanly by construction. Anything the
//! checker still rejects is a generator bug, caught by the validity test.
//!
//! Bounds keep programs small and terminating: `while`/`loop` are never
//! generated (only `repeat`/`each` with literal bounds), division uses
//! non-zero literal divisors, and indexing only touches lists of known length.
//!
//! A failure is reproducible from `(seed, config)`: the harness always prints
//! both alongside the generated source.

use crate::rng::Rng;

/// Generation bounds.
#[derive(Debug, Clone)]
pub struct Config {
    /// Total top-level statements (6 minimum, clamped below by the generator).
    pub max_stmts: usize,
    /// Maximum declared functions.
    pub max_fns: usize,
    /// Maximum expression nesting depth.
    pub max_depth: usize,
    /// Maximum list literal length.
    pub max_list_len: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_stmts: 18,
            max_fns: 3,
            max_depth: 3,
            max_list_len: 5,
        }
    }
}

/// A generated program with its reproducing seed.
pub struct Generated {
    /// Program source.
    pub source: String,
    /// Seed that produced it (with this `Config`).
    pub seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ty {
    Int,
    Bool,
    Str,
    List,
}

#[derive(Debug, Clone)]
struct Binding {
    name: String,
    ty: Ty,
    mutable: bool,
    known_len: Option<usize>,
}

struct Gen<'a> {
    rng: &'a mut Rng,
    config: &'a Config,
    counter: usize,
    fns: Vec<(String, usize)>,
    out: String,
    indent: usize,
}

impl Gen<'_> {
    fn fresh(&mut self, prefix: &str) -> String {
        let name = format!("{prefix}{}", self.counter);
        self.counter += 1;
        name
    }

    fn emit(&mut self, line: &str) {
        for _ in 0..self.indent {
            self.out.push_str("    ");
        }
        self.out.push_str(line);
        self.out.push('\n');
    }
}

/// Generates one valid program from `seed`.
#[must_use]
pub fn generate(seed: u64, config: &Config) -> Generated {
    let mut rng = Rng::new(seed);
    let mut gx = Gen {
        rng: &mut rng,
        config,
        counter: 0,
        fns: Vec::new(),
        out: String::new(),
        indent: 0,
    };
    let mut env: Vec<Binding> = Vec::new();
    let target = 6 + gx.rng.below(config.max_stmts.saturating_sub(5).max(1));
    for _ in 0..target {
        gen_stmt(&mut gx, &mut env, 0);
    }
    if !gx.out.contains("io.println") {
        gx.emit("io.println(\"smith\")");
    }
    Generated {
        source: gx.out,
        seed,
    }
}

fn gen_stmt(gx: &mut Gen, env: &mut Vec<Binding>, depth: usize) {
    // Blocks nest at most two deep: deeper nesting would explode program size
    // exponentially and never terminates the generator's own recursion budget.
    if depth >= 2 {
        let roll = gx.rng.below(100);
        if roll < 40 {
            gen_bind(gx, env);
        } else if roll < 55 {
            gen_assign(gx, env);
        } else if roll < 85 {
            let ty = *gx.rng.choose(&[Ty::Int, Ty::Bool, Ty::Str, Ty::List]);
            let expr = gen_expr(gx, env, ty, 0);
            gx.emit(&format!("io.println({expr})"));
        } else if !gx.fns.is_empty() {
            gen_fn_call_stmt(gx, env);
        } else {
            gen_bind(gx, env);
        }
        return;
    }
    let roll = gx.rng.below(100);
    if roll < 28 {
        gen_bind(gx, env);
    } else if roll < 36 {
        gen_assign(gx, env);
    } else if roll < 50 {
        let ty = *gx.rng.choose(&[Ty::Int, Ty::Bool, Ty::Str, Ty::List]);
        let expr = gen_expr(gx, env, ty, 0);
        gx.emit(&format!("io.println({expr})"));
    } else if roll < 60 {
        gen_if(gx, env, depth);
    } else if roll < 68 {
        gen_repeat(gx, env, depth);
    } else if roll < 76 {
        gen_each(gx, env, depth);
    } else if roll < 84 && depth == 0 && gx.fns.len() < gx.config.max_fns {
        // Declarations stay top-level: a function declared inside a block
        // would not be visible to later call sites.
        gen_fn_decl(gx, env);
    } else if roll < 90 && !gx.fns.is_empty() {
        gen_fn_call_stmt(gx, env);
    } else if roll < 95 {
        gen_match(gx, env, depth);
    } else {
        gen_bind(gx, env);
    }
}

fn gen_bind(gx: &mut Gen, env: &mut Vec<Binding>) {
    let ty = *gx.rng.choose(&[Ty::Int, Ty::Bool, Ty::Str, Ty::List]);
    let name = gx.fresh("v");
    let mutable = gx.rng.one_in(3);
    let keyword = if mutable { "var" } else { "let" };
    let (expr, known_len) = match ty {
        Ty::List => {
            let items = gen_list_literal(gx, env);
            let len = items.1;
            (items.0, Some(len))
        }
        _ => (gen_expr(gx, env, ty, 0), None),
    };
    gx.emit(&format!("{keyword} {name} = {expr}"));
    env.push(Binding {
        name,
        ty,
        mutable,
        known_len,
    });
}

fn gen_assign(gx: &mut Gen, env: &mut Vec<Binding>) {
    let candidates: Vec<usize> = env
        .iter()
        .enumerate()
        .filter(|(_, b)| b.mutable)
        .map(|(i, _)| i)
        .collect();
    if candidates.is_empty() {
        gen_bind(gx, env);
        return;
    }
    let at = *gx.rng.choose(&candidates);
    let (name, ty) = (env[at].name.clone(), env[at].ty);
    if ty == Ty::List && gx.rng.one_in(2) {
        let item = gen_expr(gx, env, Ty::Int, 0);
        gx.emit(&format!("{name}.add({item})"));
        env[at].known_len = None;
        return;
    }
    let expr = gen_expr(gx, env, ty, 0);
    gx.emit(&format!("{name} = {expr}"));
    if ty == Ty::List {
        env[at].known_len = None;
    }
}

fn gen_if(gx: &mut Gen, env: &[Binding], depth: usize) {
    let cond = gen_expr(gx, env, Ty::Bool, 0);
    let header = format!("if {cond}");
    gx.emit(&header);
    gx.indent += 1;
    let mut inner = env.to_vec();
    for _ in 0..1 + gx.rng.below(3) {
        gen_stmt(gx, &mut inner, depth + 1);
    }
    if gx.rng.one_in(2) {
        gx.indent -= 1;
        gx.emit("else");
        gx.indent += 1;
        let mut inner_else = env.to_vec();
        for _ in 0..1 + gx.rng.below(2) {
            gen_stmt(gx, &mut inner_else, depth + 1);
        }
    }
    gx.indent -= 1;
    gx.emit("end");
    // Bindings created inside branches stay inside: `env` unchanged.
}

fn gen_repeat(gx: &mut Gen, env: &[Binding], depth: usize) {
    let count = 1 + gx.rng.below(4);
    let header = if gx.rng.one_in(2) {
        let index = gx.fresh("i");
        format!("repeat {count} as {index}")
    } else {
        format!("repeat {count}")
    };
    gx.emit(&header);
    gx.indent += 1;
    let mut inner = env.to_vec();
    for _ in 0..1 + gx.rng.below(3) {
        gen_stmt(gx, &mut inner, depth + 1);
    }
    gx.indent -= 1;
    gx.emit("end");
}

fn gen_each(gx: &mut Gen, env: &[Binding], depth: usize) {
    let lists: Vec<String> = env
        .iter()
        .filter(|b| b.ty == Ty::List)
        .map(|b| b.name.clone())
        .collect();
    let target = if lists.is_empty() || gx.rng.one_in(3) {
        let (literal, _) = gen_list_literal(gx, env);
        literal
    } else {
        gx.rng.choose(&lists).clone()
    };
    let item = gx.fresh("e");
    let header = if gx.rng.one_in(3) {
        let index = gx.fresh("k");
        format!("each {index}, {item} in {target}")
    } else {
        format!("each {item} in {target}")
    };
    gx.emit(&header);
    gx.indent += 1;
    let mut inner = env.to_vec();
    inner.push(Binding {
        name: item.clone(),
        ty: Ty::Int,
        mutable: false,
        known_len: None,
    });
    gx.emit(&format!("io.println({item})"));
    if depth < 2 && gx.rng.one_in(3) {
        gen_stmt(gx, &mut inner, depth + 1);
    }
    gx.indent -= 1;
    gx.emit("end");
}

fn gen_match(gx: &mut Gen, env: &[Binding], depth: usize) {
    let subject = match env.iter().find(|b| b.ty == Ty::Int) {
        Some(binding) => binding.name.clone(),
        // No integer in scope: match a literal instead of declaring a binding
        // this scope cannot register.
        None => "0".to_string(),
    };
    gx.emit(&format!("match {subject}"));
    gx.indent += 1;
    for arm in [0, 1] {
        gx.emit(&format!("when {arm}"));
        gx.indent += 1;
        let mut inner = env.to_vec();
        gen_stmt(gx, &mut inner, depth + 1);
        gx.indent -= 1;
    }
    gx.emit("else");
    gx.indent += 1;
    let mut inner = env.to_vec();
    gen_stmt(gx, &mut inner, depth + 1);
    gx.indent -= 1;
    gx.indent -= 1;
    gx.emit("end");
}

fn gen_fn_decl(gx: &mut Gen, env: &[Binding]) {
    let name = gx.fresh("f");
    let arity = gx.rng.below(3);
    let mut params = Vec::new();
    for _ in 0..arity {
        params.push(gx.fresh("p"));
    }
    gx.emit(&format!("fn {name}({})", params.join(", ")));
    gx.indent += 1;
    let mut inner: Vec<Binding> = params
        .iter()
        .map(|p| Binding {
            name: p.clone(),
            ty: Ty::Int,
            mutable: false,
            known_len: None,
        })
        .collect();
    for _ in 0..1 + gx.rng.below(3) {
        gen_int_stmt(&mut *gx, &mut inner);
    }
    let result = gen_expr(gx, &inner, Ty::Int, 0);
    gx.emit(&format!("return {result}"));
    gx.indent -= 1;
    gx.emit("end");
    gx.fns.push((name.clone(), arity));
    // Exercise the function immediately with a visible result.
    let args: Vec<String> = (0..arity).map(|_| gx.rng.below(20).to_string()).collect();
    gx.emit(&format!("io.println({name}({}))", args.join(", ")));
    let _ = env;
}

fn gen_int_stmt(gx: &mut Gen, env: &mut Vec<Binding>) {
    if gx.rng.one_in(2) {
        let name = gx.fresh("v");
        let expr = gen_expr(gx, env, Ty::Int, 0);
        gx.emit(&format!("let {name} = {expr}"));
        env.push(Binding {
            name,
            ty: Ty::Int,
            mutable: false,
            known_len: None,
        });
    } else {
        let expr = gen_expr(gx, env, Ty::Int, 0);
        gx.emit(&format!("io.println({expr})"));
    }
}

fn gen_fn_call_stmt(gx: &mut Gen, env: &[Binding]) {
    let at = gx.rng.below(gx.fns.len());
    let (name, arity) = gx.fns[at].clone();
    let args: Vec<String> = (0..arity).map(|_| gen_expr(gx, env, Ty::Int, 1)).collect();
    gx.emit(&format!("io.println({name}({}))", args.join(", ")));
}

fn gen_list_literal(gx: &mut Gen, env: &[Binding]) -> (String, usize) {
    let len = gx.rng.below(gx.config.max_list_len + 1);
    let mut items = Vec::new();
    for _ in 0..len {
        items.push(gen_expr(gx, env, Ty::Int, 1));
    }
    (format!("[{}]", items.join(", ")), len)
}

fn int_vars(env: &[Binding]) -> Vec<String> {
    env.iter()
        .filter(|b| b.ty == Ty::Int)
        .map(|b| b.name.clone())
        .collect()
}

fn gen_expr(gx: &mut Gen, env: &[Binding], ty: Ty, depth: usize) -> String {
    let max = gx.config.max_depth;
    if depth >= max {
        return gen_leaf(gx, env, ty);
    }
    let roll = gx.rng.below(100);
    match ty {
        Ty::Int => {
            if roll < 35 {
                return gen_leaf(gx, env, ty);
            }
            if roll < 60 {
                let op = gx.rng.choose(&["+", "-", "*"]);
                let left = gen_expr(gx, env, Ty::Int, depth + 1);
                let right = gen_expr(gx, env, Ty::Int, depth + 1);
                return format!("{left} {op} {right}");
            }
            if roll < 68 {
                let left = gen_expr(gx, env, Ty::Int, depth + 1);
                let divisor = 1 + gx.rng.below(9);
                let op = gx.rng.choose(&["/", "div", "%"]);
                return format!("{left} {op} {divisor}");
            }
            if roll < 76 {
                let strings: Vec<String> = env
                    .iter()
                    .filter(|b| b.ty == Ty::Str || b.ty == Ty::List)
                    .map(|b| b.name.clone())
                    .collect();
                if !strings.is_empty() {
                    let target = gx.rng.choose(&strings).clone();
                    return format!("len({target})");
                }
                return gen_leaf(gx, env, ty);
            }
            if roll < 84 {
                return format!("Int(\"{}\")", gx.rng.below(1000));
            }
            gen_index_expr(gx, env)
        }
        Ty::Bool => {
            if roll < 30 {
                return gen_leaf(gx, env, ty);
            }
            if roll < 62 {
                let op = gx.rng.choose(&["==", "!=", "<", "<=", ">", ">="]);
                let left = gen_expr(gx, env, Ty::Int, depth + 1);
                let right = gen_expr(gx, env, Ty::Int, depth + 1);
                return format!("{left} {op} {right}");
            }
            if roll < 74 {
                let left = gen_expr(gx, env, Ty::Bool, depth + 1);
                let right = gen_expr(gx, env, Ty::Bool, depth + 1);
                let op = gx.rng.choose(&["and", "or"]);
                return format!("{left} {op} {right}");
            }
            if roll < 82 {
                let inner = gen_expr(gx, env, Ty::Bool, depth + 1);
                return format!("not {inner}");
            }
            let target = gen_expr(gx, env, Ty::Int, depth + 1);
            format!("{target} is Int")
        }
        Ty::Str => {
            if roll < 35 {
                return gen_leaf(gx, env, ty);
            }
            if roll < 55 {
                let left = gen_expr(gx, env, Ty::Str, depth + 1);
                let right = gen_expr(gx, env, Ty::Str, depth + 1);
                return format!("{left} + {right}");
            }
            if roll < 70 {
                // Only plain integers interpolate: any quoted text inside the
                // braces would terminate the outer string literal. The double
                // braces emit Aipo `{…}` interpolation markers.
                let value = gen_leaf(gx, env, Ty::Int);
                return format!("f\"n={{{value}}}\"");
            }
            format!("String({})", gen_expr(gx, env, Ty::Int, depth + 1))
        }
        Ty::List => {
            if roll < 45 {
                let known: Vec<String> = env
                    .iter()
                    .filter(|b| b.ty == Ty::List)
                    .map(|b| b.name.clone())
                    .collect();
                if !known.is_empty() {
                    return gx.rng.choose(&known).clone();
                }
            }
            if roll < 60 && !env.iter().any(|b| b.ty == Ty::List) {
                return "[0]".to_string();
            }
            // Higher-order call over a literal: always valid, always terminates.
            let (literal, _) = gen_list_literal(gx, env);
            if gx.rng.one_in(2) {
                format!("{literal}.transform(fn (x) return x * 2 end)")
            } else {
                format!("{literal}.filter(fn (x) return x > 1 end)")
            }
        }
    }
}

fn gen_leaf(gx: &mut Gen, env: &[Binding], ty: Ty) -> String {
    match ty {
        Ty::Int => {
            let vars = int_vars(env);
            if !vars.is_empty() && gx.rng.one_in(2) {
                gx.rng.choose(&vars).clone()
            } else {
                let base = gx.rng.below(200);
                if gx.rng.one_in(4) {
                    format!("-{}", base + 1)
                } else {
                    base.to_string()
                }
            }
        }
        Ty::Bool => {
            let vars: Vec<String> = env
                .iter()
                .filter(|b| b.ty == Ty::Bool)
                .map(|b| b.name.clone())
                .collect();
            if !vars.is_empty() && gx.rng.one_in(2) {
                gx.rng.choose(&vars).clone()
            } else if gx.rng.one_in(2) {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Ty::Str => {
            let vars: Vec<String> = env
                .iter()
                .filter(|b| b.ty == Ty::Str)
                .map(|b| b.name.clone())
                .collect();
            if !vars.is_empty() && gx.rng.one_in(2) {
                gx.rng.choose(&vars).clone()
            } else {
                let words = ["a", "hi", "x", "hey", "yo", "eh", "oh"];
                format!("\"{}\"", gx.rng.choose(&words))
            }
        }
        Ty::List => "[1]".to_string(),
    }
}

fn gen_index_expr(gx: &mut Gen, env: &[Binding]) -> String {
    let known: Vec<String> = env
        .iter()
        .filter(|b| b.ty == Ty::List && b.known_len.is_some_and(|len| len > 0))
        .map(|b| b.name.clone())
        .collect();
    if known.is_empty() {
        return gen_leaf(gx, env, Ty::Int);
    }
    let target = gx.rng.choose(&known).clone();
    format!("{target}[0]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::check_text;

    #[test]
    fn test_generated_programs_are_valid() {
        let config = Config::default();
        for seed in 0..200u64 {
            let generated = generate(seed, &config);
            if let Err(diagnostics) = check_text("smith.aipo", &generated.source) {
                panic!(
                    "seed {seed} produced an invalid program:\n{}\n diagnostics: {diagnostics:?}",
                    generated.source
                );
            }
        }
    }

    #[test]
    fn test_generation_is_deterministic() {
        let config = Config::default();
        for seed in [0u64, 1, 7, 42, 999] {
            assert_eq!(
                generate(seed, &config).source,
                generate(seed, &config).source,
                "seed {seed} must reproduce"
            );
        }
    }
}
