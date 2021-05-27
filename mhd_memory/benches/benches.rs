extern crate log; // for the trace! macro
use log::*;

extern crate criterion;
use criterion::*;
// use criterion::{ Criterion, criterion_group, criterion_main, SamplingMode, };
// use criterion::{ Bencher, PlotConfiguration, AxisScale};

/********************************* MHD Memory Benchmarks *********************************/
extern crate mhd_memory;
use mhd_memory::*;

fn bench_mhd_memory(width: usize, height: usize) {
    let mut mem = MhdMemory::new(width);

    mem.write_n_random_samples(height);

    trace!(
        "Benchmark Memory ({} width X {} rows) has scores min {} < avg {} < max {} < total {}",
        width,
        height,
        mem.min_score,
        mem.avg_score(),
        mem.max_score,
        mem.total_score,
    );

    const NUM_READS: usize = 32; // more than sample size 10 but not TOO many

    for _ in 0..NUM_READS {
        let query = Sample::new(width, ZERO_SCORE);
        let mask = Sample::new(width, ZERO_SCORE);
        let result = mem.masked_read(&mask.bytes, &query.bytes);
        assert!(result > ZERO_SCORE);
    } // end for NUM_READS reads
} // end bench_mhd_memory

fn bench_mhd_memory_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_sizes");
    group.sampling_mode(SamplingMode::Flat); // "intended for long-running benchmarks"
                                             // group.measurement_time( Duration::from_secs_f32( 61.0 ) );

    for width in [8, 128, 1024, 2048].iter() {
        for height in [8, 128, 1024, 2048].iter() {
            let bench_parm = format!("{}X{}", *width, *height);
            group.bench_function(BenchmarkId::new("MHD_Mem_Bench", bench_parm), |b| {
                b.iter(|| bench_mhd_memory(*width, *height))
            });
        } // end for loops
    } // end for widths

    group.finish();
} // end bench_mhd_memory_sizes

// criterion_group!(randomBenches, );
criterion_group!(benches, bench_mhd_memory_sizes,);
criterion_main!(benches);

/************* obsolete benchmarks ************
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
        "weight" => weight_bench(mhd_mem::mhd_memory::weight),
    }
    fn distance(SIZES) {
        "naive" => distance_bench(naive_distance),
        "distance" => distance_bench(mhd_mem::mhd_memory::distance),
    }
}

criterion_group!(benches, weight, distance);
criterion_main!(benches);

************* end obsolete benchmarks ************/
