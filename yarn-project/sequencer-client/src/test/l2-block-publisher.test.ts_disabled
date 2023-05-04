import {
  Fr,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  KernelCircuitPublicInputs,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  PublicDataRead,
  PublicDataTransition,
  range,
} from '@aztec/circuits.js';
import { fr, makeNewContractData, makeProof } from '@aztec/circuits.js/factories';
import { EthereumRpc } from '@aztec/ethereum.js/eth_rpc';
import { WalletProvider } from '@aztec/ethereum.js/provider';
import { DecoderHelper, Rollup, UnverifiedDataEmitter } from '@aztec/l1-contracts';
import { MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';
import { beforeAll, describe, expect, it } from '@jest/globals';
import { default as levelup } from 'levelup';
import { SoloBlockBuilder } from '../block_builder/solo_block_builder.js';
import { createMemDown } from '../block_builder/solo_block_builder.test.js';
import { getVerificationKeys, makePublicTx } from '../index.js';
import { EmptyRollupProver } from '../prover/empty.js';
import { EthereumjsTxSender } from '../publisher/ethereumjs-tx-sender.js';
import { L1Publisher } from '../publisher/l1-publisher.js';
import {
  makeEmptyProcessedTx as makeEmptyProcessedTxFromHistoricTreeRoots,
  makeProcessedTx,
} from '../sequencer/processed_tx.js';
import { getCombinedHistoricTreeRoots } from '../sequencer/utils.js';
import { WasmRollupCircuitSimulator } from '../simulator/rollup.js';
import { hexStringToBuffer } from '../utils.js';

// Accounts 4 and 5 of Anvil default startup with mnemonic: 'test test test test test test test test test test test junk'
const sequencerPK = '0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a';
const deployerPK = '0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba';
const anvilHost = process.env.ANVIL_HOST ?? 'http://127.0.0.1:8545';
const chainId = 31337;

describe.skip('L1Publisher integration', () => {
  let decoderHelper: DecoderHelper;
  let rollup: Rollup;
  let unverifiedDataEmitter: UnverifiedDataEmitter;
  let ethRpc: EthereumRpc;
  let publisher: L1Publisher;
  let l2Proof: Buffer;

  let builder: SoloBlockBuilder;
  let builderDb: MerkleTreeOperations;

  beforeAll(async () => {
    ({ ethRpc, decoderHelper, rollup, unverifiedDataEmitter } = await deployRollup());

    builderDb = await MerkleTrees.new(levelup(createMemDown())).then(t => t.asLatest());
    const vks = getVerificationKeys();
    const simulator = await WasmRollupCircuitSimulator.new();
    const prover = new EmptyRollupProver();
    builder = new SoloBlockBuilder(builderDb, vks, simulator, prover);

    l2Proof = Buffer.alloc(0);

    publisher = new L1Publisher(
      new EthereumjsTxSender({
        rpcUrl: anvilHost,
        chainId,
        requiredConfirmations: 1,
        rollupContract: rollup.address,
        unverifiedDataEmitterContract: unverifiedDataEmitter.address,
        publisherPrivateKey: hexStringToBuffer(sequencerPK),
      }),
      {
        retryIntervalMs: 100,
      },
    );
  }, 60_000);

  const makeEmptyProcessedTx = async () => {
    const historicTreeRoots = await getCombinedHistoricTreeRoots(builderDb);
    return makeEmptyProcessedTxFromHistoricTreeRoots(historicTreeRoots);
  };

  const makeContractDeployProcessedTx = async (seed = 0x1) => {
    const tx = await makeEmptyProcessedTx();
    tx.data.end.newContracts = [makeNewContractData(seed + 0x1000)];
    return tx;
  };

  const makePrivateProcessedTx = async (seed = 0x1) => {
    const tx = await makeEmptyProcessedTx();
    tx.data.end.newCommitments = range(KERNEL_NEW_COMMITMENTS_LENGTH, seed + 0x100).map(fr);
    tx.data.end.newNullifiers = range(KERNEL_NEW_NULLIFIERS_LENGTH, seed + 0x200).map(fr);
    return tx;
  };

  const makePublicCallProcessedTx = async (seed = 0x1) => {
    const publicTx = makePublicTx(seed);
    const kernelOutput = KernelCircuitPublicInputs.empty();
    kernelOutput.end.stateReads[0] = new PublicDataRead(fr(1), fr(0));
    kernelOutput.end.stateTransitions[0] = new PublicDataTransition(fr(2), fr(0), fr(12));
    kernelOutput.constants.historicTreeRoots = await getCombinedHistoricTreeRoots(builderDb);
    return await makeProcessedTx(publicTx, kernelOutput, makeProof());
  };

  it('Build 2 blocks of 4 txs building on each other', async () => {
    const stateInRollup_ = await rollup.methods.rollupStateHash().call();
    expect(hexStringToBuffer(stateInRollup_.toString())).toEqual(Buffer.alloc(32, 0));

    for (let i = 0; i < 2; i++) {
      // @todo Should have advanced txs as well instead of these simple transactions.
      // @todo Should have messages l1 -> l2

      const txsLeft = [await makePrivateProcessedTx(i + 1), await makePublicCallProcessedTx(i + 1)];
      const txsRight = [await makeContractDeployProcessedTx(i + 1), await makeEmptyProcessedTx()];
      const l1ToL2Messages = new Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n));

      // Actually build a block!
      const txs = [...txsLeft, ...txsRight];
      const [block] = await builder.buildL2Block(1 + i, txs, l1ToL2Messages);

      // Now we can use the block we built!
      const blockNumber = await ethRpc.blockNumber();
      await publisher.processL2Block(block);
      const logs = await rollup.getLogs('L2BlockProcessed', { fromBlock: blockNumber + 1 });
      expect(logs).toHaveLength(1);
      expect(logs[0].args.blockNum).toEqual(BigInt(i + 1));

      const ethTx = await ethRpc.getTransactionByHash(logs[0].transactionHash!);
      const expectedData = rollup.methods.process(l2Proof, block.encode()).encodeABI();
      expect(ethTx.input).toEqual(expectedData);

      const decodedHashes = await decoderHelper.methods.computeDiffRootAndMessagesHash(block.encode()).call();
      const decodedRes = await decoderHelper.methods.decode(block.encode()).call();
      const stateInRollup = await rollup.methods.rollupStateHash().call();

      // @note There seems to be something wrong here. The Bytes32 returned are actually strings :(
      expect(block.number).toEqual(Number(decodedRes[0]));
      expect(block.getStartStateHash()).toEqual(hexStringToBuffer(decodedRes[1].toString()));
      expect(block.getEndStateHash()).toEqual(hexStringToBuffer(decodedRes[2].toString()));
      expect(block.getEndStateHash()).toEqual(hexStringToBuffer(stateInRollup.toString()));
      expect(block.getPublicInputsHash().toBuffer()).toEqual(hexStringToBuffer(decodedRes[3].toString()));
      expect(block.getCalldataHash()).toEqual(hexStringToBuffer(decodedHashes[0].toString()));
      expect(block.getL1ToL2MessagesHash()).toEqual(hexStringToBuffer(decodedHashes[1].toString()));

      // @todo Broken if making two blocks in a row...
      return;
    }
  }, 60_000);
});

