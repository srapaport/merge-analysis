use criterion::{black_box, criterion_group, criterion_main, Criterion};
use merge_analysis::env;

fn bench_merge_analysis(c: &mut Criterion) {
    let opts = env::Options {
        graph: String::from(env::GRAPH_NAME_TEASER),
        results: format!("results/merge_res_bench_test.csv"),
        amount_merge: Some(env::AMOUNT_MERGE_TEASER),
    };

    let mut group = c.benchmark_group("merge_analysis");
    group.sample_size(10);  // Set sample size to 10
    group.measurement_time(std::time::Duration::from_secs(300)); // Optional: increase measurement time
    
    group.bench_function("merge_analysis_multi_thread", |b| {
        b.iter(|| {
            merge_analysis::merge_analysis_call(black_box(&opts));
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_merge_analysis);
criterion_main!(benches);