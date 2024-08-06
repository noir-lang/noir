import { type DebugLogger } from '@aztec/foundation/log';

import {
  DiagConsoleLogger,
  DiagLogLevel,
  type Meter,
  type Tracer,
  type TracerProvider,
  diag,
} from '@opentelemetry/api';
import { OTLPMetricExporter } from '@opentelemetry/exporter-metrics-otlp-http';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http';
import { HostMetrics } from '@opentelemetry/host-metrics';
import { Resource } from '@opentelemetry/resources';
import { MeterProvider, PeriodicExportingMetricReader } from '@opentelemetry/sdk-metrics';
import { BatchSpanProcessor, NodeTracerProvider } from '@opentelemetry/sdk-trace-node';
import { SEMRESATTRS_SERVICE_NAME, SEMRESATTRS_SERVICE_VERSION } from '@opentelemetry/semantic-conventions';

import { NETWORK_ID } from './attributes.js';
import { type Gauge, type TelemetryClient } from './telemetry.js';

export class OpenTelemetryClient implements TelemetryClient {
  hostMetrics: HostMetrics | undefined;
  targetInfo: Gauge | undefined;

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

    // See these two links for more information on providing target information:
    // https://opentelemetry.io/docs/specs/otel/compatibility/prometheus_and_openmetrics/#resource-attributes
    // https://github.com/OpenObservability/OpenMetrics/blob/main/specification/OpenMetrics.md#supporting-target-metadata-in-both-push-based-and-pull-based-systems
    this.targetInfo = this.meterProvider.getMeter('target').createGauge('target_info', {
      description: 'Target information',
    });
    this.targetInfo.record(1, this.resource.attributes);

    this.hostMetrics.start();
  }

  public async stop() {
    await Promise.all([this.meterProvider.shutdown()]);
  }

  public static createAndStart(
    name: string,
    version: string,
    networkIdentifier: string,
    collectorBaseUrl: URL,
    log: DebugLogger,
  ): OpenTelemetryClient {
    const resource = new Resource({
      [SEMRESATTRS_SERVICE_NAME]: name,
      [SEMRESATTRS_SERVICE_VERSION]: version,
      [NETWORK_ID]: networkIdentifier,
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
