import { type DebugLogger } from '@aztec/foundation/log';

import { type Meter, type Tracer, type TracerProvider } from '@opentelemetry/api';
import { DiagConsoleLogger, DiagLogLevel, diag } from '@opentelemetry/api';
import { OTLPMetricExporter } from '@opentelemetry/exporter-metrics-otlp-http';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http';
import { HostMetrics } from '@opentelemetry/host-metrics';
import { Resource } from '@opentelemetry/resources';
import { MeterProvider, PeriodicExportingMetricReader } from '@opentelemetry/sdk-metrics';
import { BatchSpanProcessor, NodeTracerProvider } from '@opentelemetry/sdk-trace-node';
import { SEMRESATTRS_SERVICE_NAME, SEMRESATTRS_SERVICE_VERSION } from '@opentelemetry/semantic-conventions';

import { type TelemetryClient } from './telemetry.js';

export class OpenTelemetryClient implements TelemetryClient {
  hostMetrics: HostMetrics | undefined;
  protected constructor(
    private resource: Resource,
    private meterProvider: MeterProvider,
    private traceProvider: TracerProvider,
    private log: DebugLogger,
  ) {}

  getMeter(name: string): Meter {
    return this.meterProvider.getMeter(name, this.resource.attributes[SEMRESATTRS_SERVICE_VERSION] as string);
  }

  getTracer(name: string): Tracer {
    return this.traceProvider.getTracer(name, this.resource.attributes[SEMRESATTRS_SERVICE_VERSION] as string);
  }

  public start() {
    this.log.info('Starting OpenTelemetry client');
    diag.setLogger(new DiagConsoleLogger(), DiagLogLevel.INFO);

    this.hostMetrics = new HostMetrics({
      name: this.resource.attributes[SEMRESATTRS_SERVICE_NAME] as string,
      meterProvider: this.meterProvider,
    });

    this.hostMetrics.start();
  }

  public async stop() {
    await Promise.all([this.meterProvider.shutdown()]);
  }

  public static createAndStart(
    name: string,
    version: string,
    collectorBaseUrl: URL,
    log: DebugLogger,
  ): OpenTelemetryClient {
    const resource = new Resource({
      [SEMRESATTRS_SERVICE_NAME]: name,
      [SEMRESATTRS_SERVICE_VERSION]: version,
    });

    const tracerProvider = new NodeTracerProvider({
      resource,
    });
    tracerProvider.addSpanProcessor(
      new BatchSpanProcessor(new OTLPTraceExporter({ url: new URL('/v1/traces', collectorBaseUrl).href })),
    );
    tracerProvider.register();

    const meterProvider = new MeterProvider({
      resource,
      readers: [
        new PeriodicExportingMetricReader({
          exporter: new OTLPMetricExporter({
            url: new URL('/v1/metrics', collectorBaseUrl).href,
          }),
        }),
      ],
    });

    const service = new OpenTelemetryClient(resource, meterProvider, tracerProvider, log);
    service.start();

    return service;
  }
}
