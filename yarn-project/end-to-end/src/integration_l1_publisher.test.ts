import { getConfigEnvVars } from '@aztec/aztec-node';
import {
  AztecAddress,
  GlobalVariables,
  KernelCircuitPublicInputs,
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  PublicDataUpdateRequest,
  makeTuple,
  range,
} from '@aztec/circuits.js';
import { fr, makeNewContractData, makeProof } from '@aztec/circuits.js/factories';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { to2Fields } from '@aztec/foundation/serialize';
import { DecoderHelperAbi, InboxAbi, OutboxAbi, RollupAbi } from '@aztec/l1-artifacts';
import {
  EmptyRollupProver,
  L1Publisher,
  SoloBlockBuilder,
  WasmRollupCircuitSimulator,
  getHistoricBlockData,
  getL1Publisher,
  getVerificationKeys,
  makeEmptyProcessedTx as makeEmptyProcessedTxFromHistoricTreeRoots,
  makeProcessedTx,
} from '@aztec/sequencer-client';
import { L2Actor, L2Block, mockTx } from '@aztec/types';
import { MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';

import { beforeEach, describe, expect, it } from '@jest/globals';
import { default as levelup } from 'levelup';
import memdown from 'memdown';
import {
  Address,
  Chain,
  GetContractReturnType,
  HttpTransport,
  PublicClient,
  WalletClient,
  encodeFunctionData,
  getAbiItem,
  getAddress,
  getContract,
} from 'viem';
import { PrivateKeyAccount, privateKeyToAccount } from 'viem/accounts';

import { setupL1Contracts } from './fixtures/utils.js';

// Accounts 4 and 5 of Anvil default startup with mnemonic: 'test test test test test test test test test test test junk'
const sequencerPK = '0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a';
const deployerPK = '0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba';

const logger = createDebugLogger('aztec:integration_l1_publisher');

const config = getConfigEnvVars();

const numberOfConsecutiveBlocks = 2;

describe('L1Publisher integration', () => {
  let publicClient: PublicClient<HttpTransport, Chain>;
  let deployerAccount: PrivateKeyAccount;

  let rollupAddress: Address;
  let inboxAddress: Address;
  let outboxAddress: Address;
  let decoderHelperAddress: Address;

  let rollup: GetContractReturnType<typeof RollupAbi, PublicClient<HttpTransport, Chain>>;
  let inbox: GetContractReturnType<
    typeof InboxAbi,
    PublicClient<HttpTransport, Chain>,
    WalletClient<HttpTransport, Chain>
  >;
  let outbox: GetContractReturnType<typeof OutboxAbi, PublicClient<HttpTransport, Chain>>;
  let decoderHelper: GetContractReturnType<typeof DecoderHelperAbi, PublicClient<HttpTransport, Chain>>;

  let publisher: L1Publisher;
  let l2Proof: Buffer;

  let builder: SoloBlockBuilder;
  let builderDb: MerkleTreeOperations;

  // The global variables of the last rollup
  let prevGlobals: GlobalVariables;

  beforeEach(async () => {
    deployerAccount = privateKeyToAccount(deployerPK);
    const {
      l1ContractAddresses,
      walletClient,
      publicClient: publicClient_,
    } = await setupL1Contracts(config.rpcUrl, deployerAccount, logger, true);
    publicClient = publicClient_;

    rollupAddress = getAddress(l1ContractAddresses.rollupAddress.toString());
    inboxAddress = getAddress(l1ContractAddresses.inboxAddress.toString());
    outboxAddress = getAddress(l1ContractAddresses.outboxAddress.toString());
    decoderHelperAddress = getAddress(l1ContractAddresses.decoderHelperAddress.toString());

    // Set up contract instances
    rollup = getContract({
      address: rollupAddress,
      abi: RollupAbi,
      publicClient,
    });
    inbox = getContract({
      address: inboxAddress,
      abi: InboxAbi,
      publicClient,
      walletClient,
    });
    outbox = getContract({
      address: outboxAddress,
      abi: OutboxAbi,
      publicClient,
    });
    decoderHelper = getContract({
      address: decoderHelperAddress!,
      abi: DecoderHelperAbi,
      publicClient,
    });

    builderDb = await MerkleTrees.new(levelup((memdown as any)())).then(t => t.asLatest());
    const vks = getVerificationKeys();
    const simulator = new WasmRollupCircuitSimulator();
    const prover = new EmptyRollupProver();
    builder = new SoloBlockBuilder(builderDb, vks, simulator, prover);

    l2Proof = Buffer.alloc(0);

    publisher = getL1Publisher({
      rpcUrl: config.rpcUrl,
      apiKey: '',
      requiredConfirmations: 1,
      l1Contracts: l1ContractAddresses,
      publisherPrivateKey: sequencerPK,
      l1BlockPublishRetryIntervalMS: 100,
    });

    prevGlobals = GlobalVariables.empty();
  }, 100_000);

  const makeEmptyProcessedTx = async () => {
    const historicTreeRoots = await getHistoricBlockData(builderDb, prevGlobals);
    const tx = await makeEmptyProcessedTxFromHistoricTreeRoots(
      historicTreeRoots,
      new Fr(config.chainId),
      new Fr(config.version),
    );
    return tx;
  };

  const makeBloatedProcessedTx = async (seed = 0x1) => {
    const tx = mockTx(seed);
    const kernelOutput = KernelCircuitPublicInputs.empty();
    kernelOutput.constants.txContext.chainId = fr(config.chainId);
    kernelOutput.constants.txContext.version = fr(config.version);
    kernelOutput.constants.blockData = await getHistoricBlockData(builderDb, prevGlobals);
    kernelOutput.end.publicDataUpdateRequests = makeTuple(
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
      i => new PublicDataUpdateRequest(fr(i), fr(0), fr(i + 10)),
      seed + 0x500,
    );

    const processedTx = await makeProcessedTx(tx, kernelOutput, makeProof());

    processedTx.data.end.newCommitments = makeTuple(MAX_NEW_COMMITMENTS_PER_TX, fr, seed + 0x100);
    processedTx.data.end.newNullifiers = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, fr, seed + 0x200);
    processedTx.data.end.newNullifiers[processedTx.data.end.newNullifiers.length - 1] = Fr.ZERO;
    processedTx.data.end.newL2ToL1Msgs = makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, fr, seed + 0x300);
    processedTx.data.end.newContracts = [makeNewContractData(seed + 0x1000)];
    processedTx.data.end.encryptedLogsHash = to2Fields(L2Block.computeKernelLogsHash(processedTx.encryptedLogs));
    processedTx.data.end.unencryptedLogsHash = to2Fields(L2Block.computeKernelLogsHash(processedTx.unencryptedLogs));

    return processedTx;
  };

  const sendToL2 = async (content: Fr, recipientAddress: AztecAddress) => {
    // @todo @LHerskind version hardcoded here (update to bigint or field)
    const recipient = new L2Actor(recipientAddress, 1);
    // Note: using max deadline
    const deadline = 2 ** 32 - 1;
    // getting the 32 byte hex string representation of the content
    const contentString = content.toString(true);
    // Using the 0 value for the secretHash.
    const emptySecretHash = Fr.ZERO.toString(true);

    await inbox.write.sendL2Message(
      [
        { actor: recipient.recipient.toString(), version: BigInt(recipient.version) },
        deadline,
        contentString,
        emptySecretHash,
      ],
      {} as any,
    );

    const entry = await inbox.read.computeEntryKey([
      {
        sender: {
          actor: deployerAccount.address,
          chainId: BigInt(publicClient.chain.id),
        },
        recipient: {
          actor: recipientAddress.toString(),
          version: 1n,
        },
        content: contentString,
        secretHash: emptySecretHash,
        deadline,
        fee: 0n,
      },
    ]);
    return Fr.fromString(entry);
  };

  it(`Build ${numberOfConsecutiveBlocks} blocks of 4 bloated txs building on each other`, async () => {
    const stateInRollup_ = await rollup.read.rollupStateHash();
    expect(hexStringToBuffer(stateInRollup_.toString())).toEqual(Buffer.alloc(32, 0));

    const blockNumber = await publicClient.getBlockNumber();
    // random recipient address, just kept consistent for easy testing ts/sol.
    const recipientAddress = AztecAddress.fromString(
      '0x1647b194c649f5dd01d7c832f89b0f496043c9150797923ea89e93d5ac619a93',
    );

    for (let i = 0; i < numberOfConsecutiveBlocks; i++) {
      const l1ToL2Content = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 128 * i + 1 + 0x400).map(fr);
      const l1ToL2Messages: Fr[] = [];

      for (let j = 0; j < l1ToL2Content.length; j++) {
        l1ToL2Messages.push(await sendToL2(l1ToL2Content[j], recipientAddress));
      }

      // check logs
      const inboxLogs = await publicClient.getLogs({
        address: inboxAddress,
        event: getAbiItem({
          abi: InboxAbi,
          name: 'MessageAdded',
        }),
        fromBlock: blockNumber + 1n,
      });
      expect(inboxLogs).toHaveLength(l1ToL2Messages.length * (i + 1));
      for (let j = 0; j < NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP; j++) {
        const event = inboxLogs[j + i * NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP].args;
        expect(event.content).toEqual(l1ToL2Content[j].toString(true));
        expect(event.deadline).toEqual(2 ** 32 - 1);
        expect(event.entryKey).toEqual(l1ToL2Messages[j].toString(true));
        expect(event.fee).toEqual(0n);
        expect(event.recipient).toEqual(recipientAddress.toString());
        expect(event.recipientVersion).toEqual(1n);
        expect(event.senderChainId).toEqual(BigInt(publicClient.chain.id));
        expect(event.sender).toEqual(deployerAccount.address);
      }

      // Ensure that each transaction has unique (non-intersecting nullifier values)
      const totalNullifiersPerBlock = 4 * MAX_NEW_NULLIFIERS_PER_TX;
      const txs = [
        await makeBloatedProcessedTx(totalNullifiersPerBlock * i + 1 * MAX_NEW_NULLIFIERS_PER_TX),
        await makeBloatedProcessedTx(totalNullifiersPerBlock * i + 2 * MAX_NEW_NULLIFIERS_PER_TX),
        await makeBloatedProcessedTx(totalNullifiersPerBlock * i + 3 * MAX_NEW_NULLIFIERS_PER_TX),
        await makeBloatedProcessedTx(totalNullifiersPerBlock * i + 4 * MAX_NEW_NULLIFIERS_PER_TX),
      ];
      const globalVariables = new GlobalVariables(
        new Fr(config.chainId),
        new Fr(config.version),
        new Fr(1 + i),
        new Fr(await rollup.read.lastBlockTs()),
      );
      const [block] = await builder.buildL2Block(globalVariables, txs, l1ToL2Messages);
      prevGlobals = globalVariables;

      // check that values are in the inbox
      for (let j = 0; j < l1ToL2Messages.length; j++) {
        if (l1ToL2Messages[j].isZero()) continue;
        expect(await inbox.read.contains([l1ToL2Messages[j].toString(true)])).toBeTruthy();
      }

      // check that values are not in the outbox
      for (let j = 0; j < block.newL2ToL1Msgs.length; j++) {
        expect(await outbox.read.contains([block.newL2ToL1Msgs[j].toString(true)])).toBeFalsy();
      }

      // Useful for sol tests block generation
      /* const encoded = block.encode();
      console.log(`Size (${encoded.length}): ${encoded.toString('hex')}`);
      console.log(`calldata hash: 0x${block.getCalldataHash().toString('hex')}`);
      console.log(`l1 to l2 message hash: 0x${block.getL1ToL2MessagesHash().toString('hex')}`);
      console.log(`start state hash: 0x${block.getStartStateHash().toString('hex')}`);
      console.log(`end state hash: 0x${block.getEndStateHash().toString('hex')}`);
      console.log(`public inputs hash: 0x${block.getPublicInputsHash().toBuffer().toString('hex')}`); */

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

      // check that values have been consumed from the inbox
      for (let j = 0; j < l1ToL2Messages.length; j++) {
        if (l1ToL2Messages[j].isZero()) continue;
        expect(await inbox.read.contains([l1ToL2Messages[j].toString(true)])).toBeFalsy();
      }
      // check that values are inserted into the outbox
      for (let j = 0; j < block.newL2ToL1Msgs.length; j++) {
        expect(await outbox.read.contains([block.newL2ToL1Msgs[j].toString(true)])).toBeTruthy();
      }
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
      const globalVariables = new GlobalVariables(
        new Fr(config.chainId),
        new Fr(config.version),
        new Fr(1 + i),
        new Fr(await rollup.read.lastBlockTs()),
      );
      const [block] = await builder.buildL2Block(globalVariables, txs, l1ToL2Messages);
      prevGlobals = globalVariables;

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
