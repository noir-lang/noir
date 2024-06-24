import { type Meter } from '@opentelemetry/api';
import { OTLPMetricExporter } from '@opentelemetry/exporter-metrics-otlp-http';
import { HostMetrics } from '@opentelemetry/host-metrics';
import { Resource } from '@opentelemetry/resources';
import { MeterProvider, PeriodicExportingMetricReader } from '@opentelemetry/sdk-metrics';
import { SEMRESATTRS_SERVICE_NAME, SEMRESATTRS_SERVICE_VERSION } from '@opentelemetry/semantic-conventions';

import { type TelemetryClient } from './telemetry.js';

export class OpenTelemetryClient implements TelemetryClient {
  hostMetrics: HostMetrics | undefined;
  protected constructor(private resource: Resource, private meterProvider: MeterProvider) {}

  getMeter(name: string): Meter {
    return this.meterProvider.getMeter(name, this.resource.attributes[SEMRESATTRS_SERVICE_VERSION] as string);
  }

  public start() {
    this.hostMetrics = new HostMetrics({
      name: this.resource.attributes[SEMRESATTRS_SERVICE_NAME] as string,
      meterProvider: this.meterProvider,
    });

    this.hostMetrics.start();
  }

  public async stop() {
    await Promise.all([this.meterProvider.shutdown()]);
  }

  public static createAndStart(name: string, version: string, collectorBaseUrl: URL): OpenTelemetryClient {
    const resource = new Resource({
      [SEMRESATTRS_SERVICE_NAME]: name,
      [SEMRESATTRS_SERVICE_VERSION]: version,
    });

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

    const service = new OpenTelemetryClient(resource, meterProvider);
    service.start();

    return service;
  }
}
