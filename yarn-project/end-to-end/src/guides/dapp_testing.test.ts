import { createSandbox } from '@aztec/aztec-sandbox';
import {
  AccountWallet,
  AztecRPC,
  CheatCodes,
  Fr,
  L2BlockL2Logs,
  computeMessageSecretHash,
  createAccount,
  createAztecRpcClient,
  getSandboxAccountsWallets,
  waitForSandbox,
} from '@aztec/aztec.js';
import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { TestContract, TokenContract } from '@aztec/noir-contracts/types';

const { SANDBOX_URL = 'http://localhost:8080', ETHEREUM_HOST = 'http://localhost:8545' } = process.env;

describe('guides/dapp/testing', () => {
  describe('on in-proc sandbox', () => {
    describe('private token contract', () => {
      let rpc: AztecRPC;
      let stop: () => Promise<void>;
      let owner: AccountWallet;
      let recipient: AccountWallet;
      let token: TokenContract;

      beforeAll(async () => {
        // docs:start:in-proc-sandbox
        ({ rpcServer: rpc, stop } = await createSandbox());
        // docs:end:in-proc-sandbox
        owner = await createAccount(rpc);
        recipient = await createAccount(rpc);
        token = await TokenContract.deploy(owner).send().deployed();
        await token.methods._initialize({ address: owner.getAddress() }).send().wait();
      }, 60_000);

      // docs:start:stop-in-proc-sandbox
      afterAll(() => stop());
      // docs:end:stop-in-proc-sandbox

      it('increases recipient funds on mint', async () => {
        expect(await token.methods.balance_of_private({ address: recipient.getAddress() }).view()).toEqual(0n);
        const secret = Fr.random();
        const secretHash = await computeMessageSecretHash(secret);
        await token.methods.mint_private(20n, secretHash).send().wait();
        await token.methods.redeem_shield({ address: recipient.getAddress() }, 20n, secret).send().wait();
        expect(await token.methods.balance_of_private({ address: recipient.getAddress() }).view()).toEqual(20n);
      });
    });
  });

  describe('on local sandbox', () => {
    beforeAll(async () => {
      const rpc = createAztecRpcClient(SANDBOX_URL);
      await waitForSandbox(rpc);
    });

    // docs:start:sandbox-example
    describe('private token contract', () => {
      let rpc: AztecRPC;
      let owner: AccountWallet;
      let recipient: AccountWallet;
      let token: TokenContract;

      beforeEach(async () => {
        rpc = createAztecRpcClient(SANDBOX_URL);
        owner = await createAccount(rpc);
        recipient = await createAccount(rpc);
        token = await TokenContract.deploy(owner).send().deployed();
        await token.methods._initialize({ address: owner.getAddress() }).send().wait();
      }, 30_000);

      it('increases recipient funds on mint', async () => {
        expect(await token.methods.balance_of_private({ address: recipient.getAddress() }).view()).toEqual(0n);
        const secret = Fr.random();
        const secretHash = await computeMessageSecretHash(secret);
        await token.methods.mint_private(20n, secretHash).send().wait();
        await token.methods.redeem_shield({ address: recipient.getAddress() }, 20n, secret).send().wait();
        expect(await token.methods.balance_of_private({ address: recipient.getAddress() }).view()).toEqual(20n);
      });
    });
    // docs:end:sandbox-example

    describe('private token contract with initial accounts', () => {
      let rpc: AztecRPC;
      let owner: AccountWallet;
      let recipient: AccountWallet;
      let token: TokenContract;

      beforeEach(async () => {
        // docs:start:use-existing-wallets
        rpc = createAztecRpcClient(SANDBOX_URL);
        [owner, recipient] = await getSandboxAccountsWallets(rpc);
        token = await TokenContract.deploy(owner).send().deployed();
        await token.methods._initialize({ address: owner.getAddress() }).send().wait();
        // docs:end:use-existing-wallets
      }, 30_000);

      it('increases recipient funds on mint', async () => {
        expect(await token.methods.balance_of_private({ address: recipient.getAddress() }).view()).toEqual(0n);
        const secret = Fr.random();
        const secretHash = await computeMessageSecretHash(secret);
        await token.methods.mint_private(20n, secretHash).send().wait();
        await token.methods.redeem_shield({ address: recipient.getAddress() }, 20n, secret).send().wait();
        expect(await token.methods.balance_of_private({ address: recipient.getAddress() }).view()).toEqual(20n);
      });
    });

    describe('cheats', () => {
      let rpc: AztecRPC;
      let owner: AccountWallet;
      let testContract: TestContract;
      let cheats: CheatCodes;

      beforeAll(async () => {
        rpc = createAztecRpcClient(SANDBOX_URL);
        owner = await createAccount(rpc);
        testContract = await TestContract.deploy(owner).send().deployed();
        cheats = await CheatCodes.create(ETHEREUM_HOST, rpc);
      }, 30_000);

      it('warps time to 1h into the future', async () => {
        // docs:start:warp
        const newTimestamp = Math.floor(Date.now() / 1000) + 60 * 60 * 24;
        await cheats.aztec.warp(newTimestamp);
        await testContract.methods.isTimeEqual(newTimestamp).send().wait();
        // docs:end:warp
      });
    });

    describe('assertions', () => {
      let rpc: AztecRPC;
      let owner: AccountWallet;
      let recipient: AccountWallet;
      let testContract: TestContract;
      let token: TokenContract;
      let cheats: CheatCodes;
      let ownerSlot: Fr;

      beforeAll(async () => {
        rpc = createAztecRpcClient(SANDBOX_URL);
        owner = await createAccount(rpc);
        recipient = await createAccount(rpc);
        testContract = await TestContract.deploy(owner).send().deployed();
        token = await TokenContract.deploy(owner).send().deployed();
        await token.methods._initialize({ address: owner.getAddress() }).send().wait();
        const secret = Fr.random();
        const secretHash = await computeMessageSecretHash(secret);
        await token.methods.mint_private(100n, secretHash).send().wait();
        await token.methods.redeem_shield({ address: owner.getAddress() }, 100n, secret).send().wait();

        // docs:start:calc-slot
        cheats = await CheatCodes.create(ETHEREUM_HOST, rpc);
        // The balances mapping is defined on storage slot 3 and is indexed by user address
        ownerSlot = cheats.aztec.computeSlotInMap(3n, owner.getAddress());
        // docs:end:calc-slot
      }, 60_000);

      it('checks private storage', async () => {
        // docs:start:private-storage
        const notes = await rpc.getPrivateStorageAt(owner.getAddress(), token.address, ownerSlot);
        const values = notes.map(note => note.items[0]);
        const balance = values.reduce((sum, current) => sum + current.toBigInt(), 0n);
        expect(balance).toEqual(100n);
        // docs:end:private-storage
      });

      it('checks public storage', async () => {
        // docs:start:public-storage
        await token.methods.mint_public({ address: owner.getAddress() }, 100n).send().wait();
        const ownerPublicBalanceSlot = cheats.aztec.computeSlotInMap(6n, owner.getAddress());
        const balance = await rpc.getPublicStorageAt(token.address, ownerPublicBalanceSlot);
        expect(toBigIntBE(balance!)).toEqual(100n);
        // docs:end:public-storage
      });

      it('checks unencrypted logs, [Kinda broken with current implementation]', async () => {
        // docs:start:unencrypted-logs
        const value = Fr.fromString('ef'); // Only 1 bytes will make its way in there :( so no larger stuff
        const tx = await testContract.methods.emit_unencrypted(value).send().wait();
        const logs = await rpc.getUnencryptedLogs(tx.blockNumber!, 1);
        const log = L2BlockL2Logs.unrollLogs(logs)[0];
        expect(Fr.fromBuffer(log)).toEqual(value);
        // docs:end:unencrypted-logs
      });

      it('asserts a local transaction simulation fails by calling simulate', async () => {
        // docs:start:local-tx-fails
        const call = token.methods.transfer(
          { address: owner.getAddress() },
          { address: recipient.getAddress() },
          200n,
          0,
        );
        await expect(call.simulate()).rejects.toThrowError(/Balance too low/);
        // docs:end:local-tx-fails
      });

      it('asserts a local transaction simulation fails by calling send', async () => {
        // docs:start:local-tx-fails-send
        const call = token.methods.transfer(
          { address: owner.getAddress() },
          { address: recipient.getAddress() },
          200n,
          0,
        );
        await expect(call.send().wait()).rejects.toThrowError(/Balance too low/);
        // docs:end:local-tx-fails-send
      });

      it('asserts a transaction is dropped', async () => {
        // docs:start:tx-dropped
        const call1 = token.methods.transfer(
          { address: owner.getAddress() },
          { address: recipient.getAddress() },
          80n,
          0,
        );
        const call2 = token.methods.transfer(
          { address: owner.getAddress() },
          { address: recipient.getAddress() },
          50n,
          0,
        );

        await call1.simulate();
        await call2.simulate();

        await call1.send().wait();
        await expect(call2.send().wait()).rejects.toThrowError(/dropped/);
        // docs:end:tx-dropped
      });

      it('asserts a simulation for a public function call fails', async () => {
        // docs:start:local-pub-fails
        const call = token.methods.transfer_public(
          { address: owner.getAddress() },
          { address: recipient.getAddress() },
          1000n,
          0,
        );
        await expect(call.simulate()).rejects.toThrowError(/Underflow/);
        // docs:end:local-pub-fails
      });

      it('asserts a transaction with a failing public call is dropped (until we get public reverts)', async () => {
        // docs:start:pub-dropped
        const call = token.methods.transfer_public(
          { address: owner.getAddress() },
          { address: recipient.getAddress() },
          1000n,
          0,
        );
        await expect(call.send({ skipPublicSimulation: true }).wait()).rejects.toThrowError(/dropped/);
        // docs:end:pub-dropped
      });
    });
  });
});
