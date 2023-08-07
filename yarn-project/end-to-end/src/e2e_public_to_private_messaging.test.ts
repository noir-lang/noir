import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress, AztecRPC } from '@aztec/aztec.js';
import { EthAddress } from '@aztec/circuits.js';
import { DebugLogger } from '@aztec/foundation/log';

import { CrossChainTestHarness } from './fixtures/cross_chain_test_harness.js';
import { delay, setup } from './fixtures/utils.js';

describe('e2e_public_to_private_messaging', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let logger: DebugLogger;

  let ethAccount: EthAddress;

  let underlyingERC20: any;

  const initialBalance = 10n;
  let ownerAddress: AztecAddress;

  let crossChainTestHarness: CrossChainTestHarness;

  beforeEach(async () => {
    const {
      aztecNode: aztecNode_,
      aztecRpcServer: aztecRpcServer_,
      deployL1ContractsValues,
      accounts,
      wallet,
      logger: logger_,
      cheatCodes,
    } = await setup(2);
    crossChainTestHarness = await CrossChainTestHarness.new(
      initialBalance,
      aztecNode_,
      aztecRpcServer_,
      deployL1ContractsValues,
      accounts,
      wallet,
      logger_,
      cheatCodes,
    );

    ethAccount = crossChainTestHarness.ethAccount;
    ownerAddress = crossChainTestHarness.ownerAddress;
    underlyingERC20 = crossChainTestHarness.underlyingERC20;
    aztecRpcServer = crossChainTestHarness.aztecRpcServer;
    aztecNode = aztecNode_;

    logger = logger_;
    logger('Successfully deployed contracts and initialized portal');
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) {
      await aztecRpcServer?.stop();
    }
    await crossChainTestHarness?.stop();
  });

  it('Milestone 5.4: Should be able to create a commitment in a public function and spend in a private function', async () => {
    // Generate a claim secret using pedersen
    const l1TokenBalance = 1000000n;
    const bridgeAmount = 100n;
    const shieldAmount = 50n;

    const [secret, secretHash] = await crossChainTestHarness.generateClaimSecret();

    await crossChainTestHarness.mintTokensOnL1(l1TokenBalance);
    const messageKey = await crossChainTestHarness.sendTokensToPortal(bridgeAmount, secretHash);
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(l1TokenBalance - bridgeAmount);

    // Wait for the archiver to process the message
    await delay(5000); /// waiting 5 seconds.

    // Perform another unrelated transaction on L2 to progress the rollup.
    await crossChainTestHarness.expectBalanceOnL2(ownerAddress, initialBalance);
    const transferAmount = 1n;
    await crossChainTestHarness.performL2Transfer(transferAmount);
    await crossChainTestHarness.expectBalanceOnL2(ownerAddress, initialBalance - transferAmount);

    await crossChainTestHarness.consumeMessageOnAztecAndMintPublicly(bridgeAmount, messageKey, secret);
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, bridgeAmount);

    // Create the commitment to be spent in the private domain
    await crossChainTestHarness.shieldFundsOnL2(shieldAmount, secretHash);

    // Create the transaction spending the commitment
    await crossChainTestHarness.redeemShieldPrivatelyOnL2(shieldAmount, secret);
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, bridgeAmount - shieldAmount);
    await crossChainTestHarness.expectBalanceOnL2(ownerAddress, initialBalance + shieldAmount - transferAmount);

    // Unshield the tokens again, sending them to the same account, however this can be any account.
    await crossChainTestHarness.unshieldTokensOnL2(shieldAmount);
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, bridgeAmount);
    await crossChainTestHarness.expectBalanceOnL2(ownerAddress, initialBalance - transferAmount);
  }, 200_000);
});
