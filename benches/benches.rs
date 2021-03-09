extern crate mhd_mem;
extern crate criterion;

// use mhd_mem::mhd_method::*;
use mhd_mem::mhd_optimizer::*;

fn bench_find_best_solution( num_decisions : usize ) {
    assert!( 1 < num_decisions );
    assert!( num_decisions <= 20 );

    use std::time::{Duration};
    let time_limit = Duration::new( 2, 0); // 4 seconds

    let mut first_solver   = FirstDepthFirstSolver::new(num_decisions);
    assert!( first_solver.is_empty() );

    let mut knapsack = ProblemSubsetSum::random(num_decisions);
    assert!( knapsack.is_legal() );

    let the_best = find_best_solution( & mut first_solver, & mut knapsack, time_limit );
    assert!( the_best.get_score() <= knapsack.capacity );
}

// The following code is from
// https://bheisler.github.io/criterion.rs/book/user_guide/benchmarking_with_inputs.html

use criterion::{ black_box, criterion_group, criterion_main, Criterion, Throughput, BenchmarkId };

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("find_best with", |b| b.iter(||
                           bench_find_best_solution( black_box( 6 ) as usize )
    ));
}

fn from_one_elem(c: &mut Criterion) {
    let size: usize = 6;

    c.bench_with_input(BenchmarkId::new("one_input_example 6", size), &size, |b, &s| {
        b.iter(|| bench_find_best_solution( black_box( s ) as usize ));
    });
}

fn from_many_elems(c: &mut Criterion) {

    let mut group = c.benchmark_group("SubsetSum_Sizes");
    for size in [ 8, 10, 12, 14, 16, 18 ].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| bench_find_best_solution( size as usize ) );
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark, from_one_elem, from_many_elems );
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
