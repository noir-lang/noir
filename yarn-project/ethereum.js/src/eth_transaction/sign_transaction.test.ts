import { EthAddress } from '../eth_address/index.js';
import { recoverTransaction, signTransaction } from './sign_transaction.js';

// For test with accessList below, not yet supported.
// var accessList = [
//   {
//     address: '0x0000000000000000000000000000000000000101',
//     storageKeys: [
//       "0x0000000000000000000000000000000000000000000000000000000000000000",
//       "0x00000000000000000000000000000000000000000000000000000000000060a7"
//     ]
//   }
// ];

describe('eth_account sign transaction', () => {
  // const testData = {
  //   address: EthAddress.fromString('0x2c7536E3605D9C16a7a3D7b1898e529396a65c23'),
  //   iban: 'XE0556YCRTEZ9JALZBSCXOK4UJ5F3HN03DV',
  //   privateKey: Buffer.from('4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318', 'hex'),
  //   transaction: {
  //     chainId: 1,
  //     nonce: 0,
  //     maxFeePerGas: BigInt(20000000000),
  //     maxPriorityFeePerGas: BigInt(0),
  //     gas: 21000,
  //     to: EthAddress.fromString('0xF0109fC8DF283027b6285cc889F5aA624EaC1F55'),
  //     value: BigInt(1000000000),
  //   } as EthTransaction,
  //   rawTransaction: Buffer.from(
  //     'f868808504a817c80082520894f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca008026a0afa02d193471bb974081585daabf8a751d4decbb519604ac7df612cc11e9226da04bf1bd55e82cebb2b09ed39bbffe35107ea611fa212c2d9a1f1ada4952077118',
  //     'hex',
  //   ),
  // };

  const testData =
    //[
    // {
    //   // test #24
    //   address: EthAddress.fromString('0x2c7536E3605D9C16a7a3D7b1898e529396a65c23'),
    //   // iban: 'XE0556YCRTEZ9JALZBSCXOK4UJ5F3HN03DV',
    //   privateKey: Buffer.from('4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318', 'hex'),
    //   transaction: {
    //     chainId: 1,
    //     nonce: 0,
    //     maxPriorityFeePerGas: BigInt('1000000000'),
    //     maxFeePerGas: BigInt('3000000000'),
    //     gas: 27200,
    //     to: EthAddress.fromString('0xF0109fC8DF283027b6285cc889F5aA624EaC1F55'),
    //     // toIban: 'XE04S1IRT2PR8A8422TPBL9SR6U0HODDCUT', // will be switched to "to" in the test
    //     value: BigInt('1000000000'),
    //     // accessList: accessList,
    //   },
    //   // signature from eth_signTransaction
    //   rawTransaction:
    //     '0x02f8ca0180843b9aca0084b2d05e00826a4094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080f85bf859940000000000000000000000000000000000000101f842a00000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000060a701a0e86d3360f40f934686e1f9e53d5f49634adb0227169dd8a93b66683eb32b9c1ca04e5851b4601e2e9178148ca0f4f8360d9fba16baf272931debdf270ffa6fafc9',
    //   oldSignature:
    //     '0x02f8ca0180843b9aca0084b2d05e00826a4094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080f85bf859940000000000000000000000000000000000000101f842a00000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000060a701a0e86d3360f40f934686e1f9e53d5f49634adb0227169dd8a93b66683eb32b9c1ca04e5851b4601e2e9178148ca0f4f8360d9fba16baf272931debdf270ffa6fafc9',
    //   transactionHash: '0xc102cf9a2cfa23b06d013970497793077c2fa2a203ec32ddeee2ec87a4ab1cb8',
    //   messageHash: '0x69325a2750893097fb1b7ab621bacef5fc20fd33374e1c3f44a79f9f35602b59',
    // };
    {
      // test #25
      address: EthAddress.fromString('0x2c7536E3605D9C16a7a3D7b1898e529396a65c23'),
      // iban: 'XE0556YCRTEZ9JALZBSCXOK4UJ5F3HN03DV',
      privateKey: Buffer.from('4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318', 'hex'),
      transaction: {
        chainId: 1,
        nonce: 0,
        maxPriorityFeePerGas: BigInt('0x3B9ACA00'),
        maxFeePerGas: BigInt('0xB2D05E00'),
        gas: 27200,
        to: EthAddress.fromString('0xF0109fC8DF283027b6285cc889F5aA624EaC1F55'),
        // toIban: 'XE04S1IRT2PR8A8422TPBL9SR6U0HODDCUT', // will be switched to "to" in the test
        value: BigInt('1000000000'),
        // data: ,
        // common: commonLondon,
      },
      // signature from eth_signTransaction
      rawTransaction: Buffer.from(
        '02f86e0180843b9aca0084b2d05e00826a4094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080c080a0d1290a118d51918c1ca17e3af0267c45efcd745cf42e78eabc444c424d6bcf37a003c81e1fda169575023a94200ee034128747f91020e704abaee30dbcfc785c36',
        'hex',
      ),
      oldSignature: Buffer.from(
        '02f86e0180843b9aca0084b2d05e00826a4094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080c080a0d1290a118d51918c1ca17e3af0267c45efcd745cf42e78eabc444c424d6bcf37a003c81e1fda169575023a94200ee034128747f91020e704abaee30dbcfc785c36',
        'hex',
      ),
      transactionHash: Buffer.from('82c19b39a6b7eaa0492863a8b236fad5018f267b4977c270ddd5228c4cbda60e', 'hex'),
      messageHash: Buffer.from('e3beea0918f445c21eb2f42e3cbc3c5d54321ec642f47d12c473b2765df97f2b', 'hex'),
    };
  /*
    {
      // test #26
      address: '0x2c7536E3605D9C16a7a3D7b1898e529396a65c23',
      iban: 'XE0556YCRTEZ9JALZBSCXOK4UJ5F3HN03DV',
      privateKey: '0x4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318',
      transaction: {
        chainId: 1,
        nonce: 0,
        gas: 27200,
        maxPriorityFeePerGas: '0x3B9ACA00',
        gasLimit: '0x6A40',
        to: '0xF0109fC8DF283027b6285cc889F5aA624EaC1F55',
        toIban: 'XE04S1IRT2PR8A8422TPBL9SR6U0HODDCUT', // will be switched to "to" in the test
        value: '1000000000',
        data: '',
        common: commonLondon,
      },
      // signature from eth_signTransaction
      rawTransaction:
        '0x02f86e0180843b9aca00843b9aca0e826a4094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080c080a0eb8ca6017e6926503ce11c404ba9b61f30d53ea934857e4f4489f43a6c189cf8a03655ba42b2fdcabdb3363cb39e7f672baa91455632e02bab27f92e1a275ca833',
      oldSignature:
        '0x02f86e0180843b9aca00843b9aca0e826a4094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080c080a0eb8ca6017e6926503ce11c404ba9b61f30d53ea934857e4f4489f43a6c189cf8a03655ba42b2fdcabdb3363cb39e7f672baa91455632e02bab27f92e1a275ca833',
      transactionHash: '0x488a813f2286f7c015947aa13133bdae49ec75ae1c8f5eba80034d71a038dca8',
      messageHash: '0xcd6d6dee80ecc38f1b22f2d128bf6043dc41079fc913183a8995b5b3e187df61',
    },
    {
      // test #27
      address: '0x2c7536E3605D9C16a7a3D7b1898e529396a65c23',
      iban: 'XE0556YCRTEZ9JALZBSCXOK4UJ5F3HN03DV',
      privateKey: '0x4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318',
      transaction: {
        chainId: 1,
        nonce: 0,
        gas: 27200,
        maxPriorityFeePerGas: '0x3B9ACA00',
        gasLimit: '0x6A40',
        to: '0xF0109fC8DF283027b6285cc889F5aA624EaC1F55',
        toIban: 'XE04S1IRT2PR8A8422TPBL9SR6U0HODDCUT', // will be switched to "to" in the test
        value: '1000000000',
        data: '',
        common: commonLondon,
        accessList: accessList,
      },
      // signature from eth_signTransaction
      rawTransaction:
        '0x02f8ca0180843b9aca00843b9aca0e826a4094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080f85bf859940000000000000000000000000000000000000101f842a00000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000060a780a0e3a2e10c7d3af3407ec2d38c64788d6673926e9b28d6d2e7df3c94cdf0548233a00ad3e5faafaf3a9350ab16c1be0198ce9ff3c6bef0b91e05488d757f07de9557',
      oldSignature:
        '0x02f8ca0180843b9aca00843b9aca0e826a4094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080f85bf859940000000000000000000000000000000000000101f842a00000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000060a780a0e3a2e10c7d3af3407ec2d38c64788d6673926e9b28d6d2e7df3c94cdf0548233a00ad3e5faafaf3a9350ab16c1be0198ce9ff3c6bef0b91e05488d757f07de9557',
      transactionHash: '0xbc2c9edab3d4e3a795fa402b52d6149e874de15f0cc6c0858eb34e1fe1ef31fe',
      messageHash: '0xa3a2cdc45e9cefb9a614ead90ce65f68bcf8a90dbe0ccbd84c1b62403bd05346',
    },
    {
      // test #28
      address: '0x2c7536E3605D9C16a7a3D7b1898e529396a65c23',
      iban: 'XE0556YCRTEZ9JALZBSCXOK4UJ5F3HN03DV',
      privateKey: '0x4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318',
      transaction: {
        chainId: 1,
        nonce: 0,
        maxPriorityFeePerGas: new bn('c7331248', 16),
        maxFeePerGas: new bn('cb72ec20', 16),
        gasLimit: '0x6A40',
        to: '0xF0109fC8DF283027b6285cc889F5aA624EaC1F55',
        toIban: 'XE04S1IRT2PR8A8422TPBL9SR6U0HODDCUT', // will be switched to "to" in the test
        value: '1000000000',
        data: '',
        common: commonLondon,
      },
      // signature from eth_signTransaction
      rawTransaction:
        '0x02f86e018084c733124884cb72ec20826a4094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080c001a08896fb9a5c033e0163b073cf7a951a1db2dca41b26b4188f13a05158eb26fd32a005e8855691199cd0b6dcae88f3325c374e2f0697b9c528a5c10d5bd8dfb6a3e3',
      oldSignature:
        '0x02f86e018084c733124884cb72ec20826a4094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080c001a08896fb9a5c033e0163b073cf7a951a1db2dca41b26b4188f13a05158eb26fd32a005e8855691199cd0b6dcae88f3325c374e2f0697b9c528a5c10d5bd8dfb6a3e3',
      transactionHash: '0xd5b7290a477b9c421d39e61d0f566ec33276fb49b9ff85cfd6152a18f1c92dab',
      messageHash: '0x17e20e530a889ce52057de228b5b97edcad6002468d723346cd0b6b7a9943457',
    },
    {
      // test #29
      address: '0x2c7536E3605D9C16a7a3D7b1898e529396a65c23',
      iban: 'XE0556YCRTEZ9JALZBSCXOK4UJ5F3HN03DV',
      privateKey: '0x4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318',
      transaction: {
        chainId: 1,
        nonce: 0,
        maxPriorityFeePerGas: '1000000000',
        maxFeePerGas: '3000000000',
        gasLimit: '0x6A40',
        to: '0xF0109fC8DF283027b6285cc889F5aA624EaC1F55',
        toIban: 'XE04S1IRT2PR8A8422TPBL9SR6U0HODDCUT', // will be switched to "to" in the test
        value: '1000000000',
        data: '',
        common: commonLondon,
      },
      // signature from eth_signTransaction
      rawTransaction:
        '0x02f86e0180843b9aca0084b2d05e00826a4094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080c080a0d1290a118d51918c1ca17e3af0267c45efcd745cf42e78eabc444c424d6bcf37a003c81e1fda169575023a94200ee034128747f91020e704abaee30dbcfc785c36',
      oldSignature:
        '0x02f86e0180843b9aca0084b2d05e00826a4094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080c080a0d1290a118d51918c1ca17e3af0267c45efcd745cf42e78eabc444c424d6bcf37a003c81e1fda169575023a94200ee034128747f91020e704abaee30dbcfc785c36',
      transactionHash: '0x82c19b39a6b7eaa0492863a8b236fad5018f267b4977c270ddd5228c4cbda60e',
      messageHash: '0xe3beea0918f445c21eb2f42e3cbc3c5d54321ec642f47d12c473b2765df97f2b',
    },
    {
      // test #30
      address: '0x2c7536E3605D9C16a7a3D7b1898e529396a65c23',
      iban: 'XE0556YCRTEZ9JALZBSCXOK4UJ5F3HN03DV',
      privateKey: '0x4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318',
      transaction: {
        chainId: 1,
        nonce: 0,
        maxPriorityFeePerGas: new utils.BN('0x3B9ACA00'),
        maxFeePerGas: new utils.BN('0xB2D05E00'),
        gasLimit: '0x6A40',
        to: '0xF0109fC8DF283027b6285cc889F5aA624EaC1F55',
        toIban: 'XE04S1IRT2PR8A8422TPBL9SR6U0HODDCUT', // will be switched to "to" in the test
        value: '1000000000',
        data: '',
        common: commonLondon,
      },
      // signature from eth_signTransaction
      rawTransaction:
        '0x02f86e0180843b9aca0084b2d05e00826a4094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080c080a0d1290a118d51918c1ca17e3af0267c45efcd745cf42e78eabc444c424d6bcf37a003c81e1fda169575023a94200ee034128747f91020e704abaee30dbcfc785c36',
      oldSignature:
        '0x02f86e018084c733124884cb72ec20826a4094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080c001a08896fb9a5c033e0163b073cf7a951a1db2dca41b26b4188f13a05158eb26fd32a005e8855691199cd0b6dcae88f3325c374e2f0697b9c528a5c10d5bd8dfb6a3e3',
      transactionHash: '0x82c19b39a6b7eaa0492863a8b236fad5018f267b4977c270ddd5228c4cbda60e',
      messageHash: '0xe3beea0918f445c21eb2f42e3cbc3c5d54321ec642f47d12c473b2765df97f2b',
    },
  ];
  */

  // it('signTransaction using the iban as "to" must compare to eth_signTransaction', () => {
  //   const transaction = {
  //     ...testData.transaction,
  //     to: 'XE04S1IRT2PR8A8422TPBL9SR6U0HODDCUT' as any,
  //   };
  //   const tx = signTransaction(transaction, testData.privateKey);
  //   expect(tx.rawTransaction).toEqual(testData.rawTransaction);
  // });

  it('should create correct signature', () => {
    const tx = signTransaction(testData.transaction, testData.privateKey);
    expect(tx.messageHash).toEqual(testData.messageHash);
    expect(tx.rawTransaction).toEqual(testData.rawTransaction);
  });

  it('should recover address from signature', () => {
    const tx = signTransaction(testData.transaction, testData.privateKey);
    expect(recoverTransaction(tx.rawTransaction)).toEqual(testData.address);
  });
});
