import { setupCustomSnapshotSerializers } from '@aztec/foundation/testing';

import { getSampleUnconstrainedFunctionBroadcastedEventPayload } from '../../tests/fixtures.js';
import { UnconstrainedFunctionBroadcastedEvent } from './unconstrained_function_broadcasted_event.js';

describe('UnconstrainedFunctionBroadcastedEvent', () => {
  beforeAll(() => setupCustomSnapshotSerializers(expect));
  it('parses an event as emitted by the ContractClassRegisterer', () => {
    const data = getSampleUnconstrainedFunctionBroadcastedEventPayload();
    const event = UnconstrainedFunctionBroadcastedEvent.fromLogData(data);
    expect(event).toMatchSnapshot();
  });
});