async function deployRollup() {
  // Set up client
  const provider = WalletProvider.fromHost(anvilHost);
  provider.addAccount(hexStringToBuffer(deployerPK));
  provider.addAccount(hexStringToBuffer(sequencerPK));
  const [sequencer, deployer] = provider.getAccounts();
  const ethRpc = new EthereumRpc(provider);

  // Deploy DecodeHelper, Rollup and unverifiedDataEmitter contracts
  const decoderHelper = new DecoderHelper(ethRpc, undefined, { from: deployer, gas: 1e6 });
  await decoderHelper.deploy().send().getReceipt();

  const deployedRollup = new Rollup(ethRpc, undefined, { from: deployer, gas: 1e6 });
  await deployedRollup.deploy().send().getReceipt();

  const deployedUnverifiedDataEmitter = new UnverifiedDataEmitter(ethRpc, undefined, { from: deployer, gas: 1e6 });
  await deployedUnverifiedDataEmitter.deploy().send().getReceipt();

  // Create new instance so we can attach the sequencer as sender
  const rollup = new Rollup(ethRpc, deployedRollup.address, { from: sequencer });
  const unverifiedDataEmitter = new UnverifiedDataEmitter(ethRpc, deployedUnverifiedDataEmitter.address, {
    from: sequencer,
  });

  return { decoderHelper, rollup, deployer, unverifiedDataEmitter, sequencer, ethRpc };
}
