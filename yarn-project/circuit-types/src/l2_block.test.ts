import { L2Block } from './l2_block.js';
import { EncryptedTxL2Logs } from './logs/index.js';

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
    const logs = EncryptedTxL2Logs.fromBuffer(encodedLogs, true);
    const referenceLogsHash = Buffer.alloc(32);

    const logsHash = logs.hash();
    expect(logsHash).toEqual(referenceLogsHash);
  });

  // TS equivalent of `testComputeKernelLogs1Iteration` in `Decoder.t.sol`
  it('correctly computes kernel logs hash when are logs from 1 iteration', () => {
    // The following 2 values are copied from `testComputeKernelLogs1Iteration` in `Decoder.t.sol`
    const encodedLogs = Buffer.from('0000000c000000080000000493e78a70', 'hex');
    const logs = EncryptedTxL2Logs.fromBuffer(encodedLogs, true);
    const referenceLogsHash = Buffer.from('0044339f3cafeb22de0d76423142797f1d4520c6cad559de5d1390bb7ab4c812', 'hex');

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
    const logs = EncryptedTxL2Logs.fromBuffer(encodedLogs, true);
    const referenceLogsHash = Buffer.from('00ebc16f83abc50c57496375353bf377b06bef23880bd3e9975ea1f7f5a0e8b1', 'hex');

    const logsHash = logs.hash();
    expect(logsHash).toEqual(referenceLogsHash);
  });

  // TS equivalent of `testComputeKernelLogsMiddleIterationWithoutLogs` in `Decoder.t.sol`
  it('correctly computes kernel logs hash when are logs from 3 iterations (2nd iter. without logs)', () => {
    // The following 2 values are copied from `testComputeKernelLogsMiddleIterationWithoutLogs` in `Decoder.t.sol`
    // Note: as of resolving #5017, we skip zero len logs, so we expect this and the prev hash to be the same
    const encodedLogs = Buffer.from(
      '00000028000000080000000493e78a7000000000000000140000001006a86173c86c6d3f108eefc36e7fb014',
      'hex',
    );
    const logs = EncryptedTxL2Logs.fromBuffer(encodedLogs, true);
    const referenceLogsHash = Buffer.from('00ebc16f83abc50c57496375353bf377b06bef23880bd3e9975ea1f7f5a0e8b1', 'hex');

    const logsHash = logs.hash();
    expect(logsHash).toEqual(referenceLogsHash);
  });
});
