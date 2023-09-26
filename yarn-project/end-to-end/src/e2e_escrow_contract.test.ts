import { AccountWallet, AztecAddress, BatchCall, computeMessageSecretHash, generatePublicKey } from '@aztec/aztec.js';
import { CompleteAddress, Fr, GrumpkinPrivateKey, GrumpkinScalar, getContractDeploymentInfo } from '@aztec/circuits.js';
import { DebugLogger } from '@aztec/foundation/log';
import { EscrowContractAbi } from '@aztec/noir-contracts/artifacts';
import { EscrowContract, TokenContract } from '@aztec/noir-contracts/types';
import { AztecRPC, PublicKey, TxStatus } from '@aztec/types';

import { setup } from './fixtures/utils.js';

describe('e2e_escrow_contract', () => {
  let aztecRpcServer: AztecRPC;
  let wallet: AccountWallet;
  let recipientWallet: AccountWallet;
  let accounts: CompleteAddress[];
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let token: TokenContract;
  let escrowContract: EscrowContract;
  let owner: AztecAddress;
  let recipient: AztecAddress;

  let escrowPrivateKey: GrumpkinPrivateKey;
  let escrowPublicKey: PublicKey;

  beforeEach(async () => {
    // Setup environment
    ({
      teardown,
      aztecRpcServer,
      accounts,
      wallets: [wallet, recipientWallet],
      logger,
    } = await setup(2));
    owner = accounts[0].address;
    recipient = accounts[1].address;

    // Generate private key for escrow contract, register key in rpc server, and deploy
    // Note that we need to register it first if we want to emit an encrypted note for it in the constructor
    escrowPrivateKey = GrumpkinScalar.random();
    escrowPublicKey = await generatePublicKey(escrowPrivateKey);
    const salt = Fr.random();
    const deployInfo = await getContractDeploymentInfo(EscrowContractAbi, [owner], salt, escrowPublicKey);
    await aztecRpcServer.registerAccount(escrowPrivateKey, deployInfo.completeAddress.partialAddress);

    escrowContract = await EscrowContract.deployWithPublicKey(wallet, escrowPublicKey, owner)
      .send({ contractAddressSalt: salt })
      .deployed();
    logger(`Escrow contract deployed at ${escrowContract.address}`);

    // Deploy Private Token contract and mint funds for the escrow contract
    token = await TokenContract.deploy(wallet).send().deployed();

    expect((await token.methods._initialize(owner).send().wait()).status).toBe(TxStatus.MINED);

    const secret = Fr.random();
    const secretHash = await computeMessageSecretHash(secret);

    expect((await token.methods.mint_private(100n, secretHash).send().wait()).status).toEqual(TxStatus.MINED);
    expect((await token.methods.redeem_shield(escrowContract.address, 100n, secret).send().wait()).status).toEqual(
      TxStatus.MINED,
    );

    logger(`Token contract deployed at ${token.address}`);
  }, 100_000);

  afterEach(() => teardown(), 30_000);

  const expectBalance = async (who: AztecAddress, expectedBalance: bigint) => {
    const balance = await token.methods.balance_of_private(who).view({ from: who });
    logger(`Account ${who} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  it('withdraws funds from the escrow contract', async () => {
    await expectBalance(owner, 0n);
    await expectBalance(recipient, 0n);
    await expectBalance(escrowContract.address, 100n);

    logger(`Withdrawing funds from token contract to ${recipient}`);
    await escrowContract.methods.withdraw(token.address, 30, recipient).send().wait();

    await expectBalance(owner, 0n);
    await expectBalance(recipient, 30n);
    await expectBalance(escrowContract.address, 70n);
  }, 60_000);

  it('refuses to withdraw funds as a non-owner', async () => {
    await expect(
      escrowContract.withWallet(recipientWallet).methods.withdraw(token.address, 30, recipient).simulate(),
    ).rejects.toThrowError();
  }, 60_000);

  it('moves funds using multiple keys on the same tx (#1010)', async () => {
    logger(`Minting funds in token contract to ${owner}`);
    const secret = Fr.random();
    const secretHash = await computeMessageSecretHash(secret);

    expect((await token.methods.mint_private(50n, secretHash).send().wait()).status).toEqual(TxStatus.MINED);
    expect((await token.methods.redeem_shield(owner, 50n, secret).send().wait()).status).toEqual(TxStatus.MINED);

    await expectBalance(owner, 50n);

    const actions = [
      token.methods.transfer(owner, recipient, 10, 0).request(),
      escrowContract.methods.withdraw(token.address, 20, recipient).request(),
    ];

    await new BatchCall(wallet, actions).send().wait();
    await expectBalance(recipient, 30n);
  }, 120_000);
});
