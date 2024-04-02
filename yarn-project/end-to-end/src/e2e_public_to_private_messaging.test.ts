import { type AztecAddress, type DebugLogger, type EthAddress } from '@aztec/aztec.js';

import { setup } from './fixtures/utils.js';
import { CrossChainTestHarness } from './shared/cross_chain_test_harness.js';

describe('e2e_public_to_private_messaging', () => {
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let ethAccount: EthAddress;

  let underlyingERC20: any;

  let ownerAddress: AztecAddress;

  let crossChainTestHarness: CrossChainTestHarness;

  beforeEach(async () => {
    const { aztecNode, pxe, deployL1ContractsValues, wallet, logger: logger_, teardown: teardown_ } = await setup(2);
    crossChainTestHarness = await CrossChainTestHarness.new(
      aztecNode,
      pxe,
      deployL1ContractsValues.publicClient,
      deployL1ContractsValues.walletClient,
      wallet,
      logger_,
    );

    ethAccount = crossChainTestHarness.ethAccount;
    ownerAddress = crossChainTestHarness.ownerAddress;
    underlyingERC20 = crossChainTestHarness.underlyingERC20;

    teardown = teardown_;
    logger = logger_;
    logger('Successfully deployed contracts and initialized portal');
  }, 100_000);

  afterEach(async () => {
    await teardown();
  });

  it('Milestone 5.4: Should be able to create a commitment in a public function and spend in a private function', async () => {
    // Generate a claim secret using pedersen
    const l1TokenBalance = 1000000n;
    const bridgeAmount = 100n;
    const shieldAmount = 50n;

    const [secret, secretHash] = crossChainTestHarness.generateClaimSecret();

    await crossChainTestHarness.mintTokensOnL1(l1TokenBalance);
    const msgHash = await crossChainTestHarness.sendTokensToPortalPublic(bridgeAmount, secretHash);
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(l1TokenBalance - bridgeAmount);

    await crossChainTestHarness.makeMessageConsumable(msgHash);

    await crossChainTestHarness.consumeMessageOnAztecAndMintPublicly(bridgeAmount, secret);
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, bridgeAmount);

    // Create the commitment to be spent in the private domain
    await crossChainTestHarness.shieldFundsOnL2(shieldAmount, secretHash);

    // Create the transaction spending the commitment
    await crossChainTestHarness.redeemShieldPrivatelyOnL2(shieldAmount, secret);
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, bridgeAmount - shieldAmount);
    await crossChainTestHarness.expectPrivateBalanceOnL2(ownerAddress, shieldAmount);

    // Unshield the tokens again, sending them to the same account, however this can be any account.
    await crossChainTestHarness.unshieldTokensOnL2(shieldAmount);
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, bridgeAmount);
    await crossChainTestHarness.expectPrivateBalanceOnL2(ownerAddress, 0n);
  }, 200_000);
});
