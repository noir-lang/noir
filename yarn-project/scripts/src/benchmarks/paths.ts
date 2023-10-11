import * as path from 'path';

/** Folder where to load raw logs from */
export const LogsDir = process.env.LOG_FOLDER ?? `log`;

/** Folder with the aggregated benchmark results */
export const BenchDir = process.env.BENCH_FOLDER ?? `bench`;

/** Benchmark file path */
export const BenchFile = path.join(BenchDir, 'benchmark.json');

/** Base benchmark file path */
export const BaseBenchFile = path.join(BenchDir, 'base-benchmark.json');
