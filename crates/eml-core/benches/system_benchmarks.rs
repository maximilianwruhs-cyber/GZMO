use criterion::{black_box, criterion_group, criterion_main, Criterion};

use eml_core::{execute, synth, ComplexBall, EmlExpr};

fn eval(expr: EmlExpr, args: &[f64]) -> ComplexBall {
    let prog = expr.compile();
    let balls: Vec<_> = args.iter().map(|&v| ComplexBall::from_real(v)).collect();
    execute(&prog, &balls).unwrap()
}

fn bench_eml_primitives(c: &mut Criterion) {
    let mut group = c.benchmark_group("eml_primitives");

    group.bench_function("exp(2.0)", |b| {
        b.iter(|| eval(synth::exp(EmlExpr::v(0)), black_box(&[2.0])))
    });

    group.bench_function("ln(5.0)", |b| {
        b.iter(|| eval(synth::ln(EmlExpr::v(0)), black_box(&[5.0])))
    });

    group.bench_function("sqrt(16.0)", |b| {
        b.iter(|| eval(synth::sqrt(EmlExpr::v(0)), black_box(&[16.0])))
    });

    group.bench_function("pow(2.0, 10.0)", |b| {
        b.iter(|| eval(synth::pow(EmlExpr::v(0), EmlExpr::v(1)), black_box(&[2.0, 10.0])))
    });

    group.finish();
}

fn bench_deep_trees(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_trees");

    // Build expression tree: exp(exp(exp(exp(x)))) — depth 5
    fn deep_exp(depth: usize) -> EmlExpr {
        let mut expr = EmlExpr::v(0);
        for _ in 0..depth {
            expr = synth::exp(expr);
        }
        expr
    }

    for depth in [5, 10, 20] {
        group.bench_function(format!("exp^({depth})(x)"), |b| {
            let expr = deep_exp(depth);
            b.iter(|| eval(expr.clone(), black_box(&[1.0])))
        });
    }

    group.finish();
}

fn bench_compile(c: &mut Criterion) {
    c.bench_function("compile deep(20) tree", |b| {
        let mut expr = EmlExpr::v(0);
        for _ in 0..20 {
            expr = synth::exp(expr);
        }
        b.iter(|| expr.compile())
    });
}

fn bench_vs_f64(c: &mut Criterion) {
    let mut group = c.benchmark_group("vs_f64");
    let exp_prog = synth::exp(EmlExpr::v(0)).compile();
    let ln_prog = synth::ln(EmlExpr::v(0)).compile();
    let mul_prog = synth::mul(EmlExpr::v(0), EmlExpr::v(1)).compile();
    let x = ComplexBall::from_real(2.0);
    let y = ComplexBall::from_real(3.0);

    group.bench_function("eml_exp(2)", |b| {
        b.iter(|| execute(&exp_prog, black_box(&[x])).unwrap())
    });
    group.bench_function("f64_exp(2)", |b| {
        b.iter(|| black_box(black_box(2.0f64).exp()))
    });
    group.bench_function("eml_ln(5)", |b| {
        let five = ComplexBall::from_real(5.0);
        b.iter(|| execute(&ln_prog, black_box(&[five])).unwrap())
    });
    group.bench_function("f64_ln(5)", |b| {
        b.iter(|| black_box(black_box(5.0f64).ln()))
    });
    group.bench_function("eml_mul(2,3)", |b| {
        b.iter(|| execute(&mul_prog, black_box(&[x, y])).unwrap())
    });
    group.bench_function("f64_mul(2,3)", |b| {
        let a = black_box(2.0f64);
        let c = black_box(3.0f64);
        b.iter(|| black_box(a * c))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_eml_primitives,
    bench_deep_trees,
    bench_compile,
    bench_vs_f64
);
criterion_main!(benches);
