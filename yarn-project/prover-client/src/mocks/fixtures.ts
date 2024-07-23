import {
  MerkleTreeId,
  type ProcessedTx,
  makeEmptyProcessedTx as makeEmptyProcessedTxFromHistoricalTreeRoots,
  makeProcessedTx,
  mockTx,
} from '@aztec/circuit-types';
import {
  AztecAddress,
  EthAddress,
  Fr,
  GasFees,
  GlobalVariables,
  KernelCircuitPublicInputs,
  MAX_L2_TO_L1_MSGS_PER_TX,
  MAX_NOTE_HASHES_PER_TX,
  MAX_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  NULLIFIER_TREE_HEIGHT,
  PUBLIC_DATA_SUBTREE_HEIGHT,
  PublicDataTreeLeaf,
  PublicDataUpdateRequest,
} from '@aztec/circuits.js';
import { fr } from '@aztec/circuits.js/testing';
import { makeTuple } from '@aztec/foundation/array';
import { padArrayEnd } from '@aztec/foundation/collection';
import { randomBytes } from '@aztec/foundation/crypto';
import { type DebugLogger } from '@aztec/foundation/log';
import { fileURLToPath } from '@aztec/foundation/url';
import { getVKTreeRoot } from '@aztec/noir-protocol-circuits-types';
import { NativeACVMSimulator, type SimulationProvider, WASMSimulator } from '@aztec/simulator';
import { type MerkleTreeOperations } from '@aztec/world-state';

import * as fs from 'fs/promises';
import path from 'path';

const {
  BB_RELEASE_DIR = 'cpp/build/bin',
  TEMP_DIR = '/tmp',
  BB_BINARY_PATH = '',
  BB_WORKING_DIRECTORY = '',
  NOIR_RELEASE_DIR = 'noir-repo/target/release',
  ACVM_BINARY_PATH = '',
  ACVM_WORKING_DIRECTORY = '',
} = process.env;

// Determines if we have access to the bb binary and a tmp folder for temp files
export const getEnvironmentConfig = async (logger: DebugLogger) => {
  try {
    const expectedBBPath = BB_BINARY_PATH
      ? BB_BINARY_PATH
      : `${path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../../barretenberg/', BB_RELEASE_DIR)}/bb`;
    await fs.access(expectedBBPath, fs.constants.R_OK);
    const tempWorkingDirectory = `${TEMP_DIR}/${randomBytes(4).toString('hex')}`;
    const bbWorkingDirectory = BB_WORKING_DIRECTORY ? BB_WORKING_DIRECTORY : `${tempWorkingDirectory}/bb`;
    await fs.mkdir(bbWorkingDirectory, { recursive: true });
    logger.verbose(`Using native BB binary at ${expectedBBPath} with working directory ${bbWorkingDirectory}`);

    const expectedAcvmPath = ACVM_BINARY_PATH
      ? ACVM_BINARY_PATH
      : `${path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../../noir/', NOIR_RELEASE_DIR)}/acvm`;
    await fs.access(expectedAcvmPath, fs.constants.R_OK);
    const acvmWorkingDirectory = ACVM_WORKING_DIRECTORY ? ACVM_WORKING_DIRECTORY : `${tempWorkingDirectory}/acvm`;
    await fs.mkdir(acvmWorkingDirectory, { recursive: true });
    logger.verbose(`Using native ACVM binary at ${expectedAcvmPath} with working directory ${acvmWorkingDirectory}`);
    return {
      acvmWorkingDirectory,
      bbWorkingDirectory,
      expectedAcvmPath,
      expectedBBPath,
      directoryToCleanup: ACVM_WORKING_DIRECTORY && BB_WORKING_DIRECTORY ? undefined : tempWorkingDirectory,
    };
  } catch (err) {
    logger.verbose(`Native BB not available, error: ${err}`);
    return undefined;
  }
};

export async function getSimulationProvider(
  config: { acvmWorkingDirectory: string | undefined; acvmBinaryPath: string | undefined },
  logger?: DebugLogger,
): Promise<SimulationProvider> {
  if (config.acvmBinaryPath && config.acvmWorkingDirectory) {
    try {
      await fs.access(config.acvmBinaryPath, fs.constants.R_OK);
      await fs.mkdir(config.acvmWorkingDirectory, { recursive: true });
      logger?.info(
        `Using native ACVM at ${config.acvmBinaryPath} and working directory ${config.acvmWorkingDirectory}`,
      );
      return new NativeACVMSimulator(config.acvmWorkingDirectory, config.acvmBinaryPath);
    } catch {
      logger?.warn(`Failed to access ACVM at ${config.acvmBinaryPath}, falling back to WASM`);
    }
  }
  logger?.info('Using WASM ACVM simulation');
  return new WASMSimulator();
}

