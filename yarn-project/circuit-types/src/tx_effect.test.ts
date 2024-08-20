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
    expect(txEffectHash).toMatchInlineSnapshot(`"00f0aa51fc81f8242316fcf2cb3b28196241ed3fa26dd320a959bce6c529b270"`);
  });
});
