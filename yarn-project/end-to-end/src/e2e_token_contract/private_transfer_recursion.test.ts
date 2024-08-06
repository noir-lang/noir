import { BatchCall, EventType, Fr } from '@aztec/aztec.js';
import { TokenContract } from '@aztec/noir-contracts.js';

import { TokenContractTest } from './token_contract_test.js';

describe('e2e_token_contract private transfer recursion', () => {
  const t = new TokenContractTest('odd_transfer_private');
  let { asset, accounts, wallets } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.setup();
    ({ asset, accounts, wallets } = t);
  });

  afterAll(async () => {
    await t.teardown();
  });

  async function mintNotes(noteAmounts: bigint[]): Promise<bigint> {
    // Mint all notes, 4 at a time
    for (let mintedNotes = 0; mintedNotes < noteAmounts.length; mintedNotes += 4) {
      const toMint = noteAmounts.slice(mintedNotes, mintedNotes + 4); // We mint 4 notes at a time
      const actions = toMint.map(amt => asset.methods.privately_mint_private_note(amt).request());
      await new BatchCall(wallets[0], actions).send().wait();
    }

    return noteAmounts.reduce((prev, curr) => prev + curr, 0n);
  }

  it('transfer full balance', async () => {
    // We insert 16 notes, which is large enough to guarantee that the token will need to do two recursive calls to
    // itself to consume them all (since it retrieves 2 notes on the first pass and 8 in each subsequent pass).
    const totalNotes = 16;
    const totalBalance = await mintNotes(Array(totalNotes).fill(10n));
    const tx = await asset.methods.transfer(accounts[1].address, totalBalance).send().wait({ debug: true });

    // We should have nullified all notes, plus an extra nullifier for the transaction
    expect(tx.debugInfo?.nullifiers.length).toBe(totalNotes + 1);
    // We should have created a single new note, for the recipient
    expect(tx.debugInfo?.noteHashes.length).toBe(1);

    const events = await wallets[1].getEvents(EventType.Encrypted, TokenContract.events.Transfer, tx.blockNumber!, 1);

    expect(events[0]).toEqual({
      from: accounts[0].address,
      to: accounts[1].address,
      amount: new Fr(totalBalance),
    });
  });

  it('transfer less than full balance and get change', async () => {
    const noteAmounts = [10n, 10n, 10n, 10n];
    const expectedChange = 3n; // This will result in one of the notes being partially used

    const totalBalance = await mintNotes(noteAmounts);
    const toSend = totalBalance - expectedChange;

    const tx = await asset.methods.transfer(accounts[1].address, toSend).send().wait({ debug: true });

    // We should have nullified all notes, plus an extra nullifier for the transaction
    expect(tx.debugInfo?.nullifiers.length).toBe(noteAmounts.length + 1);
    // We should have created two new notes, one for the recipient and one for the sender (with the change)
    expect(tx.debugInfo?.noteHashes.length).toBe(2);

    const senderBalance = await asset.methods.balance_of_private(accounts[0].address).simulate();
    expect(senderBalance).toEqual(expectedChange);

    const events = await wallets[1].getEvents(EventType.Encrypted, TokenContract.events.Transfer, tx.blockNumber!, 1);

    expect(events[0]).toEqual({
      from: accounts[0].address,
      to: accounts[1].address,
      amount: new Fr(toSend),
    });
  });

  describe('failure cases', () => {
    it('transfer more than balance', async () => {
      const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();

      const amount = balance0 + 1n;
      expect(amount).toBeGreaterThan(0n);

      await expect(asset.methods.transfer(accounts[1].address, amount).simulate()).rejects.toThrow(
        'Assertion failed: Balance too low',
      );
    });
  });
});
