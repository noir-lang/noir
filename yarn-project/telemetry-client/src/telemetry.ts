import {
  type AttributeValue,
  type MetricOptions,
  type Gauge as OtelGauge,
  type Histogram as OtelHistogram,
  type UpDownCounter as OtelUpDownCounter,
  type Span,
  SpanStatusCode,
  Tracer,
} from '@opentelemetry/api';

import * as Attributes from './attributes.js';
import * as Metrics from './metrics.js';

export { ValueType, Span } from '@opentelemetry/api';

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

export { Tracer };

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
   * Creates a new tracer
   * @param name - The name of the tracer.
   */
  getTracer(name: string): Tracer;

  /**
   * Stops the telemetry client.
   */
  stop(): Promise<void>;
}

/** Objects that adhere to this interface can use @trackSpan */
export interface Traceable {
  tracer: Tracer;
}

type SpanDecorator<T extends Traceable, F extends (...args: any[]) => any> = (
  originalMethod: F,
  context: ClassMethodDecoratorContext<T>,
) => F;

/**
 * Starts a new span whenever the decorated method is called.
 * @param spanName - The name of the span to create. Can be a string or a function that returns a string.
 * @param attributes - Initial attributes to set on the span. If a function is provided, it will be called with the arguments of the method.
 * @param extraAttributes - Extra attributes to set on the span after the method is called. Will be called with the return value of the method. Note: if the function throws then this will not be called.
 * @returns A decorator that wraps the method in a span.
 *
 * @privateRemarks
 * This code looks complex but it's not that difficult:
 * - decorators are functions that _replace_ a method with a different implementation
 * - normal decorators can't take function arguments, but if we write a function that returns a decorator, we can pass arguments to that function
 *
 * The trackSpan function takes a span's name and some attributes and builds a decorator that wraps a method in a span with the given name and props
 * The decorator can currently only be applied to methods on classes that have a `tracer` property. The compiler will enforce this.
 */
export function trackSpan<T extends Traceable, F extends (...args: any[]) => any>(
  spanName: string | ((this: T, ...args: Parameters<F>) => string),
  attributes?: Attributes | ((this: T, ...args: Parameters<F>) => Attributes),
  extraAttributes?: (this: T, returnValue: Awaited<ReturnType<F>>) => Attributes,
): SpanDecorator<T, F> {
  // the return value of trackSpan is a decorator
  return (originalMethod: F, _context: ClassMethodDecoratorContext<T>) => {
    // the return value of the decorator replaces the original method
    // in this wrapper method we start a span, call the original method, and then end the span
    return function replacementMethod(this: T, ...args: Parameters<F>): Promise<Awaited<ReturnType<F>>> {
      const name = typeof spanName === 'function' ? spanName.call(this, ...args) : spanName;
      const currentAttrs = typeof attributes === 'function' ? attributes.call(this, ...args) : attributes;

      // run originalMethod wrapped in an active span
      // "active" means the span will be alive for the duration of the function execution
      // and if any other spans are started during the execution of originalMethod, they will be children of this span
      // behind the scenes this uses AsyncLocalStorage https://nodejs.org/dist/latest-v18.x/docs/api/async_context.html
      return this.tracer.startActiveSpan(name, async (span: Span) => {
        span.setAttributes(currentAttrs ?? {});

        try {
          const res = await originalMethod.call(this, ...args);
          const extraAttrs = extraAttributes?.call(this, res);
          span.setAttributes(extraAttrs ?? {});
          return res;
        } catch (err) {
          span.setStatus({
            code: SpanStatusCode.ERROR,
            message: String(err),
          });
          throw err;
        } finally {
          span.end();
        }
      });
    } as F;
  };
}

/**
 * Runs an event callback in a span. The span is started immediately and completes once the callback finishes running.
 * The span will have two events added: 'callbackStart' and 'callbackEnd' to mark the start and end of the callback.
 *
 * @param tracer - The tracer instance to use
 * @param spanName - The name of the span to create
 * @param attributes - Initial attributes to set on the span
 * @param callback - The callback to wrap in a span
 *
 * @returns - A new function that wraps the callback in a span
 */
export function wrapCallbackInSpan<F extends (...args: any[]) => any>(
  tracer: Tracer,
  spanName: string,
  attributes: Attributes,
  callback: F,
): F {
  const span = tracer.startSpan(spanName, { attributes });
  return (async (...args: Parameters<F>) => {
    try {
      span.addEvent('callbackStart');
      const res = await callback(...args);
      return res;
    } catch (err) {
      span.setStatus({
        code: SpanStatusCode.ERROR,
        message: String(err),
      });
      throw err;
    } finally {
      span.addEvent('callbackEnd');
      span.end();
    }
  }) as F;
}
