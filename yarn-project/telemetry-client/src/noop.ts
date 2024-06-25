import { type Meter, type Span, type SpanContext, type Tracer, createNoopMeter } from '@opentelemetry/api';

import { type TelemetryClient } from './telemetry.js';

export class NoopTelemetryClient implements TelemetryClient {
  getMeter(): Meter {
    return createNoopMeter();
  }

  getTracer(): Tracer {
    return new NoopTracer();
  }

  stop(): Promise<void> {
    return Promise.resolve();
  }
}

// @opentelemetry/api internally uses NoopTracer and NoopSpan but they're not exported
// make our own versions
// https://github.com/open-telemetry/opentelemetry-js/issues/4518#issuecomment-2179405444
class NoopTracer implements Tracer {
  startSpan(): Span {
    return new NoopSpan();
  }

  startActiveSpan<F extends (...args: any[]) => any>(_name: string, ...args: (unknown | F)[]): ReturnType<F> {
    // there are three different signatures for startActiveSpan, grab the function, we don't care about the rest
    const fn = args.find(arg => typeof arg === 'function') as F;
    return fn(new NoopSpan());
  }
}

class NoopSpan implements Span {
  private recording: boolean = true;
  addEvent(): this {
    return this;
  }

  addLink(): this {
    return this;
  }

  addLinks(): this {
    return this;
  }

  end(): void {
    this.recording = false;
  }

  isRecording(): boolean {
    return this.recording;
  }

  recordException(): void {
    return;
  }

  setAttribute(): this {
    return this;
  }

  setAttributes(): this {
    return this;
  }

  setStatus(): this {
    return this;
  }

  spanContext(): SpanContext {
    return {
      spanId: '',
      traceId: '',
      traceFlags: 0,
    };
  }

  updateName(): this {
    return this;
  }
}
