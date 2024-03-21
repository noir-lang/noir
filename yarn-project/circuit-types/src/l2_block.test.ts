import { L2Block } from './l2_block.js';
import { TxL2Logs } from './logs/index.js';

describe('L2Block', () => {
  it('can serialize an L2 block with logs to a buffer and back', () => {
    const block = L2Block.random(42);

    const buffer = block.toBuffer();
    const recovered = L2Block.fromBuffer(buffer);

    expect(recovered).toEqual(block);
  });

  // TS equivalent of `testComputeKernelLogsIterationWithoutLogs` in `Decoder.t.sol`
  it('correctly computes kernel logs hash when there are no logs', () => {
    // The following 2 values are copied from `testComputeKernelLogsIterationWithoutLogs` in `Decoder.t.sol`
    const encodedLogs = Buffer.from('0000000400000000', 'hex');
    const logs = TxL2Logs.fromBuffer(encodedLogs, true);
    const referenceLogsHash = Buffer.from('006003947a07e21c81ce2062539d6d6864fe999b58b03fc46f6c190d9eac9b39', 'hex');

    const logsHash = logs.hash();
    expect(logsHash).toEqual(referenceLogsHash);
  });

  // TS equivalent of `testComputeKernelLogs1Iteration` in `Decoder.t.sol`
  it('correctly computes kernel logs hash when are logs from 1 iteration', () => {
    // The following 2 values are copied from `testComputeKernelLogs1Iteration` in `Decoder.t.sol`
    const encodedLogs = Buffer.from('0000000c000000080000000493e78a70', 'hex');
    const logs = TxL2Logs.fromBuffer(encodedLogs, true);
    const referenceLogsHash = Buffer.from('00f458589e520e9e9bdaf746a7d226c39124e4a438f21fd41e6117a90f25f9a6', 'hex');

    const logsHash = logs.hash();
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
    const referenceLogsHash = Buffer.from('0084c3495a8cc56372f8f1d1efc0512920dae0f134d679cf26a12aff1509de14', 'hex');

    const logsHash = logs.hash();
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
    const referenceLogsHash = Buffer.from('00fb7a99b84aad205b5a8368c12a5a6b2dc19e5d623a601717b337cdadb56aa4', 'hex');

    const logsHash = logs.hash();
    expect(logsHash).toEqual(referenceLogsHash);
  });
});
