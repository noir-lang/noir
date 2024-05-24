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
    // maskedAddress = '1100000000000000000000000000000000000000000000000000000000000000'
    const encodedLogs = Buffer.from(
      '0000002c0000002800000024110000000000000000000000000000000000000000000000000000000000000093e78a70',
      'hex',
    );
    const logs = EncryptedTxL2Logs.fromBuffer(encodedLogs, true);
    const referenceLogsHash = Buffer.from('00f7bf1d4b3b5c99b8e370989e306b0eb712ca30bba1ce18a651cef3994e6610', 'hex');

    const logsHash = logs.hash();
    expect(logsHash).toEqual(referenceLogsHash);
  });

  // TS equivalent of `testComputeKernelLogs2Iterations` in `Decoder.t.sol`
  it('correctly computes kernel logs hash when are logs from 2 iterations', () => {
    // The following 2 values are copied from `testComputeKernelLogs2Iterations` in `Decoder.t.sol`
    // maskedAddress1 = '1100000000000000000000000000000000000000000000000000000000000000'
    // maskedAddress2 = '1200000000000000000000000000000000000000000000000000000000000000'
    const encodedLogs = Buffer.from(
      '000000640000002800000024110000000000000000000000000000000000000000000000000000000000000093e78a700000003400000030120000000000000000000000000000000000000000000000000000000000000006a86173c86c6d3f108eefc36e7fb014',
      'hex',
    );
    const logs = EncryptedTxL2Logs.fromBuffer(encodedLogs, true);
    const referenceLogsHash = Buffer.from('0021b8f5c71dbf2f102772c132c59f9f27b55405a22340f9e021ce11164636a2', 'hex');

    const logsHash = logs.hash();
    expect(logsHash).toEqual(referenceLogsHash);
  });

  // TS equivalent of `testComputeKernelLogsMiddleIterationWithoutLogs` in `Decoder.t.sol`
  it('correctly computes kernel logs hash when are logs from 3 iterations (2nd iter. without logs)', () => {
    // The following 2 values are copied from `testComputeKernelLogsMiddleIterationWithoutLogs` in `Decoder.t.sol`
    // Note: as of resolving #5017, we skip zero len logs, so we expect this and the prev hash to be the same
    const encodedLogs = Buffer.from(
      '000000680000002800000024110000000000000000000000000000000000000000000000000000000000000093e78a70000000000000003400000030120000000000000000000000000000000000000000000000000000000000000006a86173c86c6d3f108eefc36e7fb014',
      'hex',
    );
    const logs = EncryptedTxL2Logs.fromBuffer(encodedLogs, true);

    const referenceLogsHash = Buffer.from('0021b8f5c71dbf2f102772c132c59f9f27b55405a22340f9e021ce11164636a2', 'hex');

    const logsHash = logs.hash();
    expect(logsHash).toEqual(referenceLogsHash);
  });
});
