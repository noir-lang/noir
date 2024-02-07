import { getConfigEnvVars } from '@aztec/aztec-node';
import {
  AztecAddress,
  Fr,
  GlobalVariables,
  L2Actor,
  L2Block,
  createDebugLogger,
  mockTx,
  to2Fields,
} from '@aztec/aztec.js';
import {
  EthAddress,
  Header,
  KernelCircuitPublicInputs,
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  PublicDataUpdateRequest,
  SideEffectLinkedToNoteHash,
} from '@aztec/circuits.js';
import {
  fr,
  makeNewContractData,
  makeNewSideEffect,
  makeNewSideEffectLinkedToNoteHash,
  makeProof,
} from '@aztec/circuits.js/factories';
import { createEthereumChain } from '@aztec/ethereum';
import { makeTuple, range } from '@aztec/foundation/array';
import { AztecLmdbStore } from '@aztec/kv-store';
import { InboxAbi, OutboxAbi, RollupAbi } from '@aztec/l1-artifacts';
import {
  EmptyRollupProver,
  L1Publisher,
  RealRollupCircuitSimulator,
  SoloBlockBuilder,
  getL1Publisher,
  getVerificationKeys,
  makeEmptyProcessedTx as makeEmptyProcessedTxFromHistoricalTreeRoots,
  makeProcessedTx,
} from '@aztec/sequencer-client';
import { MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';

import { beforeEach, describe, expect, it } from '@jest/globals';
import * as fs from 'fs';
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

  let rollup: GetContractReturnType<typeof RollupAbi, PublicClient<HttpTransport, Chain>>;
  let inbox: GetContractReturnType<
    typeof InboxAbi,
    PublicClient<HttpTransport, Chain>,
    WalletClient<HttpTransport, Chain>
  >;
  let outbox: GetContractReturnType<typeof OutboxAbi, PublicClient<HttpTransport, Chain>>;

  let publisher: L1Publisher;
  let l2Proof: Buffer;

  let builder: SoloBlockBuilder;
  let builderDb: MerkleTreeOperations;

  // The header of the last block
  let prevHeader: Header;

  const chainId = createEthereumChain(config.rpcUrl, config.apiKey).chainInfo.id;

  let coinbase: EthAddress;
  let feeRecipient: AztecAddress;

  // To overwrite the test data, set this to true and run the tests.
  const OVERWRITE_TEST_DATA = false;

  beforeEach(async () => {
    deployerAccount = privateKeyToAccount(deployerPK);
    const {
      l1ContractAddresses,
      walletClient,
      publicClient: publicClient_,
    } = await setupL1Contracts(config.rpcUrl, deployerAccount, logger);
    publicClient = publicClient_;

    rollupAddress = getAddress(l1ContractAddresses.rollupAddress.toString());
    inboxAddress = getAddress(l1ContractAddresses.inboxAddress.toString());
    outboxAddress = getAddress(l1ContractAddresses.outboxAddress.toString());

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

    builderDb = await MerkleTrees.new(await AztecLmdbStore.openTmp()).then(t => t.asLatest());
    const vks = getVerificationKeys();
    const simulator = new RealRollupCircuitSimulator();
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

    coinbase = config.coinbase || EthAddress.random();
    feeRecipient = config.feeRecipient || AztecAddress.random();

    prevHeader = await builderDb.buildInitialHeader();
  }, 100_000);

  const makeEmptyProcessedTx = async () => {
    const tx = await makeEmptyProcessedTxFromHistoricalTreeRoots(prevHeader, new Fr(chainId), new Fr(config.version));
    return tx;
  };

  const makeBloatedProcessedTx = async (seed = 0x1) => {
    const tx = mockTx(seed);
    const kernelOutput = KernelCircuitPublicInputs.empty();
    kernelOutput.constants.txContext.chainId = fr(chainId);
    kernelOutput.constants.txContext.version = fr(config.version);
    kernelOutput.constants.historicalHeader = prevHeader;
    kernelOutput.end.publicDataUpdateRequests = makeTuple(
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
      i => new PublicDataUpdateRequest(fr(i), fr(0), fr(i + 10)),
      seed + 0x500,
    );

    const processedTx = await makeProcessedTx(tx, kernelOutput, makeProof());

    processedTx.data.end.newCommitments = makeTuple(MAX_NEW_COMMITMENTS_PER_TX, makeNewSideEffect, seed + 0x100);
    processedTx.data.end.newNullifiers = makeTuple(
      MAX_NEW_NULLIFIERS_PER_TX,
      makeNewSideEffectLinkedToNoteHash,
      seed + 0x200,
    );
    processedTx.data.end.newNullifiers[processedTx.data.end.newNullifiers.length - 1] =
      SideEffectLinkedToNoteHash.empty();
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
    const contentString = content.toString();
    // Using the 0 value for the secretHash.
    const emptySecretHash = Fr.ZERO.toString();

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

  /**
   * Creates a json object that can be used to test the solidity contract.
   * The json object must be put into
   */
  const writeJson = (
    fileName: string,
    block: L2Block,
    l1ToL2Messages: Fr[],
    l1ToL2Content: Fr[],
    recipientAddress: AztecAddress,
    deployerAddress: `0x${string}`,
  ) => {
    if (!OVERWRITE_TEST_DATA) {
      return;
    }
    // Path relative to the package.json in the end-to-end folder
    const path = `../../l1-contracts/test/fixtures/${fileName}.json`;

    const jsonObject = {
      populate: {
        l1ToL2Content: l1ToL2Content.map(c => `0x${c.toBuffer().toString('hex').padStart(64, '0')}`),
        recipient: `0x${recipientAddress.toBuffer().toString('hex').padStart(64, '0')}`,
        sender: deployerAddress,
      },
      messages: {
        l1ToL2Messages: l1ToL2Messages.map(m => `0x${m.toBuffer().toString('hex').padStart(64, '0')}`),
        l2ToL1Messages: block.newL2ToL1Msgs.map(m => `0x${m.toBuffer().toString('hex').padStart(64, '0')}`),
      },
      block: {
        // The json formatting in forge is a bit brittle, so we convert Fr to a number in the few values below.
        // This should not be a problem for testing as long as the values are not larger than u32.
        archive: `0x${block.archive.root.toBuffer().toString('hex').padStart(64, '0')}`,
        body: `0x${block.bodyToBuffer().toString('hex')}`,
        calldataHash: `0x${block.getCalldataHash().toString('hex').padStart(64, '0')}`,
        decodedHeader: {
          bodyHash: `0x${block.header.bodyHash.toString('hex').padStart(64, '0')}`,
          globalVariables: {
            blockNumber: block.number,
            chainId: Number(block.header.globalVariables.chainId.toBigInt()),
            timestamp: Number(block.header.globalVariables.timestamp.toBigInt()),
            version: Number(block.header.globalVariables.version.toBigInt()),
            coinbase: `0x${block.header.globalVariables.coinbase.toBuffer().toString('hex').padStart(40, '0')}`,
            feeRecipient: `0x${block.header.globalVariables.feeRecipient.toBuffer().toString('hex').padStart(64, '0')}`,
          },
          lastArchive: {
            nextAvailableLeafIndex: block.header.lastArchive.nextAvailableLeafIndex,
            root: `0x${block.header.lastArchive.root.toBuffer().toString('hex').padStart(64, '0')}`,
          },
          stateReference: {
            l1ToL2MessageTree: {
              nextAvailableLeafIndex: block.header.state.l1ToL2MessageTree.nextAvailableLeafIndex,
              root: `0x${block.header.state.l1ToL2MessageTree.root.toBuffer().toString('hex').padStart(64, '0')}`,
            },
            partialStateReference: {
              contractTree: {
                nextAvailableLeafIndex: block.header.state.partial.contractTree.nextAvailableLeafIndex,
                root: `0x${block.header.state.partial.contractTree.root.toBuffer().toString('hex').padStart(64, '0')}`,
              },
              noteHashTree: {
                nextAvailableLeafIndex: block.header.state.partial.noteHashTree.nextAvailableLeafIndex,
                root: `0x${block.header.state.partial.noteHashTree.root.toBuffer().toString('hex').padStart(64, '0')}`,
              },
              nullifierTree: {
                nextAvailableLeafIndex: block.header.state.partial.nullifierTree.nextAvailableLeafIndex,
                root: `0x${block.header.state.partial.nullifierTree.root.toBuffer().toString('hex').padStart(64, '0')}`,
              },
              publicDataTree: {
                nextAvailableLeafIndex: block.header.state.partial.publicDataTree.nextAvailableLeafIndex,
                root: `0x${block.header.state.partial.publicDataTree.root
                  .toBuffer()
                  .toString('hex')
                  .padStart(64, '0')}`,
              },
            },
          },
        },
        header: `0x${block.header.toBuffer().toString('hex')}`,
        l1ToL2MessagesHash: `0x${block.getL1ToL2MessagesHash().toString('hex').padStart(64, '0')}`,
        publicInputsHash: `0x${block.getPublicInputsHash().toBuffer().toString('hex').padStart(64, '0')}`,
      },
    };

    const output = JSON.stringify(jsonObject, null, 2);
    fs.writeFileSync(path, output, 'utf8');
  };

  it(`Build ${numberOfConsecutiveBlocks} blocks of 4 bloated txs building on each other`, async () => {
    const archiveInRollup_ = await rollup.read.archive();
    expect(hexStringToBuffer(archiveInRollup_.toString())).toEqual(Buffer.alloc(32, 0));

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
        expect(event.content).toEqual(l1ToL2Content[j].toString());
        expect(event.deadline).toEqual(2 ** 32 - 1);
        expect(event.entryKey).toEqual(l1ToL2Messages[j].toString());
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
        new Fr(chainId),
        new Fr(config.version),
        new Fr(1 + i),
        new Fr(await rollup.read.lastBlockTs()),
        coinbase,
        feeRecipient,
      );
      const [block] = await builder.buildL2Block(globalVariables, txs, l1ToL2Messages);
      prevHeader = block.header;

      // check that values are in the inbox
      for (let j = 0; j < l1ToL2Messages.length; j++) {
        if (l1ToL2Messages[j].isZero()) {
          continue;
        }
        expect(await inbox.read.contains([l1ToL2Messages[j].toString()])).toBeTruthy();
      }

      // check that values are not in the outbox
      for (let j = 0; j < block.newL2ToL1Msgs.length; j++) {
        expect(await outbox.read.contains([block.newL2ToL1Msgs[j].toString()])).toBeFalsy();
      }

      writeJson(`mixed_block_${i}`, block, l1ToL2Messages, l1ToL2Content, recipientAddress, deployerAccount.address);

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
      expect(logs[i].args.blockNumber).toEqual(BigInt(i + 1));

      const ethTx = await publicClient.getTransaction({
        hash: logs[i].transactionHash!,
      });

      const expectedData = encodeFunctionData({
        abi: RollupAbi,
        functionName: 'process',
        args: [
          `0x${block.header.toBuffer().toString('hex')}`,
          `0x${block.archive.root.toBuffer().toString('hex')}`,
          `0x${block.getCalldataHash().toString('hex')}`,
          `0x${block.bodyToBuffer().toString('hex')}`,
          `0x${l2Proof.toString('hex')}`,
        ],
      });
      expect(ethTx.input).toEqual(expectedData);

      // check that values have been consumed from the inbox
      for (let j = 0; j < l1ToL2Messages.length; j++) {
        if (l1ToL2Messages[j].isZero()) {
          continue;
        }
        expect(await inbox.read.contains([l1ToL2Messages[j].toString()])).toBeFalsy();
      }
      // check that values are inserted into the outbox
      for (let j = 0; j < block.newL2ToL1Msgs.length; j++) {
        expect(await outbox.read.contains([block.newL2ToL1Msgs[j].toString()])).toBeTruthy();
      }
    }
  }, 360_000);

  it(`Build ${numberOfConsecutiveBlocks} blocks of 4 empty txs building on each other`, async () => {
    const archiveInRollup_ = await rollup.read.archive();
    expect(hexStringToBuffer(archiveInRollup_.toString())).toEqual(Buffer.alloc(32, 0));

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
        new Fr(chainId),
        new Fr(config.version),
        new Fr(1 + i),
        new Fr(await rollup.read.lastBlockTs()),
        coinbase,
        feeRecipient,
      );
      const [block] = await builder.buildL2Block(globalVariables, txs, l1ToL2Messages);
      prevHeader = block.header;

      writeJson(`empty_block_${i}`, block, l1ToL2Messages, [], AztecAddress.ZERO, deployerAccount.address);

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
      expect(logs[i].args.blockNumber).toEqual(BigInt(i + 1));

      const ethTx = await publicClient.getTransaction({
        hash: logs[i].transactionHash!,
      });

      const expectedData = encodeFunctionData({
        abi: RollupAbi,
        functionName: 'process',
        args: [
          `0x${block.header.toBuffer().toString('hex')}`,
          `0x${block.archive.root.toBuffer().toString('hex')}`,
          `0x${block.getCalldataHash().toString('hex')}`,
          `0x${block.bodyToBuffer().toString('hex')}`,
          `0x${l2Proof.toString('hex')}`,
        ],
      });
      expect(ethTx.input).toEqual(expectedData);
    }
  }, 60_000);
});

/**
 * Converts a hex string into a buffer. String may be 0x-prefixed or not.
 */
function hexStringToBuffer(hex: string): Buffer {
  if (!/^(0x)?[a-fA-F0-9]+$/.test(hex)) {
    throw new Error(`Invalid format for hex string: "${hex}"`);
  }
  if (hex.length % 2 === 1) {
    throw new Error(`Invalid length for hex string: "${hex}"`);
  }
  return Buffer.from(hex.replace(/^0x/, ''), 'hex');
}
