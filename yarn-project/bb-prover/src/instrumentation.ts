import { type CircuitName } from '@aztec/circuit-types/stats';
import { type Timer } from '@aztec/foundation/timer';
import {
  Attributes,
  type Gauge,
  type Histogram,
  Metrics,
  type TelemetryClient,
  type Tracer,
  ValueType,
  millisecondBuckets,
} from '@aztec/telemetry-client';

/**
 * Instrumentation class for Prover implementations.
 */
export class ProverInstrumentation {
  private simulationDuration: Histogram;
  private witGenDuration: Histogram;
  private provingDuration: Histogram;

  private witGenInputSize: Gauge;
  private witGenOutputSize: Gauge;

  private proofSize: Gauge;
  private circuitSize: Gauge;
  private circuitPublicInputCount: Gauge;

  public readonly tracer: Tracer;

  constructor(telemetry: TelemetryClient, name: string) {
    this.tracer = telemetry.getTracer(name);
    const meter = telemetry.getMeter(name);

    this.simulationDuration = meter.createHistogram(Metrics.CIRCUIT_SIMULATION_DURATION, {
      description: 'Records how long it takes to simulate a circuit',
      unit: 'ms',
      valueType: ValueType.INT,
      advice: {
        explicitBucketBoundaries: millisecondBuckets(1), // 10ms -> ~327s
      },
    });

    this.witGenDuration = meter.createHistogram(Metrics.CIRCUIT_WITNESS_GEN_DURATION, {
      description: 'Records how long it takes to generate the partial witness for a circuit',
      unit: 'ms',
      valueType: ValueType.INT,
      advice: {
        explicitBucketBoundaries: millisecondBuckets(1),
      },
    });

    this.provingDuration = meter.createHistogram(Metrics.CIRCUIT_PROVING_DURATION, {
      unit: 'ms',
      description: 'Records how long it takes to prove a circuit',
      valueType: ValueType.INT,
      advice: {
        explicitBucketBoundaries: millisecondBuckets(2), // 100ms -> 54 minutes
      },
    });

    this.witGenInputSize = meter.createGauge(Metrics.CIRCUIT_WITNESS_GEN_INPUT_SIZE, {
      unit: 'By',
      description: 'Records the size of the input to the witness generation',
      valueType: ValueType.INT,
    });

    this.witGenOutputSize = meter.createGauge(Metrics.CIRCUIT_WITNESS_GEN_OUTPUT_SIZE, {
      unit: 'By',
      description: 'Records the size of the output of the witness generation',
      valueType: ValueType.INT,
    });

    this.proofSize = meter.createGauge(Metrics.CIRCUIT_PROVING_PROOF_SIZE, {
      unit: 'By',
      description: 'Records the size of the proof generated for a circuit',
      valueType: ValueType.INT,
    });

    this.circuitPublicInputCount = meter.createGauge(Metrics.CIRCUIT_PUBLIC_INPUTS_COUNT, {
      description: 'Records the number of public inputs in a circuit',
      valueType: ValueType.INT,
    });

    this.circuitSize = meter.createGauge(Metrics.CIRCUIT_SIZE, {
      description: 'Records the size of the circuit in gates',
      valueType: ValueType.INT,
    });
  }

  /**
   * Records the duration of a circuit operation.
   * @param metric - The metric to record
   * @param circuitName - The name of the circuit
   * @param timerOrMS - The duration
   */
  recordDuration(
    metric: 'simulationDuration' | 'witGenDuration' | 'provingDuration',
    circuitName: CircuitName | 'tubeCircuit',
    timerOrMS: Timer | number,
  ) {
    const ms = typeof timerOrMS === 'number' ? timerOrMS : timerOrMS.ms();
    this[metric].record(Math.ceil(ms), {
      [Attributes.PROTOCOL_CIRCUIT_NAME]: circuitName,
      [Attributes.PROTOCOL_CIRCUIT_TYPE]: 'server',
    });
  }

  /**
   * Records the duration of an AVM circuit operation.
   * @param metric - The metric to record
   * @param appCircuitName - The name of the function circuit (should be a `contract:function` string)
   * @param timerOrMS - The duration
   */
  recordAvmDuration(metric: 'witGenDuration' | 'provingDuration', appCircuitName: string, timerOrMS: Timer | number) {
    const ms = typeof timerOrMS === 'number' ? timerOrMS : timerOrMS.s();
    this[metric].record(Math.ceil(ms), {
      [Attributes.APP_CIRCUIT_NAME]: appCircuitName,
    });
  }

  /**
   * Records the size of a circuit operation.
   * @param metric - Records the size of a circuit operation.
   * @param circuitName - The name of the circuit
   * @param size - The size
   */
  recordSize(
    metric: 'witGenInputSize' | 'witGenOutputSize' | 'proofSize' | 'circuitSize' | 'circuitPublicInputCount',
    circuitName: CircuitName | 'tubeCircuit',
    size: number,
  ) {
    this[metric].record(Math.ceil(size), {
      [Attributes.PROTOCOL_CIRCUIT_NAME]: circuitName,
      [Attributes.PROTOCOL_CIRCUIT_TYPE]: 'server',
    });
  }

  /**
   * Records the size of an AVM circuit operation.
   * @param metric - The metric to record
   * @param appCircuitName - The name of the function circuit (should be a `contract:function` string)
   * @param size - The size
   */
  recordAvmSize(
    metric: 'witGenInputSize' | 'witGenOutputSize' | 'proofSize' | 'circuitSize' | 'circuitPublicInputCount',
    appCircuitName: string,
    size: number,
  ) {
    this[metric].record(Math.ceil(size), {
      [Attributes.APP_CIRCUIT_NAME]: appCircuitName,
    });
  }
}
