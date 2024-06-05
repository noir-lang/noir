import { type AztecNodeConfig, type AztecNodeService } from '@aztec/aztec-node';
import {
  type AztecNode,
  BatchCall,
  type Fr,
  INITIAL_L2_BLOCK_NUM,
  type PXE,
  type PartialAddress,
  type SentTx,
  retryUntil,
  sleep,
} from '@aztec/aztec.js';
import { times } from '@aztec/foundation/collection';
import { randomInt } from '@aztec/foundation/crypto';
import { BenchmarkingContract } from '@aztec/noir-contracts.js/Benchmarking';
import { type PXEService, createPXEService } from '@aztec/pxe';

import { mkdirpSync } from 'fs-extra';
import { globSync } from 'glob';
import { join } from 'path';

import { type EndToEndContext, setup } from '../fixtures/utils.js';

/**
 * Setup for benchmarks. Initializes a remote node with a single account and deploys a benchmark contract.
 */
export async function benchmarkSetup(opts: Partial<AztecNodeConfig>) {
  const context = await setup(1, { ...opts });
  const contract = await BenchmarkingContract.deploy(context.wallet).send().deployed();
  context.logger.info(`Deployed benchmarking contract at ${contract.address}`);
  const sequencer = (context.aztecNode as AztecNodeService).getSequencer()!;
  return { context, contract, sequencer };
}

/**
 * Creates and returns a directory with the current job name and a random number.
 * @param index - Index to merge into the dir path.
 * @returns A path to a created dir.
 */
export function makeDataDirectory(index: number) {
  const testName = expect.getState().currentTestName!.split(' ')[0].replaceAll('/', '_');
  const db = join('data', testName, index.toString(), `${randomInt(99)}`);
  mkdirpSync(db);
  return db;
}

/**
 * Returns the size in disk of a folder.
 * @param path - Path to the folder.
 * @returns Size in bytes.
 */
export function getFolderSize(path: string): number {
  return globSync('**', { stat: true, cwd: path, nodir: true, withFileTypes: true }).reduce(
    (accum, file) => accum + (file as any as { /** Size */ size: number }).size,
    0,
  );
}

/**
 * Returns a call to the benchmark contract. Each call has a private execution (account entrypoint),
 * a nested private call (create_note), a public call (increment_balance), and a nested public
 * call (broadcast). These include emitting one private note and one unencrypted log, two storage
 * reads and one write.
 * @param index - Index of the call within a block.
 * @param context - End to end context.
 * @param contract - Benchmarking contract.
 * @returns A BatchCall instance.
 */
export function makeCall(index: number, context: EndToEndContext, contract: BenchmarkingContract) {
  const owner = context.wallet.getAddress();
  // Setting the outgoing viewer to owner here since the outgoing logs are not important in this context
  const outgoingViewer = owner;
  return new BatchCall(context.wallet, [
    contract.methods.create_note(owner, outgoingViewer, index + 1).request(),
    contract.methods.increment_balance(owner, index + 1).request(),
  ]);
}

/**
 * Assembles and sends multiple transactions simultaneously to the node in context.
 * Each tx is the result of calling makeCall.
 * @param txCount - How many txs to send
 * @param context - End to end context.
 * @param contract - Target contract.
 * @returns Array of sent txs.
 */
export async function sendTxs(
  txCount: number,
  context: EndToEndContext,
  contract: BenchmarkingContract,
): Promise<SentTx[]> {
  const calls = times(txCount, index => makeCall(index, context, contract));
  await Promise.all(calls.map(call => call.prove({ skipPublicSimulation: true })));
  const sentTxs = calls.map(call => call.send());

  // Awaiting txHash waits until the aztec node has received the tx into its p2p pool
  await Promise.all(sentTxs.map(tx => tx.getTxHash()));
  await sleep(100);

  return sentTxs;
}

/**
 * Creates a new PXE and awaits until it's synced with the node.
 * @param node - Node to connect the pxe to.
 * @param contract - Benchmark contract to add to the pxe.
 * @param startingBlock - First l2 block to process.
 * @returns The new PXE.
 */
export async function waitNewPXESynced(
  node: AztecNode,
  contract: BenchmarkingContract,
  startingBlock: number = INITIAL_L2_BLOCK_NUM,
): Promise<PXEService> {
  const pxe = await createPXEService(node, {
    l2BlockPollingIntervalMS: 100,
    l2StartingBlock: startingBlock,
  });
  await pxe.registerContract(contract);
  await retryUntil(() => pxe.isGlobalStateSynchronized(), 'pxe-global-sync');
  return pxe;
}

/**
 * Registers a new account in a pxe and waits until it's synced all its notes.
 * @param pxe - PXE where to register the account.
 * @param secretKey - Secret key of the account to register.
 * @param partialAddress - Partial address of the account to register.
 */
export async function waitRegisteredAccountSynced(pxe: PXE, secretKey: Fr, partialAddress: PartialAddress) {
  const l2Block = await pxe.getBlockNumber();
  const accountAddress = (await pxe.registerAccount(secretKey, partialAddress)).address;
  const isAccountSynced = async () => (await pxe.getSyncStatus()).notes[accountAddress.toString()] === l2Block;
  await retryUntil(isAccountSynced, 'pxe-notes-sync');
}
