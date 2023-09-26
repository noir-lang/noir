import { CompleteAddress, TxHash, Wallet } from '@aztec/aztec.js';
import { Fr, MAX_NEW_COMMITMENTS_PER_CALL } from '@aztec/circuits.js';
import { DebugLogger } from '@aztec/foundation/log';
import { PrivateTokenAirdropContract } from '@aztec/noir-contracts/types';
import { NotePreimage } from '@aztec/types';

import { setup } from './fixtures/utils.js';

class Claim {
  static EMPTY = new Claim(0n, Fr.ZERO);

  constructor(public readonly amount: bigint, public readonly secret: Fr) {}

  get preimage() {
    return new NotePreimage([new Fr(this.amount), this.secret]);
  }
}

describe('private airdrop', () => {
  const numberOfAccounts = 3;
  const initialSupply = 1000n;
  const claimsStorageSlot = new Fr(2n);

  let wallets: Wallet[];
  let contracts: PrivateTokenAirdropContract[];
  let accounts: CompleteAddress[];
  let claims: Claim[];
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  beforeEach(async () => {
    ({ teardown, accounts, wallets, logger } = await setup(numberOfAccounts));

    logger(`Deploying zk token contract...`);
    const owner = accounts[0].address;
    const contract = await PrivateTokenAirdropContract.deploy(wallets[0], initialSupply, owner).send().deployed();
    logger(`zk token contract deployed at ${contract.address}`);

    contracts = [contract];
    for (let i = 1; i < accounts.length; ++i) {
      contracts.push(await PrivateTokenAirdropContract.at(contract.address, wallets[i]));
    }
  }, 100_000);

  afterEach(() => teardown());

  const expectBalance = async (accountIndex: number, expectedBalance: bigint) => {
    const account = accounts[accountIndex].address;
    const balance = await contracts[accountIndex].methods.getBalance(account).view({ from: account });
    logger(`Account ${accountIndex} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const createClaims = (amounts: bigint[]) => {
    claims = amounts.map(amount => new Claim(amount, Fr.random()));
    claims.push(
      ...Array(MAX_NEW_COMMITMENTS_PER_CALL - amounts.length)
        .fill(0)
        .map(() => Claim.EMPTY),
    );
  };

  const claimToken = async (accountIndex: number, claim: Claim, txHash: TxHash, nonceIndex = 0) => {
    const contract = contracts[accountIndex];
    const account = accounts[accountIndex].address;
    const wallet = wallets[accountIndex];
    const nonces = await wallet.getNoteNonces(contract.address, claimsStorageSlot, claim.preimage, txHash);

    const preimageBuf = claim.preimage.toBuffer();
    const numNonces = claims.reduce((count, c) => count + (c.preimage.toBuffer().equals(preimageBuf) ? 1 : 0), 0);
    expect(nonces.length).toBe(numNonces);
    expect(nonces[nonceIndex]).not.toEqual(Fr.ZERO);

    return contract.methods.claim(claim.amount, claim.secret, account, nonces[nonceIndex]).send().wait();
  };

  it('should create claim notes for any accounts to claim', async () => {
    let txHash: TxHash;

    // Transaction 1
    {
      logger(`Create claims...`);
      const accountIndex = 0;
      await expectBalance(accountIndex, initialSupply);

      createClaims([12n, 345n]);
      // Create a claim that has the exact same preimage as another claim.
      claims[2] = claims[0];

      const amounts = claims.map(c => c.amount);
      const secrets = claims.map(c => c.secret);
      ({ txHash } = await contracts[accountIndex].methods.createClaims(amounts, secrets).send().wait());

      const amountSum = amounts.reduce((sum, a) => sum + a, 0n);
      await expectBalance(accountIndex, initialSupply - amountSum);
    }

    // Transaction 2
    {
      logger(`Account 1 claims note 0...`);
      const accountIndex = 1;
      const claim = claims[0];
      await expectBalance(accountIndex, 0n);

      await claimToken(accountIndex, claim, txHash);

      await expectBalance(accountIndex, claim.amount);

      logger(`Fails to claim note 0 again...`);
      await expect(claimToken(accountIndex, claim, txHash)).rejects.toThrow();
    }

    // Transaction 3
    {
      logger(`Account 2 claims note 1...`);
      const accountIndex = 2;
      const claim = claims[1];
      await expectBalance(accountIndex, 0n);

      await claimToken(accountIndex, claim, txHash);

      await expectBalance(accountIndex, claim.amount);

      logger(`Fails to claim note 1 again...`);
      await expect(claimToken(accountIndex, claim, txHash)).rejects.toThrow();

      logger(`Fails to claim note 0...`);
      await expect(claimToken(accountIndex, claims[0], txHash)).rejects.toThrow();
    }

    // Transaction 4
    {
      logger(`Account 1 claims note 2...`);
      const accountIndex = 1;
      const claim0 = claims[0];
      const claim2 = claims[2];
      expect(claim2.preimage).toEqual(claim0.preimage);

      await expectBalance(accountIndex, claim0.amount);

      // Claim 2 has the same preimage as claim 0.
      // `getNoteNonces` will return 2 nonces. And we need to use nonce 1 to spend the duplicated claim.
      const nonceIndex = 1;
      await claimToken(accountIndex, claim2, txHash, nonceIndex);

      await expectBalance(accountIndex, claim0.amount + claim2.amount);
    }
  }, 100_000);
});
