import {
  type AccountWallet,
  type AztecAddress,
  BatchCall,
  type DebugLogger,
  ExtendedNote,
  Fr,
  Note,
  type PXE,
  computeSecretHash,
  deriveKeys,
} from '@aztec/aztec.js';
import { computePartialAddress } from '@aztec/circuits.js';
import { EscrowContract } from '@aztec/noir-contracts.js/Escrow';
import { TokenContract } from '@aztec/noir-contracts.js/Token';

import { setup } from './fixtures/utils.js';

describe('e2e_escrow_contract', () => {
  let pxe: PXE;
  let wallet: AccountWallet;
  let recipientWallet: AccountWallet;

  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let token: TokenContract;
  let escrowContract: EscrowContract;
  let owner: AztecAddress;
  let recipient: AztecAddress;

  let escrowSecretKey: Fr;
  let escrowPublicKeysHash: Fr;

  beforeEach(async () => {
    // Setup environment
    ({
      teardown,
      pxe,
      wallets: [wallet, recipientWallet],
      logger,
    } = await setup(2));
    owner = wallet.getAddress();
    recipient = recipientWallet.getAddress();

    // Generate private key for escrow contract, register key in pxe service, and deploy
    // Note that we need to register it first if we want to emit an encrypted note for it in the constructor
    escrowSecretKey = Fr.random();
    escrowPublicKeysHash = deriveKeys(escrowSecretKey).publicKeys.hash();
    const escrowDeployment = EscrowContract.deployWithPublicKeysHash(escrowPublicKeysHash, wallet, owner);
    const escrowInstance = escrowDeployment.getInstance();
    await pxe.registerAccount(escrowSecretKey, computePartialAddress(escrowInstance));
    escrowContract = await escrowDeployment.send().deployed();
    logger.info(`Escrow contract deployed at ${escrowContract.address}`);

    // Deploy Token contract and mint funds for the escrow contract
    token = await TokenContract.deploy(wallet, owner, 'TokenName', 'TokenSymbol', 18).send().deployed();

    const mintAmount = 100n;
    const secret = Fr.random();
    const secretHash = computeSecretHash(secret);

    const receipt = await token.methods.mint_private(mintAmount, secretHash).send().wait();

    const note = new Note([new Fr(mintAmount), secretHash]);

    const extendedNote = new ExtendedNote(
      note,
      owner,
      token.address,
      TokenContract.storage.pending_shields.slot,
      TokenContract.notes.TransparentNote.id,
      receipt.txHash,
    );
    await wallet.addNote(extendedNote);

    await token.methods.redeem_shield(escrowContract.address, mintAmount, secret).send().wait();

    // We allow our wallet to see the escrow contract's notes.
    wallet.setScopes([wallet.getAddress(), escrowContract.address]);

    logger.info(`Token contract deployed at ${token.address}`);
  });

  afterEach(() => teardown(), 30_000);

  const expectBalance = async (who: AztecAddress, expectedBalance: bigint) => {
    const balance = await token.methods.balance_of_private(who).simulate({ from: who });
    logger.info(`Account ${who} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  it('withdraws funds from the escrow contract', async () => {
    await expectBalance(owner, 0n);
    await expectBalance(recipient, 0n);
    await expectBalance(escrowContract.address, 100n);

    logger.info(`Withdrawing funds from token contract to ${recipient}`);
    await escrowContract.methods.withdraw(token.address, 30, recipient).send().wait();

    await expectBalance(owner, 0n);
    await expectBalance(recipient, 30n);
    await expectBalance(escrowContract.address, 70n);
  });

  it('refuses to withdraw funds as a non-owner', async () => {
    await expect(
      escrowContract.withWallet(recipientWallet).methods.withdraw(token.address, 30, recipient).prove(),
    ).rejects.toThrow();
  });

  it('moves funds using multiple keys on the same tx (#1010)', async () => {
    logger.info(`Minting funds in token contract to ${owner}`);
    const mintAmount = 50n;
    const secret = Fr.random();
    const secretHash = computeSecretHash(secret);

    const receipt = await token.methods.mint_private(mintAmount, secretHash).send().wait();

    const note = new Note([new Fr(mintAmount), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      owner,
      token.address,
      TokenContract.storage.pending_shields.slot,
      TokenContract.notes.TransparentNote.id,
      receipt.txHash,
    );
    await wallet.addNote(extendedNote);

    await token.methods.redeem_shield(owner, mintAmount, secret).send().wait();

    await expectBalance(owner, 50n);

    const actions = [
      token.methods.transfer(recipient, 10).request(),
      escrowContract.methods.withdraw(token.address, 20, recipient).request(),
    ];

    await new BatchCall(wallet, actions).send().wait();
    await expectBalance(recipient, 30n);
  });
});
