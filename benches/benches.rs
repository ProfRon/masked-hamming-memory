extern crate mhd_mem;
extern crate criterion;

// use mhd_mem::mhd_method::*;
use mhd_mem::mhd_optimizer::*;

use crate::

use std::time::Duration;

fn bench_depth_first( num_decisions : usize ) {
    assert!( 1 < num_decisions );
    assert!( num_decisions <= 20 );
    let time_limit = Duration::new( 2, 0); // 4 seconds

    let mut first_solver   = FirstDepthFirstSolver::new(num_decisions);
    assert!( first_solver.is_empty() );

    let mut knapsack = ProblemSubsetSum::random(num_decisions);
    assert!( knapsack.is_legal() );

    let the_best = find_best_solution( & mut first_solver, & mut knapsack, time_limit )
                        .expect("could not find best solution");

    assert!( the_best.get_score() <= knapsack.capacity );
}

// The following code is from
// https://bheisler.github.io/criterion.rs/book/user_guide/benchmarking_with_inputs.html
// or (later) from
// https://bheisler.github.io/criterion.rs/criterion/struct.BenchmarkGroup.html

use criterion::{ criterion_group, criterion_main, Criterion, BenchmarkId };

fn depth_first_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("SubsetSum_DepthFirst");

    for b in [2, 3, 4, 6, 8, 10, 14, 18 ].iter() {
        let bits : usize = *b;
        // group.throughput(Throughput::Elements(*size as u64));

        group.measurement_time( Duration::from_secs(bits as u64 ) ); // size in seconds
        // actually, we should take something of "big O" O(2^size),
        // but who has the patience?!?

        let parameter_string = format!("bit size {}", bits );
        group.bench_with_input(BenchmarkId::new("Optimize", parameter_string),
                               &bits,
                               |b, bs| {
                                    b.iter(|| bench_depth_first( *bs   ) );
        });
    }
    group.finish();
}

fn bench_best_first( num_decisions : usize ) {
    assert!( 1 < num_decisions );
    assert!( num_decisions <= 20 );

    let time_limit = Duration::new( 2, 0); // 4 seconds

    let mut best_solver   = BestFirstSolver::new(num_decisions);
    assert!( best_solver.is_empty() );

    let mut knapsack = ProblemSubsetSum::random(num_decisions);
    assert!( knapsack.is_legal() );

    let the_best = find_best_solution( & mut best_solver, & mut knapsack, time_limit )
        .expect("could not find best solution");

    assert!( the_best.get_score() <= knapsack.capacity );
}

fn best_first_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("SubsetSum_BestFirst");

    for b in [2, 3, 4, 6, 8, 10, 14, 18 ].iter() {
        let bits : usize = *b;
        // group.throughput(Throughput::Elements(*size as u64));

        group.measurement_time( Duration::from_secs(bits as u64 ) ); // size in seconds
        // actually, we should take something of "big O" O(2^size),
        // but who has the patience?!?

        let parameter_string = format!("bit size {}", bits );
        group.bench_with_input(BenchmarkId::new("Optimize", parameter_string),
                               &bits,
                               |b, bs| {
                                   b.iter(|| bench_best_first( *bs   ) );
                               });
    }
    group.finish();
}

criterion_group!(benches, depth_first_benches, best_first_benches );
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
