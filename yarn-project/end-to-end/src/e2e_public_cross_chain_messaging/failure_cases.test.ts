import { EthAddress, Fr, L1Actor, L1ToL2Message, L2Actor } from '@aztec/aztec.js';
import { sha256ToField } from '@aztec/foundation/crypto';

import { toFunctionSelector } from 'viem';

import { PublicCrossChainMessagingContractTest } from './public_cross_chain_messaging_contract_test.js';

describe('e2e_public_cross_chain_messaging failures', () => {
  const t = new PublicCrossChainMessagingContractTest('failures');

  let { crossChainTestHarness, ethAccount, l2Bridge, user1Wallet, user2Wallet } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.setup();
    // Have to destructure again to ensure we have latest refs.
    ({ crossChainTestHarness, user1Wallet, user2Wallet } = t);
    ethAccount = crossChainTestHarness.ethAccount;
    l2Bridge = crossChainTestHarness.l2Bridge;
  }, 300_000);

  afterAll(async () => {
    await t.teardown();
  });

  it("Bridge can't withdraw my funds if I don't give approval", async () => {
    const mintAmountToOwner = 100n;
    await crossChainTestHarness.mintTokensPublicOnL2(mintAmountToOwner);

    const withdrawAmount = 9n;
    const nonce = Fr.random();
    // Should fail as owner has not given approval to bridge burn their funds.
    await expect(
      l2Bridge
        .withWallet(user1Wallet)
        .methods.exit_to_l1_public(ethAccount, withdrawAmount, EthAddress.ZERO, nonce)
        .prove(),
    ).rejects.toThrow(/unauthorized/);
  }, 60_000);

  it("can't claim funds privately which were intended for public deposit from the token portal", async () => {
    const bridgeAmount = 100n;
    const [secret, secretHash] = crossChainTestHarness.generateClaimSecret();

    await crossChainTestHarness.mintTokensOnL1(bridgeAmount);
    const msgHash = await crossChainTestHarness.sendTokensToPortalPublic(bridgeAmount, secretHash);
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(0n);

    await crossChainTestHarness.makeMessageConsumable(msgHash);

    // Wrong message hash
    const content = sha256ToField([
      Buffer.from(toFunctionSelector('mint_private(bytes32,uint256)').substring(2), 'hex'),
      secretHash,
      new Fr(bridgeAmount),
    ]);
    const wrongMessage = new L1ToL2Message(
      new L1Actor(crossChainTestHarness.tokenPortalAddress, crossChainTestHarness.publicClient.chain.id),
      new L2Actor(l2Bridge.address, 1),
      content,
      secretHash,
    );

    await expect(
      l2Bridge.withWallet(user2Wallet).methods.claim_private(secretHash, bridgeAmount, secret).prove(),
    ).rejects.toThrow(`No non-nullified L1 to L2 message found for message hash ${wrongMessage.hash().toString()}`);
  }, 60_000);
});
