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
} from '@aztec/aztec.js';
import { times } from '@aztec/foundation/collection';
import { pedersenHash } from '@aztec/foundation/crypto';
import { StatefulTestContractArtifact } from '@aztec/noir-contracts.js';
import { TestContract } from '@aztec/noir-contracts.js/Test';
import { TokenContract } from '@aztec/noir-contracts.js/Token';

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
    }, 100_000);

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
      logger(`Txs sent with hashes: `);
      for (const tx of txs) {
        logger(` ${await tx.getTxHash()}`);
      }

      // Await txs to be mined and assert they are all mined on the same block
      const receipts = await Promise.all(txs.map(tx => tx.wait()));
      expect(receipts.map(r => r.blockNumber)).toEqual(times(TX_COUNT, () => receipts[0].blockNumber));

      // Assert all contracts got deployed
      const isContractDeployed = async (address: AztecAddress) => !!(await pxe.getContractInstance(address));
      const areDeployed = await Promise.all(receipts.map(r => isContractDeployed(r.contract.address)));
      expect(areDeployed).toEqual(times(TX_COUNT, () => true));
    }, 60_000);

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
    }, 60_000);
  });

  describe('double-spends', () => {
    let contract: TestContract;
    let teardown: () => Promise<void>;

    beforeAll(async () => {
      ({ teardown, pxe, logger, wallet: owner } = await setup(1));
      contract = await TestContract.deploy(owner).send().deployed();
      logger(`Test contract deployed at ${contract.address}`);
    }, 100_000);

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
      }, 30_000);

      it('drops tx with public nullifier already emitted on the same block', async () => {
        const secret = Fr.random();
        const calls = times(2, () => contract.methods.create_nullifier_public(140n, secret));
        for (const call of calls) {
          await call.prove();
        }
        const [tx1, tx2] = calls.map(call => call.send());
        await expectXorTx(tx1, tx2);
      }, 30_000);

      it('drops tx with two equal nullifiers', async () => {
        const nullifier = Fr.random();
        const calls = times(2, () => contract.methods.emit_nullifier(nullifier).request());
        await expect(new BatchCall(owner, calls).send().wait()).rejects.toThrow(/dropped/);
      }, 30_000);

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
      }, 30_000);
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
