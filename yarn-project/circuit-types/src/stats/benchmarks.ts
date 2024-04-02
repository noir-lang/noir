import { type MetricName } from './metrics.js';

/** Aggregated benchmark results. */
export type BenchmarkResults = Partial<Record<MetricName, BenchmarkMetricResults>>;

/** Aggregated benchmark result for a given metric (values aggregated by bucket such as chain size). */
export type BenchmarkMetricResults = Record<string, number>;

/** Aggregated benchmark results with a timestamp. */
export type BenchmarkResultsWithTimestamp = BenchmarkResults & {
  /** When did this benchmark happen. */ timestamp: string;
};
