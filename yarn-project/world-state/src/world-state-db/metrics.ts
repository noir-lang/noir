import { type Timer } from '@aztec/foundation/timer';
import {
  Attributes,
  type Gauge,
  type Histogram,
  Metrics,
  type TelemetryClient,
  ValueType,
} from '@aztec/telemetry-client';

export class WorldStateMetrics {
  private treeSize: Gauge;
  private dbSize: Gauge;
  private forkDuration: Histogram;
  private syncDuration: Histogram;

  constructor(client: TelemetryClient, name = 'MerkleTreesDb') {
    const meter = client.getMeter(name);
    this.treeSize = meter.createGauge(Metrics.WORLD_STATE_MERKLE_TREE_SIZE, {
      description: 'The size of Merkle trees',
      valueType: ValueType.INT,
    });

    this.dbSize = meter.createGauge(Metrics.WORLD_STATE_DB_SIZE, {
      description: 'The size of the World State DB',
      valueType: ValueType.INT,
      unit: 'By',
    });

    this.forkDuration = meter.createHistogram(Metrics.WORLD_STATE_FORK_DURATION, {
      description: 'The duration of a fork operation',
      unit: 'ms',
      valueType: ValueType.INT,
    });

    this.syncDuration = meter.createHistogram(Metrics.WORLD_STATE_SYNC_DURATION, {
      description: 'The duration of a sync operation',
      unit: 'ms',
      valueType: ValueType.INT,
    });
  }

  recordTreeSize(treeName: string, treeSize: bigint) {
    this.treeSize.record(Number(treeSize), {
      [Attributes.MERKLE_TREE_NAME]: treeName,
    });
  }

  recordDbSize(dbSizeInBytes: number) {
    this.dbSize.record(dbSizeInBytes);
  }

  recordForkDuration(timerOrMs: Timer | number) {
    const ms = Math.ceil(typeof timerOrMs === 'number' ? timerOrMs : timerOrMs.ms());
    this.forkDuration.record(ms);
  }

  recordSyncDuration(syncType: 'commit' | 'rollback_and_update', timerOrMs: Timer | number) {
    const ms = Math.ceil(typeof timerOrMs === 'number' ? timerOrMs : timerOrMs.ms());
    this.syncDuration.record(ms, {
      [Attributes.STATUS]: syncType,
    });
  }
}
