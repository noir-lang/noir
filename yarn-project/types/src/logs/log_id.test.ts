import { LogId } from './log_id.js';

describe('LogId', () => {
  let logId: LogId;
  beforeEach(() => {
    const blockNumber = Math.floor(Math.random() * 1000);
    const txIndex = Math.floor(Math.random() * 1000);
    const logIndex = Math.floor(Math.random() * 1000);
    logId = new LogId(blockNumber, txIndex, logIndex);
  });

  it('toBuffer and fromBuffer works', () => {
    const buffer = logId.toBuffer();
    const parsedLogId = LogId.fromBuffer(buffer);

    expect(parsedLogId).toEqual(logId);
  });

  it('toBuffer and fromBuffer works', () => {
    const buffer = logId.toBuffer();
    const parsedLogId = LogId.fromBuffer(buffer);

    expect(parsedLogId).toEqual(logId);
  });
});
