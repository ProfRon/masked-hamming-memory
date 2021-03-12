extern crate mhd_mem;
extern crate criterion;

// use mhd_mem::mhd_method::*;
use mhd_mem::mhd_optimizer::*;
use mhd_mem::mhd_optimizer::{ Solution, Problem, Solver };
use mhd_mem::implementations::*;

use std::time::Duration;

fn bench_optimization< Sol  : Solution,
                       Solv : Solver< Sol >,
                       Prob : Problem< Sol > >
                     ( time_limit : Duration,
                       problem    : & Prob,
                       solver     : & mut Solv  ) {

    assert!( solver.is_empty(),  "Will not bench solver which is not empty" );
    assert!( problem.is_legal(), "Will not bench problem which is illegal" );

    let the_best = find_best_solution( solver, problem, time_limit )
                                     .expect("could not find best solution on bench");

    assert!( problem.solution_is_legal( & the_best ));
    assert!( problem.solution_is_complete( & the_best ));

    // let best_score = the_best.get_score();
    // assert!( ZERO_SCORE < best_score );
    // assert_eq!( best_score, problem.solution_score( & the_best ));
    // assert_eq!( best_score, problem.solution_best_score( & the_best ));
}

// The following code is from
// https://bheisler.github.io/criterion.rs/book/user_guide/benchmarking_with_inputs.html
// or (later) from
// https://bheisler.github.io/criterion.rs/criterion/struct.BenchmarkGroup.html

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, BenchmarkGroup};
use criterion::measurement::WallTime;

fn bench_one_size(  group : & mut BenchmarkGroup<WallTime>,
                    size : usize ) {

    let time_limit= Duration::from_secs( size as u64 - 2 );
    group.measurement_time( time_limit ); // size in seconds

    // actually, we should take something of "big O" O(2^size),
    // but who has the patience?!?

    // First one problem, then another, since they are not mutable
    let problem_a = ProblemSubsetSum::random( size );
    let prob_name = format!("{} bits, Subset Sum", size );

    // ...with the Depth First Solver
    let mut solver_a = DepthFirstSolver::new( size );
    let bench_name = format!( "{}, Depth First", prob_name );

    assert!( problem_a.is_legal(), "illegal subset sum knapsack size {}", size );
    assert!( solver_a.is_empty(),  "illegal depth first solver size {}", size );

    group.bench_function ( BenchmarkId::new("Optimize", bench_name ),
                           |b| b.iter(
                               || bench_optimization( time_limit, & problem_a, & mut solver_a )
                           ));

    // ...and with the Best First Solver
    let mut solver_b = BestFirstSolver::new( size );
    let bench_name = format!( "{}, Best First", prob_name );

    assert!( problem_a.is_legal(), "illegal subset sum knapsack size {}", size );
    assert!( solver_b.is_empty(),  "illegal best first solver size {}", size );

    group.bench_function ( BenchmarkId::new("Optimize", bench_name),
                           |b| b.iter(
                               || bench_optimization( time_limit, & problem_a, & mut solver_b )
                           ));
    // First one problem, then another, since they are not mutable
    let problem_b = Problem01Knapsack::random( size );
    let prob_name = format!("{} bits, 01 Knapsack", size );

    // ...with the Depth First Solver
    let mut solver_c = DepthFirstSolver::new( size );
    let bench_name = format!( "{}/Depth First", prob_name );

    assert!( problem_b.is_legal(), "illegal subset 01 knapsack size {}", size );
    assert!( solver_c.is_empty(),  "illegal depth first solver size {}", size  );

    group.bench_function ( BenchmarkId::new("Optimize", bench_name ),
                           |b| b.iter(
                               || bench_optimization( time_limit, & problem_b, & mut solver_c )
                           ));

    // ...and with the Best First Solver
    let mut solver_d = BestFirstSolver::new( size );
    let bench_name = format!( "{}/Best First", prob_name );

    assert!( problem_b.is_legal(), "illegal subset 01 knapsack size {}", size );
    assert!( solver_d.is_empty(),  "illegal best first solver size {}", size  );

    group.bench_function ( BenchmarkId::new("Optimize", bench_name),
                           |b| b.iter(
                               || bench_optimization( time_limit, & problem_b, & mut solver_d )
                           ));


}

fn bench_sizes( c: &mut Criterion ) {

    let mut group = c.benchmark_group( "Optimization Benches" );

    for bits in [ 4, 5, 6, 8, 10, 12, 14, 16 ].iter() {
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