export const makeBloatedProcessedTx = (builderDb: MerkleTreeOperations, seed = 0x1) => {
  seed *= MAX_NULLIFIERS_PER_TX; // Ensure no clashing given incremental seeds
  const tx = mockTx(seed);
  const kernelOutput = KernelCircuitPublicInputs.empty();
  kernelOutput.constants.vkTreeRoot = getVKTreeRoot();
  kernelOutput.constants.historicalHeader = builderDb.getInitialHeader();
  kernelOutput.end.publicDataUpdateRequests = makeTuple(
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    i => new PublicDataUpdateRequest(fr(i), fr(i + 10), i + 20),
    seed + 0x500,
  );
  kernelOutput.end.publicDataUpdateRequests = makeTuple(
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    i => new PublicDataUpdateRequest(fr(i), fr(i + 10), i + 20),
    seed + 0x600,
  );

  const processedTx = makeProcessedTx(tx, kernelOutput, []);

  processedTx.data.end.noteHashes = makeTuple(MAX_NOTE_HASHES_PER_TX, fr, seed + 0x100);
  processedTx.data.end.nullifiers = makeTuple(MAX_NULLIFIERS_PER_TX, fr, seed + 0x100000);

  processedTx.data.end.nullifiers[tx.data.forPublic!.end.nullifiers.length - 1] = Fr.zero();

  processedTx.data.end.l2ToL1Msgs = makeTuple(MAX_L2_TO_L1_MSGS_PER_TX, fr, seed + 0x300);
  processedTx.data.end.noteEncryptedLogsHash = Fr.fromBuffer(processedTx.noteEncryptedLogs.hash());
  processedTx.data.end.encryptedLogsHash = Fr.fromBuffer(processedTx.encryptedLogs.hash());

  return processedTx;
};

export const makeEmptyProcessedTx = (builderDb: MerkleTreeOperations, chainId: Fr, version: Fr) => {
  const header = builderDb.getInitialHeader();
  return makeEmptyProcessedTxFromHistoricalTreeRoots(header, chainId, version, getVKTreeRoot());
};

// Updates the expectedDb trees based on the new note hashes, contracts, and nullifiers from these txs
export const updateExpectedTreesFromTxs = async (db: MerkleTreeOperations, txs: ProcessedTx[]) => {
  await db.appendLeaves(
    MerkleTreeId.NOTE_HASH_TREE,
    txs.flatMap(tx =>
      padArrayEnd(
        tx.data.end.noteHashes.filter(x => !x.isZero()),
        Fr.zero(),
        MAX_NOTE_HASHES_PER_TX,
      ),
    ),
  );
  await db.batchInsert(
    MerkleTreeId.NULLIFIER_TREE,
    txs.flatMap(tx =>
      padArrayEnd(
        tx.data.end.nullifiers.filter(x => !x.isZero()),
        Fr.zero(),
        MAX_NULLIFIERS_PER_TX,
      ).map(x => x.toBuffer()),
    ),
    NULLIFIER_TREE_HEIGHT,
  );
  for (const tx of txs) {
    await db.batchInsert(
      MerkleTreeId.PUBLIC_DATA_TREE,
      tx.data.end.publicDataUpdateRequests.map(write => {
        return new PublicDataTreeLeaf(write.leafSlot, write.newValue).toBuffer();
      }),
      PUBLIC_DATA_SUBTREE_HEIGHT,
    );
  }
};

export const makeGlobals = (blockNumber: number) => {
  return new GlobalVariables(
    Fr.ZERO,
    Fr.ZERO,
    new Fr(blockNumber),
    Fr.ZERO,
    EthAddress.ZERO,
    AztecAddress.ZERO,
    GasFees.empty(),
  );
};

export const makeEmptyProcessedTestTx = (builderDb: MerkleTreeOperations): ProcessedTx =>
  makeEmptyProcessedTx(builderDb, Fr.ZERO, Fr.ZERO);
