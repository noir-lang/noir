import { EthAddress } from '../eth_address/index.js';
import { hashMessage } from './hash_message.js';
import { recoverFromSigBuffer, recoverFromSignature, recoverFromVRS, sign } from './sign.js';

const tests = [
  {
    address: EthAddress.fromString('0xEB014f8c8B418Db6b45774c326A0E64C78914dC0'),
    privateKey: Buffer.from('be6383dad004f233317e46ddb46ad31b16064d14447a95cc1d8c8d4bc61c3728', 'hex'),
    data: hashMessage(Buffer.from('Some data')),
    signature: Buffer.from(
      'a8037a6116c176a25e6fc224947fde9e79a2deaa0dd8b67b366fbdfdbffc01f953e41351267b20d4a89ebfe9c8f03c04de9b345add4a52f15bd026b63c8fb1501b',
      'hex',
    ),
  },
  {
    address: EthAddress.fromString('0xEB014f8c8B418Db6b45774c326A0E64C78914dC0'),
    privateKey: Buffer.from('be6383dad004f233317e46ddb46ad31b16064d14447a95cc1d8c8d4bc61c3728', 'hex'),
    data: hashMessage(Buffer.from('Some data!%$$%&@*')),
    signature: Buffer.from(
      '05252412b097c5d080c994d1ea12abcee6f1cae23feb225517a0b691a66e12866b3f54292f9cfef98f390670b4d010fc4af7fcd46e41d72870602c117b14921c1c',
      'hex',
    ),
  },
];

describe('eth_account sign', () => {
  tests.forEach(test => {
    it('sign data', () => {
      const data = sign(test.data, test.privateKey, 27);
      expect(data.toBuffer()).toEqual(test.signature);
    });

    it('recover signature from signature buffer', () => {
      const address1 = recoverFromSigBuffer(test.data, test.signature);

      expect(address1).toEqual(test.address);
    });

    it('recover signature using a hash and r s v values', () => {
      const sig = sign(test.data, test.privateKey);
      const address1 = recoverFromVRS(test.data, sig.v, sig.r, sig.s);

      expect(address1).toEqual(test.address);
    });

    it('recover signature using a signature object', () => {
      const sig = sign(test.data, test.privateKey);
      const address1 = recoverFromSignature(test.data, sig);

      expect(address1).toEqual(test.address);
    });
  });
});
