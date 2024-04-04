import {
  type AccountWalletWithPrivateKey,
  type AztecAddress,
  type EthAddress,
  type FeePaymentMethod,
  NativeFeePaymentMethod,
  PrivateFeePaymentMethod,
  PublicFeePaymentMethod,
  TxStatus,
  getContractClassFromArtifact,
} from '@aztec/aztec.js';
import { FPCContract, GasTokenContract, TokenContract } from '@aztec/noir-contracts.js';
import { getCanonicalGasTokenAddress } from '@aztec/protocol-contracts/gas-token';

import { jest } from '@jest/globals';

import { publicDeployAccounts, setup } from '../fixtures/utils.js';

jest.setTimeout(50_000);

describe('benchmarks/tx_size_fees', () => {
  let aliceWallet: AccountWalletWithPrivateKey;
  let bobAddress: AztecAddress;
  let sequencerAddress: AztecAddress;
  let gas: GasTokenContract;
  let fpc: FPCContract;
  let token: TokenContract;
  let gasPortalAddress: EthAddress;

  // setup the environment
  beforeAll(async () => {
    const { wallets, aztecNode, deployL1ContractsValues } = await setup(3);

    gasPortalAddress = deployL1ContractsValues.l1ContractAddresses.gasPortalAddress;

    aliceWallet = wallets[0];
    bobAddress = wallets[1].getAddress();
    sequencerAddress = wallets[2].getAddress();

    await aztecNode.setConfig({
      feeRecipient: sequencerAddress,
      allowedFeePaymentContractClasses: [getContractClassFromArtifact(FPCContract.artifact).id],
    });

    await publicDeployAccounts(aliceWallet, wallets);
  });

  // deploy the contracts
  beforeAll(async () => {
    gas = await GasTokenContract.at(getCanonicalGasTokenAddress(gasPortalAddress), aliceWallet);
    token = await TokenContract.deploy(aliceWallet, aliceWallet.getAddress(), 'test', 'test', 18).send().deployed();
    fpc = await FPCContract.deploy(aliceWallet, token.address, gas.address).send().deployed();
  });

  // mint tokens
  beforeAll(async () => {
    await Promise.all([
      gas.methods.mint_public(aliceWallet.getAddress(), 1000n).send().wait(),
      token.methods.privately_mint_private_note(1000n).send().wait(),
      token.methods.mint_public(aliceWallet.getAddress(), 1000n).send().wait(),

      gas.methods.mint_public(fpc.address, 1000n).send().wait(),
    ]);
  });

  it.each<() => Promise<FeePaymentMethod | undefined>>([
    () => Promise.resolve(undefined),
    () => NativeFeePaymentMethod.create(aliceWallet),
    () => Promise.resolve(new PublicFeePaymentMethod(token.address, fpc.address, aliceWallet)),
    () => Promise.resolve(new PrivateFeePaymentMethod(token.address, fpc.address, aliceWallet)),
  ])('sends a tx with a fee', async createPaymentMethod => {
    const paymentMethod = await createPaymentMethod();
    const tx = await token.methods
      .transfer(aliceWallet.getAddress(), bobAddress, 1n, 0)
      .send({
        fee: paymentMethod
          ? {
              maxFee: 3n,
              paymentMethod,
            }
          : undefined,
      })
      .wait();

    expect(tx.status).toEqual(TxStatus.MINED);
  });
});
