import { AztecAddress, ContractDeployer, NativeFeePaymentMethod } from '@aztec/aztec.js';
import { GasTokenContract, TokenContract } from '@aztec/noir-contracts.js';
import { getCanonicalGasToken } from '@aztec/protocol-contracts/gas-token';

import { setup } from './fixtures/utils.js';

describe('e2e_fees', () => {
  let aliceAddress: AztecAddress;
  let _bobAddress: AztecAddress;
  let sequencerAddress: AztecAddress;
  let gasTokenContract: GasTokenContract;
  let testContract: TokenContract;

  beforeAll(async () => {
    process.env.PXE_URL = '';
    const { accounts, aztecNode, wallet } = await setup(3);

    await aztecNode.setConfig({
      feeRecipient: accounts.at(-1)!.address,
    });
    const canonicalGasToken = getCanonicalGasToken();
    const deployer = new ContractDeployer(canonicalGasToken.artifact, wallet);
    const { contract } = await deployer
      .deploy()
      .send({
        contractAddressSalt: canonicalGasToken.instance.salt,
      })
      .wait();

    gasTokenContract = contract as GasTokenContract;
    aliceAddress = accounts.at(0)!.address;
    _bobAddress = accounts.at(1)!.address;
    sequencerAddress = accounts.at(-1)!.address;

    testContract = await TokenContract.deploy(wallet, aliceAddress, 'Test', 'TEST', 1).send().deployed();

    // Alice gets a balance of 1000 gas token
    await gasTokenContract.methods.redeem_bridged_balance(1000).send().wait();
  }, 100_000);

  it('deploys gas token contract at canonical address', () => {
    expect(gasTokenContract.address).toEqual(getCanonicalGasToken().address);
  });

  describe('NativeFeePaymentMethod', () => {
    it.skip('pays out the expected fee to the sequencer', async () => {
      await testContract.methods
        .mint_public(aliceAddress, 1000)
        .send({
          fee: {
            maxFee: 1,
            paymentMethod: new NativeFeePaymentMethod(),
          },
        })
        .wait();

      const [sequencerBalance, aliceBalance] = await Promise.all([
        gasTokenContract.methods.balance_of(sequencerAddress).view(),
        gasTokenContract.methods.balance_of(aliceAddress).view(),
      ]);

      expect(sequencerBalance).toEqual(1n);
      expect(aliceBalance).toEqual(999n);
    });
  });
});
