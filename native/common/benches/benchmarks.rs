use common::world::*;
use criterion::{criterion_group, criterion_main, Criterion};
use tempfile::tempfile;

fn iterate_fresh_file(c: &mut Criterion) {
    c.bench_function("iterate_fresh_file", |b| {
        b.iter(|| {
            let mut index = TerrainDiskStorage::initialize(tempfile().unwrap(), tempfile().unwrap()).unwrap();
            index.get_chunks_in_range((-50, -50, -50), (50, 50, 50), |_chunk| Ok(())).unwrap();
        })
    });
}

fn iterate_pregen_file(c: &mut Criterion) {
    let mut index = TerrainDiskStorage::initialize(tempfile().unwrap(), tempfile().unwrap()).unwrap();
    index.get_chunks_in_range((-50, -50, -50), (50, 50, 50), |_chunk| Ok(())).unwrap();

    let profiler = pprof::ProfilerGuard::new(100).unwrap();

    c.bench_function("iterate_pregen_file", |b| {
        b.iter(|| {
            index.get_chunks_in_range((-50, -50, -50), (50, 50, 50), |_chunk| Ok(())).unwrap();
        })
    });

    if let Ok(report) = profiler.report().build() {
        let file = std::fs::File::create("flamegraphs/iterate_pregen_file.svg").unwrap();
        report.flamegraph(file).unwrap();
    };
}

fn single_chunk_fresh_file(c: &mut Criterion) {
    let mut index = TerrainDiskStorage::initialize(tempfile().unwrap(), tempfile().unwrap()).unwrap();

    c.bench_function("single_chunk_fresh_file", |b| {
        b.iter(|| {
            index.get_chunk(0, 0, 0, |_chunk| Ok(())).unwrap();
        })
    });
}

criterion_group!(terrain_io, iterate_fresh_file, iterate_pregen_file, single_chunk_fresh_file);
criterion_main!(terrain_io);
