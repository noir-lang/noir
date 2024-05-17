import { TxEffect } from './tx_effect.js';

describe('TxEffect', () => {
  it('convert to and from buffer', () => {
    const txEffect = TxEffect.random();
    const buf = txEffect.toBuffer();
    expect(TxEffect.fromBuffer(buf)).toEqual(txEffect);
  });

  it('hash of empty tx effect matches snapshot', () => {
    const txEffectHash = TxEffect.empty().hash().toString('hex');
    // If you change this you have to change the hardcoded value in TxsDecoder.sol!
    expect(txEffectHash).toMatchInlineSnapshot(`"009f12fb98ebbf4e5deef4cf51ade63094a795b891880217958b226707c95f43"`);
  });
});
