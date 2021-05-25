extern crate criterion;

// use mhd_mem::mhd_memory::*;
extern crate mhd_optimization;
use mhd_optimization::optimizer::*;
use mhd_optimization::implementations::*;

extern crate log;
use log::*;
use std::time::Duration;

/********************************* Benchmark Utilities *********************************/

// Is: Sol: Solution, Solv: Solver<Sol>, Prob: Problem<Sol = Sol>
// Was: Solv: Solver<<Prob as Problem>::Sol>, Prob: Problem

// inline because https://bheisler.github.io/criterion.rs/book/getting_started.html
#[inline]
fn bench_optimization<Sol: Solution, Solv: Solver<Sol>, Prob: Problem<Sol = Sol>>(
    problem: &Prob,
    solver: &mut Solv,
) {
    solver.clear();

    let the_best = solver
        .find_best_solution(problem, Duration::from_secs_f32(3.1415926))
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

    // ...and then with the MCTS Solver
    let mut solver_mc =
        MonteCarloTreeSolver::<MinimalSolution, ProblemSubsetSum>::builder(&problem_a);
    bench_one_combo(group, BENCH_NAME, &problem_a, &mut solver_mc);

    // ...and then with the MHD Solver
    let mut solver_mhd =
        MhdMonteCarloSolver::<MinimalSolution, ProblemSubsetSum>::builder(&problem_a);
    bench_one_combo(group, BENCH_NAME, &problem_a, &mut solver_mhd);

    // First one problem, then another, since they are not mutable
    let problem_b = Problem01Knapsack::random(size);

    // ...with the Depth First Solver
    let mut solver_c = DepthFirstSolver::<ZeroOneKnapsackSolution>::new(size);
    bench_one_combo(group, BENCH_NAME, &problem_b, &mut solver_c);

    // ...and with the Best First Solver
    let mut solver_d = BestFirstSolver::<ZeroOneKnapsackSolution>::new(size);
    bench_one_combo(group, BENCH_NAME, &problem_b, &mut solver_d);

    // ...and then with the MCTS Solver
    let mut solver_mcts =
        MonteCarloTreeSolver::<ZeroOneKnapsackSolution, Problem01Knapsack>::builder(&problem_b);
    bench_one_combo(group, BENCH_NAME, &problem_b, &mut solver_mcts);

    // ...and then with the MHD Solver
    let mut solver_mhd2 =
        MhdMonteCarloSolver::<ZeroOneKnapsackSolution, Problem01Knapsack>::builder(&problem_b);
    bench_one_combo(group, BENCH_NAME, &problem_b, &mut solver_mhd2);
}

fn bench_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sized");

    group.sample_size(10); // smallest size allowed
    group.sampling_mode(SamplingMode::Flat); // "intended for long-running benchmarks"

    // group.measurement_time( Duration::from_secs_f32( 61.0 ) ); // 30 * 2 = 6ß
    // actually, we should take something of "big O" O(2^size),
    // but who has the patience?!?

    for bits in [8, 16, 24, 32].iter() {
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

                // ... and with the MCTS Solver
                let mut mcts_solver = MonteCarloTreeSolver::<
                    ZeroOneKnapsackSolution,
                    Problem01Knapsack,
                >::builder(&knapsack);
                bench_one_combo(group, &id, &knapsack, &mut mcts_solver);
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
criterion_group!(
    benches,
    bench_sizes,
    bench_directory,
);
criterion_main!(benches);

