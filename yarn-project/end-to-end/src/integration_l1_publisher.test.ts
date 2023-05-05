import { createMemDown, getConfigEnvVars } from '@aztec/aztec-node';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import {
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_L2_TO_L1_MSGS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  KernelCircuitPublicInputs,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  PublicDataTransition,
  STATE_TRANSITIONS_LENGTH,
  range,
} from '@aztec/circuits.js';
import { fr, makeNewContractData, makeProof } from '@aztec/circuits.js/factories';
import { createDebugLogger } from '@aztec/foundation/log';
import { DecoderHelperAbi, RollupAbi } from '@aztec/l1-artifacts';
import {
  EmptyRollupProver,
  L1Publisher,
  SoloBlockBuilder,
  WasmRollupCircuitSimulator,
  getCombinedHistoricTreeRoots,
  getL1Publisher,
  getVerificationKeys,
  makeEmptyProcessedTx as makeEmptyProcessedTxFromHistoricTreeRoots,
  makeProcessedTx,
  makePublicTx,
} from '@aztec/sequencer-client';
import { MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';
import { beforeEach, describe, expect, it } from '@jest/globals';
import { default as levelup } from 'levelup';
import {
  Address,
  Chain,
  GetContractReturnType,
  HttpTransport,
  PublicClient,
  createPublicClient,
  encodeFunctionData,
  getAbiItem,
  getAddress,
  getContract,
  http,
} from 'viem';
import { privateKeyToAccount } from 'viem/accounts';
import { foundry } from 'viem/chains';
import { deployL1Contracts } from './deploy_l1_contracts.js';

// Accounts 4 and 5 of Anvil default startup with mnemonic: 'test test test test test test test test test test test junk'
const sequencerPK = '0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a';
const deployerPK = '0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba';

const logger = createDebugLogger('aztec:integration_l1_publisher');

const config = getConfigEnvVars();

// @todo (Issue https://github.com/AztecProtocol/aztec-packages/issues/472) Figure out why l1 -> l2 messages are breaking >1 consecutive blocks for mixed blocks.
const numberOfConsecutiveBlocks = 1;

describe('L1Publisher integration', () => {
  let publicClient: PublicClient<HttpTransport, Chain>;

  let rollupAddress: Address;
  let unverifiedDataEmitterAddress: Address;
  let decoderHelperAddress: Address;

  let rollup: GetContractReturnType<typeof RollupAbi, PublicClient<HttpTransport, Chain>>;
  let decoderHelper: GetContractReturnType<typeof DecoderHelperAbi, PublicClient<HttpTransport, Chain>>;

  let publisher: L1Publisher;
  let l2Proof: Buffer;

  let builder: SoloBlockBuilder;
  let builderDb: MerkleTreeOperations;

  beforeEach(async () => {
    const deployerAccount = privateKeyToAccount(deployerPK);
    const {
      rollupAddress: rollupAddress_,
      unverifiedDataEmitterAddress: unverifiedDataEmitterAddress_,
      decoderHelperAddress: decoderHelperAddress_,
    } = await deployL1Contracts(config.rpcUrl, deployerAccount, logger, true);

    rollupAddress = getAddress(rollupAddress_.toString());
    unverifiedDataEmitterAddress = getAddress(unverifiedDataEmitterAddress_.toString());
    decoderHelperAddress = getAddress(decoderHelperAddress_!.toString());

    publicClient = createPublicClient({
      chain: foundry,
      transport: http(config.rpcUrl),
    });

    // Set up contract instances
    rollup = getContract({
      address: rollupAddress,
      abi: RollupAbi,
      publicClient,
    });
    decoderHelper = getContract({
      address: decoderHelperAddress!,
      abi: DecoderHelperAbi,
      publicClient,
    });

    builderDb = await MerkleTrees.new(levelup(createMemDown())).then(t => t.asLatest());
    const vks = getVerificationKeys();
    const simulator = await WasmRollupCircuitSimulator.new();
    const prover = new EmptyRollupProver();
    builder = new SoloBlockBuilder(builderDb, vks, simulator, prover);

    l2Proof = Buffer.alloc(0);

    publisher = getL1Publisher({
      rpcUrl: config.rpcUrl,
      chainId: config.chainId,
      requiredConfirmations: 1,
      rollupContract: EthAddress.fromString(rollupAddress),
      unverifiedDataEmitterContract: EthAddress.fromString(unverifiedDataEmitterAddress),
      publisherPrivateKey: hexStringToBuffer(sequencerPK),
      retryIntervalMs: 100,
    });
  }, 60_000);

  const makeEmptyProcessedTx = async () => {
    const historicTreeRoots = await getCombinedHistoricTreeRoots(builderDb);
    return makeEmptyProcessedTxFromHistoricTreeRoots(historicTreeRoots);
  };

  const makeBloatedProcessedTx = async (seed = 0x1) => {
    const publicTx = makePublicTx(seed);
    const kernelOutput = KernelCircuitPublicInputs.empty();
    kernelOutput.constants.historicTreeRoots = await getCombinedHistoricTreeRoots(builderDb);
    kernelOutput.end.stateTransitions = range(STATE_TRANSITIONS_LENGTH, seed + 0x500).map(
      i => new PublicDataTransition(fr(i), fr(0), fr(i + 10)),
    );

    const tx = await makeProcessedTx(publicTx, kernelOutput, makeProof());

    tx.data.end.newCommitments = range(KERNEL_NEW_COMMITMENTS_LENGTH, seed + 0x100).map(fr);
    tx.data.end.newNullifiers = range(KERNEL_NEW_NULLIFIERS_LENGTH, seed + 0x200).map(fr);
    tx.data.end.newL2ToL1Msgs = range(KERNEL_NEW_L2_TO_L1_MSGS_LENGTH, seed + 0x300).map(fr);
    tx.data.end.newContracts = [makeNewContractData(seed + 0x1000)];

    return tx;
  };

  it(`Build ${numberOfConsecutiveBlocks} blocks of 4 bloated txs building on each other`, async () => {
    const stateInRollup_ = await rollup.read.rollupStateHash();
    expect(hexStringToBuffer(stateInRollup_.toString())).toEqual(Buffer.alloc(32, 0));

    const blockNumber = await publicClient.getBlockNumber();

    for (let i = 0; i < numberOfConsecutiveBlocks; i++) {
      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 128 * i + 1 + 0x400).map(fr);
      const txs = [
        await makeBloatedProcessedTx(128 * i + 32),
        await makeBloatedProcessedTx(128 * i + 64),
        await makeBloatedProcessedTx(128 * i + 96),
        await makeBloatedProcessedTx(128 * i + 128),
      ];
      const [block] = await builder.buildL2Block(1 + i, txs, l1ToL2Messages);

      /*// Useful for sol tests block generation
      const encoded = block.encode();
      console.log(`Size (${encoded.length}): ${encoded.toString('hex')}`);
      console.log(`calldata hash: 0x${block.getCalldataHash().toString('hex')}`);
      console.log(`l1 to l2 message hash: 0x${block.getL1ToL2MessagesHash().toString('hex')}`);
      console.log(`start state hash: 0x${block.getStartStateHash().toString('hex')}`);
      console.log(`end state hash: 0x${block.getEndStateHash().toString('hex')}`);
      console.log(`public data hash: 0x${block.getPublicInputsHash().toBuffer().toString('hex')}`);*/

      await publisher.processL2Block(block);

      const logs = await publicClient.getLogs({
        address: rollupAddress,
        event: getAbiItem({
          abi: RollupAbi,
          name: 'L2BlockProcessed',
        }),
        fromBlock: blockNumber + 1n,
      });
      expect(logs).toHaveLength(i + 1);
      expect(logs[i].args.blockNum).toEqual(BigInt(i + 1));

      const ethTx = await publicClient.getTransaction({
        hash: logs[i].transactionHash!,
      });

      const expectedData = encodeFunctionData({
        abi: RollupAbi,
        functionName: 'process',
        args: [`0x${l2Proof.toString('hex')}`, `0x${block.encode().toString('hex')}`],
      });
      expect(ethTx.input).toEqual(expectedData);

      const decoderArgs = [`0x${block.encode().toString('hex')}`] as const;
      const decodedHashes = await decoderHelper.read.computeDiffRootAndMessagesHash(decoderArgs);
      const decodedRes = await decoderHelper.read.decode(decoderArgs);
      const stateInRollup = await rollup.read.rollupStateHash();

      expect(block.number).toEqual(Number(decodedRes[0]));
      expect(block.getStartStateHash()).toEqual(hexStringToBuffer(decodedRes[1].toString()));
      expect(block.getEndStateHash()).toEqual(hexStringToBuffer(decodedRes[2].toString()));
      expect(block.getEndStateHash()).toEqual(hexStringToBuffer(stateInRollup.toString()));
      expect(block.getPublicInputsHash().toBuffer()).toEqual(hexStringToBuffer(decodedRes[3].toString()));
      expect(block.getCalldataHash()).toEqual(hexStringToBuffer(decodedHashes[0].toString()));
      expect(block.getL1ToL2MessagesHash()).toEqual(hexStringToBuffer(decodedHashes[1].toString()));
    }
  }, 60_000);

  it(`Build ${numberOfConsecutiveBlocks} blocks of 4 empty txs building on each other`, async () => {
    const stateInRollup_ = await rollup.read.rollupStateHash();
    expect(hexStringToBuffer(stateInRollup_.toString())).toEqual(Buffer.alloc(32, 0));

    const blockNumber = await publicClient.getBlockNumber();

    for (let i = 0; i < numberOfConsecutiveBlocks; i++) {
      const l1ToL2Messages = new Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n));
      const txs = [
        await makeEmptyProcessedTx(),
        await makeEmptyProcessedTx(),
        await makeEmptyProcessedTx(),
        await makeEmptyProcessedTx(),
      ];
      const [block] = await builder.buildL2Block(1 + i, txs, l1ToL2Messages);

      await publisher.processL2Block(block);

      const logs = await publicClient.getLogs({
        address: rollupAddress,
        event: getAbiItem({
          abi: RollupAbi,
          name: 'L2BlockProcessed',
        }),
        fromBlock: blockNumber + 1n,
      });
      expect(logs).toHaveLength(i + 1);
      expect(logs[i].args.blockNum).toEqual(BigInt(i + 1));

      const ethTx = await publicClient.getTransaction({
        hash: logs[i].transactionHash!,
      });

      const expectedData = encodeFunctionData({
        abi: RollupAbi,
        functionName: 'process',
        args: [`0x${l2Proof.toString('hex')}`, `0x${block.encode().toString('hex')}`],
      });
      expect(ethTx.input).toEqual(expectedData);

      const decoderArgs = [`0x${block.encode().toString('hex')}`] as const;
      const decodedHashes = await decoderHelper.read.computeDiffRootAndMessagesHash(decoderArgs);
      const decodedRes = await decoderHelper.read.decode(decoderArgs);
      const stateInRollup = await rollup.read.rollupStateHash();

      expect(block.number).toEqual(Number(decodedRes[0]));
      expect(block.getStartStateHash()).toEqual(hexStringToBuffer(decodedRes[1].toString()));
      expect(block.getEndStateHash()).toEqual(hexStringToBuffer(decodedRes[2].toString()));
      expect(block.getEndStateHash()).toEqual(hexStringToBuffer(stateInRollup.toString()));
      expect(block.getPublicInputsHash().toBuffer()).toEqual(hexStringToBuffer(decodedRes[3].toString()));
      expect(block.getCalldataHash()).toEqual(hexStringToBuffer(decodedHashes[0].toString()));
      expect(block.getL1ToL2MessagesHash()).toEqual(hexStringToBuffer(decodedHashes[1].toString()));
    }
  }, 60_000);
});

/**
 * Converts a hex string into a buffer. String may be 0x-prefixed or not.
 */
function hexStringToBuffer(hex: string): Buffer {
  if (!/^(0x)?[a-fA-F0-9]+$/.test(hex)) throw new Error(`Invalid format for hex string: "${hex}"`);
  if (hex.length % 2 === 1) throw new Error(`Invalid length for hex string: "${hex}"`);
  return Buffer.from(hex.replace(/^0x/, ''), 'hex');
}
