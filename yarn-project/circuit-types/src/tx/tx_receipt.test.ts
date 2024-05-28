import { TxHash } from './tx_hash.js';
import { TxReceipt, TxStatus } from './tx_receipt.js';

describe('TxReceipt', () => {
  it('serializes and deserializes from json', () => {
    const receipt = new TxReceipt(
      TxHash.random(),
      TxStatus.SUCCESS,
      'error',
      BigInt(1),
      Buffer.from('blockHash'),
      undefined,
    );

    expect(TxReceipt.fromJSON(receipt.toJSON())).toEqual(receipt);
  });

  it('serializes and deserializes from json with undefined fields', () => {
    const receipt = new TxReceipt(TxHash.random(), TxStatus.DROPPED, 'error', undefined, undefined, undefined);

    expect(TxReceipt.fromJSON(receipt.toJSON())).toEqual(receipt);
  });
});
