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
import { awsEc2Detector, awsEcsDetector } from '@opentelemetry/resource-detector-aws';
import {
  type IResource,
  detectResourcesSync,
  envDetectorSync,
  osDetectorSync,
  processDetectorSync,
  serviceInstanceIdDetectorSync,
} from '@opentelemetry/resources';
import { MeterProvider, PeriodicExportingMetricReader } from '@opentelemetry/sdk-metrics';
import { BatchSpanProcessor, NodeTracerProvider } from '@opentelemetry/sdk-trace-node';
import { SEMRESATTRS_SERVICE_NAME, SEMRESATTRS_SERVICE_VERSION } from '@opentelemetry/semantic-conventions';

import { aztecDetector } from './aztec_resource_detector.js';
import { type Gauge, type TelemetryClient } from './telemetry.js';

export class OpenTelemetryClient implements TelemetryClient {
  hostMetrics: HostMetrics | undefined;
  targetInfo: Gauge | undefined;

  protected constructor(
    private resource: IResource,
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

    if (this.resource.asyncAttributesPending) {
      void this.resource.waitForAsyncAttributes!().then(() => {
        this.targetInfo!.record(1, this.resource.attributes);
      });
    } else {
      this.targetInfo.record(1, this.resource.attributes);
    }

    this.hostMetrics.start();
  }

  public async stop() {
    await Promise.all([this.meterProvider.shutdown()]);
  }

  public static createAndStart(collectorBaseUrl: URL, log: DebugLogger): OpenTelemetryClient {
    const resource = detectResourcesSync({
      detectors: [
        osDetectorSync,
        envDetectorSync,
        processDetectorSync,
        serviceInstanceIdDetectorSync,
        awsEc2Detector,
        awsEcsDetector,
        aztecDetector,
      ],
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
