import {
  type AccountWalletWithSecretKey,
  type AztecAddress,
  FeeJuicePaymentMethod,
  type FeePaymentMethod,
  PrivateFeePaymentMethod,
  PublicFeePaymentMethod,
  TxStatus,
} from '@aztec/aztec.js';
import { GasSettings } from '@aztec/circuits.js';
import { FPCContract, FeeJuiceContract, TokenContract } from '@aztec/noir-contracts.js';
import { FeeJuiceAddress } from '@aztec/protocol-contracts/fee-juice';

import { jest } from '@jest/globals';

import { publicDeployAccounts, setup } from '../fixtures/utils.js';

jest.setTimeout(100_000);

describe('benchmarks/tx_size_fees', () => {
  let aliceWallet: AccountWalletWithSecretKey;
  let bobAddress: AztecAddress;
  let sequencerAddress: AztecAddress;
  let feeJuice: FeeJuiceContract;
  let fpc: FPCContract;
  let token: TokenContract;

  // setup the environment
  beforeAll(async () => {
    const { wallets, aztecNode } = await setup(3, {}, {}, true);

    aliceWallet = wallets[0];
    bobAddress = wallets[1].getAddress();
    sequencerAddress = wallets[2].getAddress();

    await aztecNode.setConfig({
      feeRecipient: sequencerAddress,
    });

    await publicDeployAccounts(aliceWallet, wallets);
  });

  // deploy the contracts
  beforeAll(async () => {
    feeJuice = await FeeJuiceContract.at(FeeJuiceAddress, aliceWallet);
    token = await TokenContract.deploy(aliceWallet, aliceWallet.getAddress(), 'test', 'test', 18).send().deployed();
    fpc = await FPCContract.deploy(aliceWallet, token.address).send().deployed();
  });

  // mint tokens
  beforeAll(async () => {
    await Promise.all([
      feeJuice.methods.mint_public(aliceWallet.getAddress(), 100e9).send().wait(),
      feeJuice.methods.mint_public(fpc.address, 100e9).send().wait(),
    ]);
    await token.methods.privately_mint_private_note(100e9).send().wait();
    await token.methods.mint_public(aliceWallet.getAddress(), 100e9).send().wait();
  });

  it.each<[string, () => FeePaymentMethod | undefined /*bigint*/]>([
    ['no', () => undefined /*200021120n*/],
    [
      'fee_juice',
      () => new FeeJuicePaymentMethod(aliceWallet.getAddress()),
      // Same cost as no fee payment, since payment is done natively
      // 200021120n,
    ],
    [
      'public fee',
      () => new PublicFeePaymentMethod(token.address, fpc.address, aliceWallet),
      // DA:
      // non-rev: 1 nullifiers, overhead; rev: 2 note hashes, 1 nullifier, 1168 B enc note logs, 0 B enc logs,0 B unenc logs, teardown
      // L2:
      // non-rev: 0; rev: 0
      // 200062330n,
    ],
    [
      'private fee',
      () => new PrivateFeePaymentMethod(token.address, fpc.address, aliceWallet),
      // DA:
      // non-rev: 3 nullifiers, overhead; rev: 2 note hashes, 1168 B enc note logs, 0 B enc logs, 0 B unenc logs, teardown
      // L2:
      // non-rev: 0; rev: 0
      // 200032492n,
    ],
  ] as const)(
    'sends a tx with a fee with %s payment method',
    async (_name, createPaymentMethod /*expectedTransactionFee*/) => {
      const paymentMethod = createPaymentMethod();
      const gasSettings = GasSettings.default();
      const tx = await token.methods
        .transfer(bobAddress, 1n)
        .send({ fee: paymentMethod ? { gasSettings, paymentMethod } : undefined })
        .wait();

      expect(tx.status).toEqual(TxStatus.SUCCESS);
      // TODO: reinstante this check when ossified
      // expect(tx.transactionFee).toEqual(expectedTransactionFee);
    },
  );
});
