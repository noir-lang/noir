// docs:start:imports
import { createAccount, getDeployedTestAccountsWallets } from '@aztec/accounts/testing';
import {
  type AccountWallet,
  CheatCodes,
  ExtendedNote,
  Fr,
  Note,
  type PXE,
  TxStatus,
  computeSecretHash,
  createPXEClient,
  waitForPXE,
} from '@aztec/aztec.js';
// docs:end:imports
// docs:start:import_contract
import { TestContract } from '@aztec/noir-contracts.js/Test';
// docs:end:import_contract
import { TokenContract } from '@aztec/noir-contracts.js/Token';

import { U128_UNDERFLOW_ERROR } from '../fixtures/fixtures.js';

const { PXE_URL = 'http://localhost:8080', ETHEREUM_HOST = 'http://localhost:8545' } = process.env;

describe('guides/dapp/testing', () => {
  describe('on local sandbox', () => {
    beforeAll(async () => {
      // docs:start:create_pxe_client
      const pxe = createPXEClient(PXE_URL);
      await waitForPXE(pxe);
      // docs:end:create_pxe_client
    });

    describe('token contract', () => {
      let pxe: PXE;
      let owner: AccountWallet;
      let recipient: AccountWallet;
      let token: TokenContract;

      beforeEach(async () => {
        pxe = createPXEClient(PXE_URL);
        owner = await createAccount(pxe);
        recipient = await createAccount(pxe);
        token = await TokenContract.deploy(owner, owner.getCompleteAddress(), 'TokenName', 'TokenSymbol', 18)
          .send()
          .deployed();
      });

      it('increases recipient funds on mint', async () => {
        const recipientAddress = recipient.getAddress();
        expect(await token.methods.balance_of_private(recipientAddress).simulate()).toEqual(0n);

        const mintAmount = 20n;
        const secret = Fr.random();
        const secretHash = computeSecretHash(secret);
        const receipt = await token.methods.mint_private(mintAmount, secretHash).send().wait();

        const note = new Note([new Fr(mintAmount), secretHash]);
        const extendedNote = new ExtendedNote(
          note,
          recipientAddress,
          token.address,
          TokenContract.storage.pending_shields.slot,
          TokenContract.notes.TransparentNote.id,
          receipt.txHash,
        );
        await pxe.addNote(extendedNote);

        await token.methods.redeem_shield(recipientAddress, mintAmount, secret).send().wait();
        expect(await token.methods.balance_of_private(recipientAddress).simulate()).toEqual(20n);
      });
    });

    describe('token contract with initial accounts', () => {
      let pxe: PXE;
      let owner: AccountWallet;
      let recipient: AccountWallet;
      let token: TokenContract;

      beforeEach(async () => {
        // docs:start:use-existing-wallets
        pxe = createPXEClient(PXE_URL);
        [owner, recipient] = await getDeployedTestAccountsWallets(pxe);
        token = await TokenContract.deploy(owner, owner.getCompleteAddress(), 'TokenName', 'TokenSymbol', 18)
          .send()
          .deployed();
        // docs:end:use-existing-wallets
      });

      it('increases recipient funds on mint', async () => {
        expect(await token.methods.balance_of_private(recipient.getAddress()).simulate()).toEqual(0n);
        const recipientAddress = recipient.getAddress();
        const mintAmount = 20n;
        const secret = Fr.random();
        const secretHash = computeSecretHash(secret);
        const receipt = await token.methods.mint_private(mintAmount, secretHash).send().wait();

        const note = new Note([new Fr(mintAmount), secretHash]);
        const extendedNote = new ExtendedNote(
          note,
          recipientAddress,
          token.address,
          TokenContract.storage.pending_shields.slot,
          TokenContract.notes.TransparentNote.id,
          receipt.txHash,
        );
        await pxe.addNote(extendedNote);

        await token.methods.redeem_shield(recipientAddress, mintAmount, secret).send().wait();
        expect(await token.methods.balance_of_private(recipientAddress).simulate()).toEqual(20n);
      });
    });

    describe('cheats', () => {
      let pxe: PXE;
      let owner: AccountWallet;
      let testContract: TestContract;
      let cheats: CheatCodes;

      beforeAll(async () => {
        pxe = createPXEClient(PXE_URL);
        owner = await createAccount(pxe);
        testContract = await TestContract.deploy(owner).send().deployed();
        cheats = CheatCodes.create(ETHEREUM_HOST, pxe);
      });

      // TODO(@spalladino) Disabled due to flakiness after #7347. Note that warp is already tested in e2e_cheat_codes.
      it.skip('warps time to 1h into the future', async () => {
        // docs:start:warp
        const newTimestamp = Math.floor(Date.now() / 1000) + 60 * 60 * 24;
        await cheats.aztec.warp(newTimestamp);
        await testContract.methods.is_time_equal(newTimestamp).send().wait();
        // docs:end:warp
      });
    });

    describe('assertions', () => {
      let pxe: PXE;
      let owner: AccountWallet;
      let recipient: AccountWallet;
      let testContract: TestContract;
      let token: TokenContract;
      let cheats: CheatCodes;
      let ownerSlot: Fr;

      beforeAll(async () => {
        pxe = createPXEClient(PXE_URL);
        owner = await createAccount(pxe);
        recipient = await createAccount(pxe);
        testContract = await TestContract.deploy(owner).send().deployed();
        token = await TokenContract.deploy(owner, owner.getCompleteAddress(), 'TokenName', 'TokenSymbol', 18)
          .send()
          .deployed();

        const ownerAddress = owner.getAddress();
        const mintAmount = 100n;
        const secret = Fr.random();
        const secretHash = computeSecretHash(secret);
        const receipt = await token.methods.mint_private(100n, secretHash).send().wait();

        const note = new Note([new Fr(mintAmount), secretHash]);
        const extendedNote = new ExtendedNote(
          note,
          ownerAddress,
          token.address,
          TokenContract.storage.pending_shields.slot,
          TokenContract.notes.TransparentNote.id,
          receipt.txHash,
        );
        await pxe.addNote(extendedNote);

        await token.methods.redeem_shield(ownerAddress, 100n, secret).send().wait();

        // docs:start:calc-slot
        cheats = CheatCodes.create(ETHEREUM_HOST, pxe);
        // The balances mapping is indexed by user address
        ownerSlot = cheats.aztec.computeSlotInMap(TokenContract.storage.balances.slot, ownerAddress);
        // docs:end:calc-slot
      });

      it('checks private storage', async () => {
        // docs:start:private-storage
        const notes = await pxe.getIncomingNotes({
          owner: owner.getAddress(),
          contractAddress: token.address,
          storageSlot: ownerSlot,
        });
        const values = notes.map(note => note.note.items[0]);
        const balance = values.reduce((sum, current) => sum + current.toBigInt(), 0n);
        expect(balance).toEqual(100n);
        // docs:end:private-storage
      });

      it('checks public storage', async () => {
        // docs:start:public-storage
        await token.methods.mint_public(owner.getAddress(), 100n).send().wait();
        const ownerPublicBalanceSlot = cheats.aztec.computeSlotInMap(6n, owner.getAddress());
        const balance = await pxe.getPublicStorageAt(token.address, ownerPublicBalanceSlot);
        expect(balance.value).toEqual(100n);
        // docs:end:public-storage
      });

      it('checks unencrypted logs, [Kinda broken with current implementation]', async () => {
        // docs:start:unencrypted-logs
        const value = Fr.fromString('ef'); // Only 1 bytes will make its way in there :( so no larger stuff
        const tx = await testContract.methods.emit_unencrypted(value).send().wait();
        const filter = {
          fromBlock: tx.blockNumber!,
          limit: 1, // 1 log expected
        };
        const logs = (await pxe.getUnencryptedLogs(filter)).logs;
        expect(Fr.fromBuffer(logs[0].log.data)).toEqual(value);
        // docs:end:unencrypted-logs
      });

      it('asserts a local transaction simulation fails by calling simulate', async () => {
        // docs:start:local-tx-fails
        const call = token.methods.transfer(recipient.getAddress(), 200n);
        await expect(call.prove()).rejects.toThrow(/Balance too low/);
        // docs:end:local-tx-fails
      });

      it('asserts a local transaction simulation fails by calling send', async () => {
        // docs:start:local-tx-fails-send
        const call = token.methods.transfer(recipient.getAddress(), 200n);
        await expect(call.send().wait()).rejects.toThrow(/Balance too low/);
        // docs:end:local-tx-fails-send
      });

      it('asserts a transaction is dropped', async () => {
        // docs:start:tx-dropped
        const call1 = token.methods.transfer(recipient.getAddress(), 80n);
        const call2 = token.methods.transfer(recipient.getAddress(), 50n);

        await call1.prove();
        await call2.prove();

        await call1.send().wait();
        await expect(call2.send().wait()).rejects.toThrow(/dropped/);
        // docs:end:tx-dropped
      });

      it('asserts a simulation for a public function call fails', async () => {
        // docs:start:local-pub-fails
        const call = token.methods.transfer_public(owner.getAddress(), recipient.getAddress(), 1000n, 0);
        await expect(call.prove()).rejects.toThrow(U128_UNDERFLOW_ERROR);
        // docs:end:local-pub-fails
      });

      it('asserts a transaction with a failing public call is included (with no state changes)', async () => {
        // docs:start:pub-reverted
        const call = token.methods.transfer_public(owner.getAddress(), recipient.getAddress(), 1000n, 0);
        const receipt = await call.send({ skipPublicSimulation: true }).wait({ dontThrowOnRevert: true });
        expect(receipt.status).toEqual(TxStatus.APP_LOGIC_REVERTED);
        const ownerPublicBalanceSlot = cheats.aztec.computeSlotInMap(6n, owner.getAddress());
        const balance = await pxe.getPublicStorageAt(token.address, ownerPublicBalanceSlot);
        expect(balance.value).toEqual(100n);
        // docs:end:pub-reverted
      });
    });
  });
});
