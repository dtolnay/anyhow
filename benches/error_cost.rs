//! Benchmark: quantify the cost of backtrace capture in anyhow::Error
//! construction vs the _lite variants that skip backtrace.
//!
//! Run via `benches/run_bench.sh` which controls RUST_LIB_BACKTRACE across
//! three rounds (disabled / DWARF CFI / frame-pointer).

use anyhow::{anyhow, anyhow_lite, Context};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

// ---------------------------------------------------------------------------
// A minimal thiserror type representing "error-code" style errors.
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
#[error("demo error code={code}")]
struct DemoError {
    code: u32,
}

// ---------------------------------------------------------------------------
// Helper: create controlled stack depth with #[inline(never)]
// ---------------------------------------------------------------------------

#[inline(never)]
fn recurse_anyhow(depth: u32) -> anyhow::Error {
    if depth == 0 {
        anyhow!("bottom of stack")
    } else {
        recurse_anyhow(depth - 1)
    }
}

#[inline(never)]
fn recurse_anyhow_lite(depth: u32) -> anyhow::Error {
    if depth == 0 {
        anyhow_lite!("bottom of stack")
    } else {
        recurse_anyhow_lite(depth - 1)
    }
}

// ---------------------------------------------------------------------------
// Group A: Construction cost comparison at the same stack depth
// ---------------------------------------------------------------------------

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");

    // 1. thiserror enum construction (near-zero baseline)
    group.bench_function("thiserror_construct", |b| {
        b.iter(|| {
            let e: Result<(), DemoError> = Err(DemoError { code: 42 });
            black_box(e)
        });
    });

    // 2. anyhow_lite!("msg") — heap alloc only, no backtrace
    group.bench_function("anyhow_lite_msg", |b| {
        b.iter(|| {
            let e = anyhow_lite!("something went wrong");
            black_box(e)
        });
    });

    // 3. anyhow!("msg") — heap alloc + Backtrace::capture()
    group.bench_function("anyhow_msg", |b| {
        b.iter(|| {
            let e = anyhow!("something went wrong");
            black_box(e)
        });
    });

    // 4. anyhow::Error::new(thiserror_val)
    group.bench_function("anyhow_new_from_thiserror", |b| {
        b.iter(|| {
            let e = anyhow::Error::new(DemoError { code: 42 });
            black_box(e)
        });
    });

    // 5. anyhow::Error::new_lite(thiserror_val)
    group.bench_function("anyhow_new_lite_from_thiserror", |b| {
        b.iter(|| {
            let e = anyhow::Error::new_lite(DemoError { code: 42 });
            black_box(e)
        });
    });

    // 6. .context() on a Result<(), ThiserrorErr> — first-time conversion
    group.bench_function("context_on_thiserror", |b| {
        b.iter(|| {
            let r: Result<(), DemoError> = Err(DemoError { code: 42 });
            let e = r.context("adding context");
            black_box(e)
        });
    });

    // 7. .context_lite() on a Result<(), ThiserrorErr>
    group.bench_function("context_lite_on_thiserror", |b| {
        b.iter(|| {
            let r: Result<(), DemoError> = Err(DemoError { code: 42 });
            let e = r.context_lite("adding context");
            black_box(e)
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Group B: Stack depth impact on backtrace capture
// ---------------------------------------------------------------------------

fn bench_stack_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("stack_depth");

    for depth in [8, 32, 128] {
        group.bench_function(format!("anyhow_depth_{depth}"), |b| {
            b.iter(|| {
                let e = recurse_anyhow(depth);
                black_box(e)
            });
        });

        group.bench_function(format!("anyhow_lite_depth_{depth}"), |b| {
            b.iter(|| {
                let e = recurse_anyhow_lite(depth);
                black_box(e)
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------

criterion_group!(benches, bench_construction, bench_stack_depth);
criterion_main!(benches);
