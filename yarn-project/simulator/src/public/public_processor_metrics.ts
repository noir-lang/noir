import { type PublicKernelType } from '@aztec/circuit-types';
import { type ContractClassRegisteredEvent } from '@aztec/circuits.js';
import {
  Attributes,
  type Histogram,
  Metrics,
  type TelemetryClient,
  type Tracer,
  type UpDownCounter,
  ValueType,
} from '@aztec/telemetry-client';

export class PublicProcessorMetrics {
  public readonly tracer: Tracer;

  private txDuration: Histogram;
  private txCount: UpDownCounter;
  private txPhaseCount: UpDownCounter;

  private phaseDuration: Histogram;
  private phaseCount: UpDownCounter;

  private bytecodeDeployed: Histogram;

  constructor(client: TelemetryClient, name = 'PublicProcessor') {
    this.tracer = client.getTracer(name);
    const meter = client.getMeter(name);

    this.txDuration = meter.createHistogram(Metrics.PUBLIC_PROCESSOR_TX_DURATION, {
      description: 'How long it takes to process a transaction',
      unit: 'ms',
      valueType: ValueType.INT,
    });

    this.txCount = meter.createUpDownCounter(Metrics.PUBLIC_PROCESSOR_TX_COUNT, {
      description: 'Number of transactions processed',
    });

    this.txPhaseCount = meter.createUpDownCounter(Metrics.PUBLIC_PROCESSOR_TX_PHASE_COUNT, {
      description: 'Number of phases processed',
    });

    this.phaseDuration = meter.createHistogram(Metrics.PUBLIC_PROCESSOR_PHASE_DURATION, {
      description: 'How long it takes to process a phase',
      unit: 'ms',
      valueType: ValueType.INT,
    });

    this.phaseCount = meter.createUpDownCounter(Metrics.PUBLIC_PROCESSOR_PHASE_COUNT, {
      description: 'Number of failed phases',
    });

    this.bytecodeDeployed = meter.createHistogram(Metrics.PUBLIC_PROCESSOR_DEPLOY_BYTECODE_SIZE, {
      description: 'Size of deployed bytecode',
      unit: 'By',
    });
  }

  recordPhaseDuration(phaseName: PublicKernelType, durationMs: number) {
    this.phaseCount.add(1, { [Attributes.TX_PHASE_NAME]: phaseName, [Attributes.OK]: true });
    this.phaseDuration.record(Math.ceil(durationMs), { [Attributes.TX_PHASE_NAME]: phaseName });
  }

  recordTx(phaseCount: number, durationMs: number) {
    this.txPhaseCount.add(phaseCount);
    this.txDuration.record(Math.ceil(durationMs));
    this.txCount.add(1, {
      [Attributes.OK]: true,
    });
  }

  recordFailedTx() {
    this.txCount.add(1, {
      [Attributes.OK]: false,
    });
  }

  recordRevertedPhase(phaseName: PublicKernelType) {
    this.phaseCount.add(1, { [Attributes.TX_PHASE_NAME]: phaseName, [Attributes.OK]: false });
  }

  recordClassRegistration(...events: ContractClassRegisteredEvent[]) {
    let totalBytecode = 0;
    for (const event of events) {
      totalBytecode += event.packedPublicBytecode.length;
    }

    this.bytecodeDeployed.record(totalBytecode);
  }
}
