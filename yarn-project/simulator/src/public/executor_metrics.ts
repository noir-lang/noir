import {
  Attributes,
  type Histogram,
  Metrics,
  type TelemetryClient,
  type UpDownCounter,
  ValueType,
} from '@aztec/telemetry-client';

export class ExecutorMetrics {
  private fnCount: UpDownCounter;
  private fnDuration: Histogram;
  private bytecodeSize: Histogram;

  constructor(client: TelemetryClient, name = 'PublicExecutor') {
    const meter = client.getMeter(name);

    this.fnCount = meter.createUpDownCounter(Metrics.PUBLIC_EXECUTOR_SIMULATION_COUNT, {
      description: 'Number of functions executed',
    });

    this.fnDuration = meter.createHistogram(Metrics.PUBLIC_EXECUTOR_SIMULATION_DURATION, {
      description: 'How long it takes to execute a function',
      unit: 'ms',
      valueType: ValueType.INT,
    });

    this.bytecodeSize = meter.createHistogram(Metrics.PUBLIC_EXECUTION_SIMULATION_BYTECODE_SIZE, {
      description: 'Size of the function bytecode',
      unit: 'By',
      valueType: ValueType.INT,
    });
  }

  recordFunctionSimulation(bytecodeSize: number, durationMs: number) {
    this.fnCount.add(1, {
      [Attributes.OK]: true,
    });
    this.bytecodeSize.record(bytecodeSize);
    this.fnDuration.record(Math.ceil(durationMs));
  }

  recordFunctionSimulationFailure() {
    this.fnCount.add(1, {
      [Attributes.OK]: false,
    });
  }
}
