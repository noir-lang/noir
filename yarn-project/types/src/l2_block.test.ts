import { TxL2Logs } from './index.js';
import { L2Block } from './l2_block.js';

describe('L2Block', () => {
  it('can encode a L2 block data object to buffer and back', () => {
    const block = L2Block.random(42);

    const buffer = block.encode();
    const recovered = L2Block.decode(buffer);

    expect(recovered).toEqual(block);
  });

  it('can encode a L2 block to string and back', () => {
    const block = L2Block.random(42);
    const serialised = block.toString();
    const recovered = L2Block.fromString(serialised);

    expect(recovered).toEqual(block);
  });

  // TS equivalent of `testComputeKernelLogsIterationWithoutLogs` in `Decoder.t.sol`
  it('correctly computes kernel logs hash when there are no logs', () => {
    // The following 2 values are copied from `testComputeKernelLogsIterationWithoutLogs` in `Decoder.t.sol`
    const encodedLogs = Buffer.from('0000000400000000', 'hex');
    const logs = TxL2Logs.fromBuffer(encodedLogs, true);
    const referenceLogsHash = Buffer.from('1c9ecec90e28d2461650418635878a5c91e49f47586ecf75f2b0cbb94e897112', 'hex');

    const logsHash = L2Block.computeKernelLogsHash(logs);
    expect(logsHash).toEqual(referenceLogsHash);
  });

  // TS equivalent of `testComputeKernelLogs1Iteration` in `Decoder.t.sol`
  it('correctly computes kernel logs hash when are logs from 1 iteration', () => {
    // The following 2 values are copied from `testComputeKernelLogs1Iteration` in `Decoder.t.sol`
    const encodedLogs = Buffer.from('0000000c000000080000000493e78a70', 'hex');
    const logs = TxL2Logs.fromBuffer(encodedLogs, true);
    const referenceLogsHash = Buffer.from('1aa06a32df232f0d94b4735cffd46671c29dd1d4aec7cd562f856e643b4df833', 'hex');

    const logsHash = L2Block.computeKernelLogsHash(logs);
    expect(logsHash).toEqual(referenceLogsHash);
  });

  // TS equivalent of `testComputeKernelLogs2Iterations` in `Decoder.t.sol`
  it('correctly computes kernel logs hash when are logs from 2 iterations', () => {
    // The following 2 values are copied from `testComputeKernelLogs2Iterations` in `Decoder.t.sol`
    const encodedLogs = Buffer.from(
      '00000024000000080000000493e78a70000000140000001006a86173c86c6d3f108eefc36e7fb014',
      'hex',
    );
    const logs = TxL2Logs.fromBuffer(encodedLogs, true);
    const referenceLogsHash = Buffer.from('6030bd40b448d1075bfaaebf0a0c70407598df13d04c44e95454aab642fadcb2', 'hex');

    const logsHash = L2Block.computeKernelLogsHash(logs);
    expect(logsHash).toEqual(referenceLogsHash);
  });

  // TS equivalent of `testComputeKernelLogsMiddleIterationWithoutLogs` in `Decoder.t.sol`
  it('correctly computes kernel logs hash when are logs from 3 iterations (2nd iter. without logs)', () => {
    // The following 2 values are copied from `testComputeKernelLogsMiddleIterationWithoutLogs` in `Decoder.t.sol`
    const encodedLogs = Buffer.from(
      '00000028000000080000000493e78a7000000000000000140000001006a86173c86c6d3f108eefc36e7fb014',
      'hex',
    );
    const logs = TxL2Logs.fromBuffer(encodedLogs, true);
    const referenceLogsHash = Buffer.from('5e7f868e0f851f68a2c6f0b091512f99424fcedaabe02d4b087c0066112d72e8', 'hex');

    const logsHash = L2Block.computeKernelLogsHash(logs);
    expect(logsHash).toEqual(referenceLogsHash);
  });
});
