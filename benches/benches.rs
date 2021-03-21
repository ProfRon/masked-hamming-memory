extern crate criterion;
extern crate mhd_mem;

// use mhd_mem::mhd_method::*;
use mhd_mem::implementations::*;
use mhd_mem::mhd_optimizer::{MinimalSolution, Problem, Solution, Solver};

extern crate log;
use log::*;
use std::time::Duration;

/********************************* Benchmark Utilities *********************************/

// inline because https://bheisler.github.io/criterion.rs/book/getting_started.html
#[inline]
fn bench_optimization<Solv: Solver<<Prob as Problem>::Sol>, Prob: Problem>(
    problem: &Prob,
    solver: &mut Solv,
) {
    solver.clear();

    let the_best = problem
        .find_best_solution(solver, Duration::from_secs_f32(1.0))
        .expect("could not find best solution on bench");

    let best_score = the_best.get_score();
    // assert!( ZERO_SCORE < best_score );
    assert_eq!(best_score, problem.solution_score(&the_best));
    assert_eq!(best_score, problem.solution_best_score(&the_best));
}

// The following code is from
// https://bheisler.github.io/criterion.rs/book/user_guide/benchmarking_with_inputs.html
// or (later) from
// https://bheisler.github.io/criterion.rs/criterion/struct.BenchmarkGroup.html

use criterion::measurement::WallTime;
use criterion::{
    criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion, SamplingMode,
};

fn bench_one_combo<Solv: Solver<<Prob as Problem>::Sol>, Prob: Problem>(
    group: &mut BenchmarkGroup<WallTime>,
    bench_id: &str,
    problem: &Prob,
    solver: &mut Solv,
) {
    assert!(
        problem.is_legal(),
        "illegal knapsack {}",
        problem.short_description()
    );

    let bench_name = format!(
        "{}+{}({} bits)",
        solver.name(),
        problem.name(),
        problem.problem_size()
    );

    group.bench_function(BenchmarkId::new(bench_id, bench_name), |b| {
        b.iter(|| bench_optimization(problem, solver))
    });
}

/********************************* Random Benchmarks *********************************/

fn bench_one_size(group: &mut BenchmarkGroup<WallTime>, size: usize) {
    // First one problem, then another, since they are not mutable
    let problem_a = ProblemSubsetSum::random(size);

    const BENCH_NAME: &str = "Random";
    // ...with the Depth First Solver
    let mut solver_a = DepthFirstSolver::<MinimalSolution>::new(size);
    bench_one_combo(group, BENCH_NAME, &problem_a, &mut solver_a);

    // ...and with the Best First Solver
    let mut solver_b = BestFirstSolver::<MinimalSolution>::new(size);
    bench_one_combo(group, BENCH_NAME, &problem_a, &mut solver_b);

    // First one problem, then another, since they are not mutable
    let problem_b = Problem01Knapsack::random(size);

    // ...with the Depth First Solver
    let mut solver_c = DepthFirstSolver::<ZeroOneKnapsackSolution>::new(size);
    bench_one_combo(group, BENCH_NAME, &problem_b, &mut solver_c);

    // ...and with the Best First Solver
    let mut solver_d = BestFirstSolver::<ZeroOneKnapsackSolution>::new(size);
    bench_one_combo(group, BENCH_NAME, &problem_b, &mut solver_d);
}

fn bench_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sized");

    group.sample_size(10); // smallest size allowed
    group.sampling_mode(SamplingMode::Flat); // "intended for long-running benchmarks"

    // group.measurement_time( Duration::from_secs_f32( 61.0 ) ); // 30 * 2 = 6ß
    // actually, we should take something of "big O" O(2^size),
    // but who has the patience?!?

    for bits in [4, 16, 64, 256].iter() {
        bench_one_size(&mut group, *bits);
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

/********************************* Filebased Benchmarks *********************************/

use std::io;
use std::path::*;

fn bench_a_file(group: &mut BenchmarkGroup<WallTime>, pathname: PathBuf) {
    let filename = pathname.to_str().expect("cannot convert path to string");
    let file = std::fs::File::open(filename).expect("Could not open file");
    let mut input = io::BufReader::new(file);

    const MAX_KNAPSACKS_PER_FILE: i32 = 8;
    let mut knapsack_num = 0;
    loop {
        // over the problems in this file (there can be many more than one)
        knapsack_num += 1;
        if MAX_KNAPSACKS_PER_FILE < knapsack_num {
            break;
        }

        match parse_dot_dat_stream(&mut input) {
            Err(_) => break,
            Ok(knapsack) => {
                let size = knapsack.problem_size();
                let id = format!("{}.{}", filename, knapsack_num);
                // Bench twice,
                // first with the Depth First Solver:
                let mut dfs_solver = DepthFirstSolver::<ZeroOneKnapsackSolution>::new(size);
                bench_one_combo(group, &id, &knapsack, &mut dfs_solver);

                // ...and with the Best First Solver
                let mut bfs_solver = BestFirstSolver::<ZeroOneKnapsackSolution>::new(size);
                bench_one_combo(group, &id, &knapsack, &mut bfs_solver);
            } // end on match OK( Knapsack )
        } // end match Result<knapsack>
    } // end loop until no more knapsacks in file
}

fn bench_directory(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory");

    group.sample_size(10); // minimal amount allowed by criterion

    group.sampling_mode(SamplingMode::Flat); // "intended for long-running benchmarks"
                                             // group.measurement_time( Duration::from_secs_f32( 61.0 ) ); // 30 * 2 = 6ß
                                             // actually, we should take something of "big O" O(2^size),
                                             // but who has the patience?!?
    const DIR_NAME: &str = "Data_Files";
    let path = Path::new(DIR_NAME);
    assert!(
        path.is_dir(),
        "Cannot bench directory because - not a directory!"
    );

    for entry_result in path.read_dir().expect("read_dir call failed") {
        match entry_result {
            Ok(dir_entry) => bench_a_file(&mut group, dir_entry.path()),
            Err(e) => warn!("Error {:?} in directory {}", e, DIR_NAME),
        };
    } // end for all entries in directory
    group.finish();
}

// criterion_group!(randomBenches, );
criterion_group!(benches, bench_sizes, bench_directory,);
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
