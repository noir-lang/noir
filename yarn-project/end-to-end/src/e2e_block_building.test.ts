import {
  BatchCall,
  ContractDeployer,
  ContractFunctionInteraction,
  Fr,
  Wallet,
  isContractDeployed,
} from '@aztec/aztec.js';
import { CircuitsWasm } from '@aztec/circuits.js';
import { pedersenPlookupCommitInputs } from '@aztec/circuits.js/barretenberg';
import { DebugLogger } from '@aztec/foundation/log';
import { TestContractAbi } from '@aztec/noir-contracts/artifacts';
import { TestContract, TokenContract } from '@aztec/noir-contracts/types';
import { PXE, TxStatus } from '@aztec/types';

import times from 'lodash.times';

import { setup } from './fixtures/utils.js';

describe('e2e_block_building', () => {
  let pxe: PXE;
  let logger: DebugLogger;
  let owner: Wallet;
  let minter: Wallet;
  let teardown: () => Promise<void>;

  describe('multi-txs block', () => {
    const abi = TestContractAbi;

    beforeAll(async () => {
      ({
        teardown,
        pxe,
        logger,
        wallets: [owner, minter],
      } = await setup(2));
    }, 100_000);

    afterAll(() => teardown());

    it('assembles a block with multiple txs', async () => {
      // Assemble N contract deployment txs
      // We need to create them sequentially since we cannot have parallel calls to a circuit
      const TX_COUNT = 8;
      const deployer = new ContractDeployer(abi, owner);
      const methods = times(TX_COUNT, () => deployer.deploy());

      for (const i in methods) {
        await methods[i].create({ contractAddressSalt: new Fr(BigInt(i + 1)) });
        await methods[i].simulate({});
      }

      // Send them simultaneously to be picked up by the sequencer
      const txs = await Promise.all(methods.map(method => method.send()));
      logger(`Txs sent with hashes: `);
      for (const tx of txs) logger(` ${await tx.getTxHash()}`);

      // Await txs to be mined and assert they are all mined on the same block
      const receipts = await Promise.all(txs.map(tx => tx.wait()));
      expect(receipts.map(r => r.status)).toEqual(times(TX_COUNT, () => TxStatus.MINED));
      expect(receipts.map(r => r.blockNumber)).toEqual(times(TX_COUNT, () => receipts[0].blockNumber));

      // Assert all contracts got deployed
      const areDeployed = await Promise.all(receipts.map(r => isContractDeployed(pxe, r.contractAddress!)));
      expect(areDeployed).toEqual(times(TX_COUNT, () => true));
    }, 60_000);

    it('can call public function from different tx in same block', async () => {
      // Deploy a contract in the first transaction
      // In the same block, call a public method on the contract
      const deployer = TokenContract.deploy(owner, owner.getCompleteAddress());
      await deployer.create();

      // We can't use `TokenContract.at` to call a function because it checks the contract is deployed
      // but we are in the same block as the deployment transaction
      const callInteraction = new ContractFunctionInteraction(
        owner,
        deployer.completeAddress!.address,
        TokenContract.abi.functions.find(x => x.name === 'set_minter')!,
        [minter.getCompleteAddress(), true],
      );

      await deployer.simulate({});
      await callInteraction.simulate({
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

  // Regressions for https://github.com/AztecProtocol/aztec-packages/issues/2502
  describe('double-spends on the same block', () => {
    let contract: TestContract;
    let teardown: () => Promise<void>;

    beforeAll(async () => {
      ({ teardown, pxe, logger, wallet: owner } = await setup(1));
      contract = await TestContract.deploy(owner).send().deployed();
    }, 100_000);

    afterAll(() => teardown());

    it('drops tx with private nullifier already emitted on the same block', async () => {
      const nullifier = Fr.random();
      const calls = times(2, () => contract.methods.emit_nullifier(nullifier));
      for (const call of calls) await call.simulate();
      const [tx1, tx2] = calls.map(call => call.send());
      await tx1.wait();
      await expect(tx2.wait()).rejects.toThrowError(/dropped/);
    }, 30_000);

    it('drops tx with public nullifier already emitted on the same block', async () => {
      const secret = Fr.random();
      const calls = times(2, () => contract.methods.createNullifierPublic(140n, secret));
      for (const call of calls) await call.simulate();
      const [tx1, tx2] = calls.map(call => call.send());
      await tx1.wait();
      await expect(tx2.wait()).rejects.toThrowError(/dropped/);
    }, 30_000);

    it('drops tx with two equal nullifiers', async () => {
      const nullifier = Fr.random();
      const calls = times(2, () => contract.methods.emit_nullifier(nullifier).request());
      await expect(new BatchCall(owner, calls).send().wait()).rejects.toThrowError(/dropped/);
    });

    it('drops tx with private nullifier already emitted from public on the same block', async () => {
      const secret = Fr.random();
      // See yarn-project/acir-simulator/src/public/index.test.ts 'Should be able to create a nullifier from the public context'
      const emittedPublicNullifier = pedersenPlookupCommitInputs(
        await CircuitsWasm.get(),
        [new Fr(140), secret].map(a => a.toBuffer()),
      );

      const calls = [
        contract.methods.createNullifierPublic(140n, secret),
        contract.methods.emit_nullifier(emittedPublicNullifier),
      ];

      for (const call of calls) await call.simulate();
      const [tx1, tx2] = calls.map(call => call.send());
      await tx1.wait();
      await expect(tx2.wait()).rejects.toThrowError(/dropped/);
    });
  });
});
