import { createSandbox } from '@aztec/aztec-sandbox';
import {
  AccountWallet,
  AztecRPC,
  CheatCodes,
  Fr,
  L2BlockL2Logs,
  createAccount,
  createAztecRpcClient,
  getSandboxAccountsWallets,
  waitForSandbox,
} from '@aztec/aztec.js';
import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { NativeTokenContract, PrivateTokenContract, TestContract } from '@aztec/noir-contracts/types';

describe('guides/dapp/testing', () => {
  describe('on in-proc sandbox', () => {
    describe('private token contract', () => {
      let rpc: AztecRPC;
      let stop: () => Promise<void>;
      let owner: AccountWallet;
      let recipient: AccountWallet;
      let token: PrivateTokenContract;

      beforeAll(async () => {
        // docs:start:in-proc-sandbox
        ({ rpcServer: rpc, stop } = await createSandbox());
        // docs:end:in-proc-sandbox
        owner = await createAccount(rpc);
        recipient = await createAccount(rpc);
        token = await PrivateTokenContract.deploy(owner, 100n, owner.getAddress()).send().deployed();
      }, 60_000);

      // docs:start:stop-in-proc-sandbox
      afterAll(() => stop());
      // docs:end:stop-in-proc-sandbox

      it('increases recipient funds on transfer', async () => {
        expect(await token.methods.getBalance(recipient.getAddress()).view()).toEqual(0n);
        await token.methods.transfer(20n, recipient.getAddress()).send().wait();
        expect(await token.methods.getBalance(recipient.getAddress()).view()).toEqual(20n);
      });
    });
  });

  describe('on local sandbox', () => {
    beforeAll(async () => {
      const { SANDBOX_URL = 'http://localhost:8080' } = process.env;
      const rpc = createAztecRpcClient(SANDBOX_URL);
      await waitForSandbox(rpc);
    });

    // docs:start:sandbox-example
    describe('private token contract', () => {
      const { SANDBOX_URL = 'http://localhost:8080' } = process.env;

      let rpc: AztecRPC;
      let owner: AccountWallet;
      let recipient: AccountWallet;
      let token: PrivateTokenContract;

      beforeEach(async () => {
        rpc = createAztecRpcClient(SANDBOX_URL);
        owner = await createAccount(rpc);
        recipient = await createAccount(rpc);
        token = await PrivateTokenContract.deploy(owner, 100n, owner.getAddress()).send().deployed();
      }, 30_000);

      it('increases recipient funds on transfer', async () => {
        expect(await token.methods.getBalance(recipient.getAddress()).view()).toEqual(0n);
        await token.methods.transfer(20n, recipient.getAddress()).send().wait();
        expect(await token.methods.getBalance(recipient.getAddress()).view()).toEqual(20n);
      });
    });
    // docs:end:sandbox-example

    describe('private token contract with initial accounts', () => {
      const { SANDBOX_URL = 'http://localhost:8080' } = process.env;

      let rpc: AztecRPC;
      let owner: AccountWallet;
      let recipient: AccountWallet;
      let token: PrivateTokenContract;

      beforeEach(async () => {
        // docs:start:use-existing-wallets
        rpc = createAztecRpcClient(SANDBOX_URL);
        [owner, recipient] = await getSandboxAccountsWallets(rpc);
        token = await PrivateTokenContract.deploy(owner, 100n, owner.getAddress()).send().deployed();
        // docs:end:use-existing-wallets
      }, 30_000);

      it('increases recipient funds on transfer', async () => {
        expect(await token.methods.getBalance(recipient.getAddress()).view()).toEqual(0n);
        await token.methods.transfer(20n, recipient.getAddress()).send().wait();
        expect(await token.methods.getBalance(recipient.getAddress()).view()).toEqual(20n);
      });
    });

    describe('cheats', () => {
      const { SANDBOX_URL = 'http://localhost:8080', ETHEREUM_HOST = 'http://localhost:8545' } = process.env;

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
      const { SANDBOX_URL = 'http://localhost:8080', ETHEREUM_HOST = 'http://localhost:8545' } = process.env;

      let rpc: AztecRPC;
      let owner: AccountWallet;
      let recipient: AccountWallet;
      let token: PrivateTokenContract;
      let nativeToken: NativeTokenContract;
      let cheats: CheatCodes;
      let ownerSlot: Fr;

      beforeAll(async () => {
        rpc = createAztecRpcClient(SANDBOX_URL);
        owner = await createAccount(rpc);
        recipient = await createAccount(rpc);
        token = await PrivateTokenContract.deploy(owner, 100n, owner.getAddress()).send().deployed();
        nativeToken = await NativeTokenContract.deploy(owner, 100n, owner.getAddress()).send().deployed();

        // docs:start:calc-slot
        cheats = await CheatCodes.create(ETHEREUM_HOST, rpc);
        // The balances mapping is defined on storage slot 1 and is indexed by user address
        ownerSlot = cheats.aztec.computeSlotInMap(1n, owner.getAddress());
        // docs:end:calc-slot
      }, 30_000);

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
        await nativeToken.methods.owner_mint_pub(owner.getAddress(), 100n).send().wait();
        const ownerPublicBalanceSlot = cheats.aztec.computeSlotInMap(4n, owner.getAddress());
        const balance = await rpc.getPublicStorageAt(nativeToken.address, ownerPublicBalanceSlot);
        expect(toBigIntBE(balance!)).toEqual(100n);
        // docs:end:public-storage
      });

      it('checks unencrypted logs', async () => {
        // docs:start:unencrypted-logs
        const tx = await nativeToken.methods.owner_mint_pub(owner.getAddress(), 100n).send().wait();
        const logs = await rpc.getUnencryptedLogs(tx.blockNumber!, 1);
        const textLogs = L2BlockL2Logs.unrollLogs(logs).map(log => log.toString('ascii'));
        expect(textLogs).toEqual(['Coins minted']);
        // docs:end:unencrypted-logs
      });

      it('asserts a local transaction simulation fails by calling simulate', async () => {
        // docs:start:local-tx-fails
        const call = token.methods.transfer(200n, recipient.getAddress());
        await expect(call.simulate()).rejects.toThrowError(/Balance too low/);
        // docs:end:local-tx-fails
      });

      it('asserts a local transaction simulation fails by calling send', async () => {
        // docs:start:local-tx-fails-send
        const call = token.methods.transfer(200n, recipient.getAddress());
        await expect(call.send().wait()).rejects.toThrowError(/Balance too low/);
        // docs:end:local-tx-fails-send
      });

      it('asserts a transaction is dropped', async () => {
        // docs:start:tx-dropped
        const call1 = token.methods.transfer(80n, recipient.getAddress());
        const call2 = token.methods.transfer(50n, recipient.getAddress());

        await call1.simulate();
        await call2.simulate();

        await call1.send().wait();
        await expect(call2.send().wait()).rejects.toThrowError(/dropped/);
        // docs:end:tx-dropped
      });

      it('asserts a simulation for a public function call fails', async () => {
        // docs:start:local-pub-fails
        const call = nativeToken.methods.transfer_pub(recipient.getAddress(), 1000n);
        await expect(call.simulate()).rejects.toThrowError(/Balance too low/);
        // docs:end:local-pub-fails
      });

      it('asserts a transaction with a failing public call is dropped (until we get public reverts)', async () => {
        // docs:start:pub-dropped
        const call = nativeToken.methods.transfer_pub(recipient.getAddress(), 1000n);
        await expect(call.send({ skipPublicSimulation: true }).wait()).rejects.toThrowError(/dropped/);
        // docs:end:pub-dropped
      });
    });
  });
});
