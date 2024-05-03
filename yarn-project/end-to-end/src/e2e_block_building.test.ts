import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import {
  type AztecAddress,
  type AztecNode,
  BatchCall,
  ContractDeployer,
  ContractFunctionInteraction,
  type DebugLogger,
  Fr,
  type PXE,
  type SentTx,
  type TxReceipt,
  TxStatus,
  type Wallet,
  deriveKeys,
} from '@aztec/aztec.js';
import { times } from '@aztec/foundation/collection';
import { pedersenHash } from '@aztec/foundation/crypto';
import { StatefulTestContractArtifact } from '@aztec/noir-contracts.js';
import { TestContract } from '@aztec/noir-contracts.js/Test';
import { TokenContract } from '@aztec/noir-contracts.js/Token';

import { TaggedNote } from '../../circuit-types/src/logs/l1_note_payload/tagged_note.js';
import { setup } from './fixtures/utils.js';

describe('e2e_block_building', () => {
  let pxe: PXE;
  let logger: DebugLogger;
  let owner: Wallet;
  let minter: Wallet;
  let aztecNode: AztecNode;
  let teardown: () => Promise<void>;

  describe('multi-txs block', () => {
    const artifact = StatefulTestContractArtifact;

    beforeAll(async () => {
      ({
        teardown,
        pxe,
        logger,
        aztecNode,
        wallets: [owner, minter],
      } = await setup(2));
    });

    afterEach(() => aztecNode.setConfig({ minTxsPerBlock: 1 }));
    afterAll(() => teardown());

    it('assembles a block with multiple txs', async () => {
      // Assemble N contract deployment txs
      // We need to create them sequentially since we cannot have parallel calls to a circuit
      const TX_COUNT = 8;
      await aztecNode.setConfig({ minTxsPerBlock: TX_COUNT });
      const deployer = new ContractDeployer(artifact, owner);
      const methods = times(TX_COUNT, i => deployer.deploy(owner.getCompleteAddress().address, i));
      for (let i = 0; i < TX_COUNT; i++) {
        await methods[i].create({
          contractAddressSalt: new Fr(BigInt(i + 1)),
          skipClassRegistration: true,
          skipPublicDeployment: true,
        });
        await methods[i].prove({});
      }

      // Send them simultaneously to be picked up by the sequencer
      const txs = await Promise.all(methods.map(method => method.send()));
      logger.info(`Txs sent with hashes: `);
      for (const tx of txs) {
        logger.info(` ${await tx.getTxHash()}`);
      }

      // Await txs to be mined and assert they are all mined on the same block
      const receipts = await Promise.all(txs.map(tx => tx.wait()));
      expect(receipts.map(r => r.blockNumber)).toEqual(times(TX_COUNT, () => receipts[0].blockNumber));

      // Assert all contracts got deployed
      const isContractDeployed = async (address: AztecAddress) => !!(await pxe.getContractInstance(address));
      const areDeployed = await Promise.all(receipts.map(r => isContractDeployed(r.contract.address)));
      expect(areDeployed).toEqual(times(TX_COUNT, () => true));
    });

    it.skip('can call public function from different tx in same block', async () => {
      // Ensure both txs will land on the same block
      await aztecNode.setConfig({ minTxsPerBlock: 2 });

      // Deploy a contract in the first transaction
      // In the same block, call a public method on the contract
      const deployer = TokenContract.deploy(owner, owner.getCompleteAddress(), 'TokenName', 'TokenSymbol', 18);
      await deployer.create();

      // We can't use `TokenContract.at` to call a function because it checks the contract is deployed
      // but we are in the same block as the deployment transaction
      const callInteraction = new ContractFunctionInteraction(
        owner,
        deployer.getInstance().address,
        TokenContract.artifact.functions.find(x => x.name === 'set_minter')!,
        [minter.getCompleteAddress(), true],
      );

      await deployer.prove({});
      await callInteraction.prove({
        // we have to skip simulation of public calls simulation is done on individual transactions
        // and the tx deploying the contract might go in the same block as this one
        skipPublicSimulation: true,
      });

      const [deployTxReceipt, callTxReceipt] = await Promise.all([
        deployer.send().wait(),
        callInteraction.send({ skipPublicSimulation: true }).wait(),
      ]);

      expect(deployTxReceipt.blockNumber).toEqual(callTxReceipt.blockNumber);
    });
  });

  describe('double-spends', () => {
    let contract: TestContract;
    let teardown: () => Promise<void>;

    beforeAll(async () => {
      ({ teardown, pxe, logger, wallet: owner } = await setup(1));
      contract = await TestContract.deploy(owner).send().deployed();
      logger.info(`Test contract deployed at ${contract.address}`);
    });

    afterAll(() => teardown());

    // Regressions for https://github.com/AztecProtocol/aztec-packages/issues/2502
    describe('in the same block', () => {
      it('drops tx with private nullifier already emitted on the same block', async () => {
        const nullifier = Fr.random();
        const calls = times(2, () => contract.methods.emit_nullifier(nullifier));
        for (const call of calls) {
          await call.prove();
        }
        const [tx1, tx2] = calls.map(call => call.send());
        await expectXorTx(tx1, tx2);
      });

      it('drops tx with public nullifier already emitted on the same block', async () => {
        const secret = Fr.random();
        const calls = times(2, () => contract.methods.create_nullifier_public(140n, secret));
        for (const call of calls) {
          await call.prove();
        }
        const [tx1, tx2] = calls.map(call => call.send());
        await expectXorTx(tx1, tx2);
      });

      it('drops tx with two equal nullifiers', async () => {
        const nullifier = Fr.random();
        const calls = times(2, () => contract.methods.emit_nullifier(nullifier).request());
        await expect(new BatchCall(owner, calls).send().wait()).rejects.toThrow(/dropped/);
      });

      it('drops tx with private nullifier already emitted from public on the same block', async () => {
        const secret = Fr.random();
        // See yarn-project/simulator/src/public/index.test.ts 'Should be able to create a nullifier from the public context'
        const emittedPublicNullifier = pedersenHash([new Fr(140), secret]);

        const calls = [
          contract.methods.create_nullifier_public(140n, secret),
          contract.methods.emit_nullifier(emittedPublicNullifier),
        ];

        for (const call of calls) {
          await call.prove();
        }
        const [tx1, tx2] = calls.map(call => call.send());
        await expectXorTx(tx1, tx2);
      });
    });

    describe('across blocks', () => {
      it('drops a tx that tries to spend a nullifier already emitted on a previous block', async () => {
        const secret = Fr.random();
        const emittedPublicNullifier = pedersenHash([new Fr(140), secret]);

        await expect(contract.methods.create_nullifier_public(140n, secret).send().wait()).resolves.toEqual(
          expect.objectContaining({
            status: TxStatus.MINED,
          }),
        );

        await expect(contract.methods.emit_nullifier(emittedPublicNullifier).send().wait()).rejects.toThrow(/dropped/);
      });
    });
  });

  describe('logs in nested calls are ordered as expected', () => {
    // This test was originally writted for e2e_nested, but it was refactored
    // to not use TestContract.
    let testContract: TestContract;

    beforeEach(async () => {
      ({ teardown, pxe, logger, wallet: owner } = await setup(1));
      logger.info(`Deploying test contract`);
      testContract = await TestContract.deploy(owner).send().deployed();
    }, 30_000);

    it('calls a method with nested unencrypted logs', async () => {
      const tx = await testContract.methods.emit_unencrypted_logs_nested([1, 2, 3, 4, 5]).send().wait();
      const logs = (await pxe.getUnencryptedLogs({ txHash: tx.txHash })).logs.map(l => l.log);

      // First log should be contract address
      expect(logs[0].data).toEqual(testContract.address.toBuffer());

      // Second log should be array of fields
      let expectedBuffer = Buffer.concat([1, 2, 3, 4, 5].map(num => new Fr(num).toBuffer()));
      expect(logs[1].data.subarray(-32 * 5)).toEqual(expectedBuffer);

      // Third log should be string "test"
      expectedBuffer = Buffer.concat(
        ['t', 'e', 's', 't'].map(num => Buffer.concat([Buffer.alloc(31), Buffer.from(num)])),
      );
      expect(logs[2].data.subarray(-32 * 5)).toEqual(expectedBuffer);
    }, 60_000);

    it('calls a method with nested encrypted logs', async () => {
      // account setup
      const privateKey = new Fr(7n);
      const keys = deriveKeys(privateKey);
      const account = getSchnorrAccount(pxe, privateKey, keys.masterIncomingViewingSecretKey);
      await account.deploy().wait();
      const thisWallet = await account.getWallet();

      // call test contract
      const action = testContract.methods.emit_encrypted_logs_nested(10, thisWallet.getAddress());
      const tx = await action.prove();
      const rct = await action.send().wait();

      // compare logs
      expect(rct.status).toEqual('mined');
      const decryptedLogs = tx.encryptedLogs
        .unrollLogs()
        .map(l => TaggedNote.fromEncryptedBuffer(l.data, keys.masterIncomingViewingSecretKey));
      const notevalues = decryptedLogs.map(l => l?.notePayload.note.items[0]);
      expect(notevalues[0]).toEqual(new Fr(10));
      expect(notevalues[1]).toEqual(new Fr(11));
      expect(notevalues[2]).toEqual(new Fr(12));
    }, 30_000);
  });
});

/**
 * Checks that only one of the two provided transactions succeeds.
 * @param tx1 - A transaction.
 * @param tx2 - Another transaction.
 */
async function expectXorTx(tx1: SentTx, tx2: SentTx) {
  const receipts = await Promise.allSettled([tx1.wait(), tx2.wait()]);
  const succeeded = receipts.find((r): r is PromiseSettledResult<TxReceipt> => r.status === 'fulfilled');
  const failed = receipts.find((r): r is PromiseRejectedResult => r.status === 'rejected');

  expect(succeeded).toBeDefined();
  expect(failed).toBeDefined();
  expect((failed?.reason as Error).message).toMatch(/dropped/);
}
