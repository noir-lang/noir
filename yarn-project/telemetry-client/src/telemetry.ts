import {
  type AttributeValue,
  type MetricOptions,
  type Gauge as OtelGauge,
  type Histogram as OtelHistogram,
  type UpDownCounter as OtelUpDownCounter,
} from '@opentelemetry/api';

import * as Attributes from './attributes.js';
import * as Metrics from './metrics.js';

export { ValueType } from '@opentelemetry/api';

type ValuesOf<T> = T extends Record<string, infer U> ? U : never;

/** Global registry of attributes */
type Attributes = Partial<Record<ValuesOf<typeof Attributes>, AttributeValue>>;
export { Attributes };

/** Global registry of metrics */
type Metrics = (typeof Metrics)[keyof typeof Metrics];
export { Metrics };

export type Gauge = OtelGauge<Attributes>;
export type Histogram = OtelHistogram<Attributes>;
export type UpDownCounter = OtelUpDownCounter<Attributes>;

// INTERNAL NOTE: this interface is the same as opentelemetry's Meter, but with proper types
/**
 * A meter that provides instruments for recording metrics.
 */
export interface Meter {
  /**
   * Creates a new gauge instrument. A gauge is a metric that represents a single numerical value that can arbitrarily go up and down.
   * @param name - The name of the gauge
   * @param options - The options for the gauge
   */
  createGauge(name: Metrics, options?: MetricOptions): Gauge;

  /**
   * Creates a new histogram instrument. A histogram is a metric that samples observations (usually things like request durations or response sizes) and counts them in configurable buckets.
   * @param name - The name of the histogram
   * @param options - The options for the histogram
   */
  createHistogram(name: Metrics, options?: MetricOptions): Histogram;

  /**
   * Creates a new counter instrument. A counter can go up or down with a delta from the previous value.
   * @param name - The name of the counter
   * @param options - The options for the counter
   */
  createUpDownCounter(name: Metrics, options?: MetricOptions): UpDownCounter;
}

/**
 * A telemetry client that provides meters for recording metrics.
 */
export interface TelemetryClient {
  /**
   * Creates a new meter
   * @param name - The name of the meter.
   */
  getMeter(name: string): Meter;

  /**
   * Stops the telemetry client.
   */
  stop(): Promise<void>;
}
