extern crate mhd_mem;
extern crate criterion;

// use mhd_mem::mhd_method::*;
use mhd_mem::mhd_optimizer::*;
use mhd_mem::mhd_optimizer::{ Solution, Problem, Solver };
use mhd_mem::implementations::*;

use std::time::Duration;

// inline because https://bheisler.github.io/criterion.rs/book/getting_started.html
#[inline]
fn bench_optimization< Sol  : Solution,
                       Solv : Solver< Sol >,
                       Prob : Problem< Sol > >
                     ( problem    : & Prob,
                       solver     : & mut Solv  ) {

    solver.clear();

    let the_best = black_box(
                    find_best_solution( solver, problem,
                                               Duration::from_secs_f32( 2.5 ) )
                        .expect("could not find best solution on bench") );

    let best_score = the_best.get_score();
    // assert!( ZERO_SCORE < best_score );
    assert_eq!( best_score, problem.solution_score(      & the_best ));
    assert_eq!( best_score, problem.solution_best_score( & the_best ));
}

// The following code is from
// https://bheisler.github.io/criterion.rs/book/user_guide/benchmarking_with_inputs.html
// or (later) from
// https://bheisler.github.io/criterion.rs/criterion/struct.BenchmarkGroup.html

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, BenchmarkGroup, black_box};
use criterion::measurement::WallTime;

fn bench_one_combo< Sol  : Solution,
                    Solv : Solver< Sol >,
                    Prob : Problem< Sol > >(  group   : & mut BenchmarkGroup<WallTime>,
                                              problem : & Prob,
                                              solver  : & mut Solv,
                                              size    : usize ) {

    assert!( problem.is_legal(),   "illegal {} knapsack size {}", problem.name(), size );

    let prefix= format!( "Bench {} bits", size );
    let bench_name = format!( "{}+{}", problem.name(), solver.name() );

    group.bench_function ( BenchmarkId::new(prefix, bench_name ),
                           |b| b.iter(
                               || bench_optimization( problem, solver )
                           ));

}

fn bench_one_size(  group : & mut BenchmarkGroup<WallTime>,
                    size : usize ) {


    // First one problem, then another, since they are not mutable
    let problem_a = ProblemSubsetSum::random( size );

    // ...with the Depth First Solver
    let mut solver_a = DepthFirstSolver::new( size );
    bench_one_combo(  group, & problem_a, & mut solver_a, size );

    // ...and with the Best First Solver
    let mut solver_b = BestFirstSolver::new( size );
    bench_one_combo(  group, & problem_a, & mut solver_b, size );

    // First one problem, then another, since they are not mutable
    let problem_b = Problem01Knapsack::random( size );

    // ...with the Depth First Solver
    let mut solver_c = DepthFirstSolver::new( size );
    bench_one_combo(  group, & problem_b, & mut solver_c, size );

    // ...and with the Best First Solver
    let mut solver_d = BestFirstSolver::new( size );
    bench_one_combo(  group, & problem_b, & mut solver_d, size );


}

fn bench_sizes( c: &mut Criterion ) {

    let mut group = c.benchmark_group( "Optimization Benches" );

    group.sample_size( 32 ); // less than default 100
    // group.sampling_mode(SamplingMode::Flat); // "intended for long-running benchmarks"

    group.measurement_time( Duration::from_secs( 8 ) ); // size in seconds
    // actually, we should take something of "big O" O(2^size),
    // but who has the patience?!?

    for bits in [ 4, 8, 16, 32, 64, 128, 256, 1024 ].iter() {
        bench_one_size( & mut group, *bits );
        //let bits : usize = *b;
        // group.throughput(Throughput::Elements(*size as u64));
        // let parameter_string = "Optimize Series";
        // group.bench_with_input(BenchmarkId::new("Optimize", parameter_string),
        //                        &bits,
        //                        |b, bs| {
        //                            b.iter(|| bench_one_size(  & mut group, *bs   ) );
        //                        });
    }
    group.finish();
}

criterion_group!(benches, bench_sizes );
criterion_main!(benches);

/********************************** OBSOLETE OLD HAMMING BENCHES ****************************
use criterion::{Criterion, Bencher, ParameterizedBenchmark, PlotConfiguration, AxisScale};

const SIZES: [usize; 7] = [1, 10, 100, 1000, 10_000, 100_000, 1_000_000];

macro_rules! create_benchmarks {
    ($(
        fn $group_id: ident($input: expr) {
            $first_name: expr => $first_func: expr,
            $($rest_name: expr => $rest_func: expr,)*
        }
    )*) => {
        $(
            fn $group_id(c: &mut Criterion) {
                let input = $input;

                let plot_config =
                    PlotConfiguration::default()
                    .summary_scale(AxisScale::Logarithmic);
                let bench = ParameterizedBenchmark::new(
                    $first_name, $first_func, input.iter().cloned())
                    $( .with_function($rest_name, $rest_func) )*
                    .plot_config(plot_config);
                c.bench(stringify!($group_id), bench);
            }
        )*
    }
}

fn naive_weight(x: &[u8]) -> u64 {
    x.iter().fold(0, |a, b| a + b.count_ones() as u64)
}

fn weight_bench<F: 'static + FnMut(&[u8]) -> u64>(mut f: F) -> impl FnMut(&mut Bencher, &usize) {
    move |b, n| {
        let data = vec![0xFF; *n];
        b.iter(|| f(criterion::black_box(&data)))
    }
}

fn naive_distance(mask: &[u8], x: &[u8], y: &[u8]) -> u64 {
    assert_eq!(mask.len(), x.len());
    assert_eq!(x.len(), y.len());
    mask.iter()
        .zip( x.iter().zip(y) )
        .fold(0, |a, ( m, (b, c)) | a + (*m & (*b ^ *c)).count_ones() as u64)
}

fn distance_bench<F: 'static + FnMut(&[u8], &[u8], &[u8]) -> u64>(mut f: F) -> impl FnMut(&mut Bencher, &usize) {
    move |b, n| {
        let data = vec![0xFF; *n];
        b.iter(|| {
            let d0 = criterion::black_box(&data);
            let d1 = criterion::black_box(&data);
            let d2 = criterion::black_box(&data);
            f(d0, d1, d2)
        })
    }
}



create_benchmarks! {
    fn weight(SIZES) {
        "naive" => weight_bench(naive_weight),
        "weight" => weight_bench(mhd_mem::mhd_method::weight),
    }
    fn distance(SIZES) {
        "naive" => distance_bench(naive_distance),
        "distance" => distance_bench(mhd_mem::mhd_method::distance),
    }
}

criterion_group!(benches, weight, distance);
criterion_main!(benches);
******************************* END OBSOLETE OLD HAMMING BENCHES ****************************/
