import { AztecRPC, TxHash, TxReceipt, TxStatus } from '@aztec/types';

import { MockProxy, mock } from 'jest-mock-extended';

import { SentTx } from './sent_tx.js';

describe('SentTx', () => {
  let rpc: MockProxy<AztecRPC>;
  let txHashPromise: Promise<TxHash>;

  let sentTx: SentTx;

  beforeEach(() => {
    rpc = mock();
    txHashPromise = Promise.resolve(TxHash.fromBigInt(1n));
    sentTx = new SentTx(rpc, txHashPromise);
  });

  describe('wait', () => {
    let txReceipt: TxReceipt;
    beforeEach(() => {
      txReceipt = { status: TxStatus.MINED, blockNumber: 20 } as TxReceipt;
      rpc.getTxReceipt.mockResolvedValue(txReceipt);
    });

    it('waits for all notes accounts to be synced', async () => {
      rpc.getSyncStatus
        .mockResolvedValueOnce({ blocks: 25, notes: { '0x1': 19, '0x2': 20 } })
        .mockResolvedValueOnce({ blocks: 25, notes: { '0x1': 20, '0x2': 20 } });

      const actual = await sentTx.wait({ timeout: 1, interval: 0.4 });
      expect(actual).toEqual(txReceipt);
    });

    it('fails if an account is not synced', async () => {
      rpc.getSyncStatus.mockResolvedValue({ blocks: 25, notes: { '0x1': 19, '0x2': 20 } });
      await expect(sentTx.wait({ timeout: 1, interval: 0.4 })).rejects.toThrowError(/timeout/i);
    });

    it('does not wait for notes sync', async () => {
      rpc.getSyncStatus.mockResolvedValue({ blocks: 19, notes: { '0x1': 19, '0x2': 19 } });
      const actual = await sentTx.wait({ timeout: 1, interval: 0.4, waitForNotesSync: false });
      expect(actual).toEqual(txReceipt);
    });

    it('throws if tx is dropped', async () => {
      rpc.getTxReceipt.mockResolvedValue({ ...txReceipt, status: TxStatus.DROPPED } as TxReceipt);
      rpc.getSyncStatus.mockResolvedValue({ blocks: 19, notes: { '0x1': 19, '0x2': 19 } });
      await expect(sentTx.wait({ timeout: 1, interval: 0.4 })).rejects.toThrowError(/dropped/);
    });
  });
});
