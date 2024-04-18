import { sha256 } from '@aztec/foundation/crypto';
import { type LogFn } from '@aztec/foundation/log';
import { Timer } from '@aztec/foundation/timer';
import { type NoirCompiledCircuit } from '@aztec/types/noir';

import * as proc from 'child_process';
import * as fs from 'fs/promises';

import { BBNativeRollupProver, type BBProverConfig } from '../prover/bb_prover.js';

export enum BB_RESULT {
  SUCCESS,
  FAILURE,
  ALREADY_PRESENT,
}

export type BBSuccess = {
  status: BB_RESULT.SUCCESS | BB_RESULT.ALREADY_PRESENT;
  duration: number;
  path?: string;
};

export type BBFailure = {
  status: BB_RESULT.FAILURE;
  reason: string;
};

export type BBResult = BBSuccess | BBFailure;

/**
 * Invokes the Barretenberg binary with the provided command and args
 * @param pathToBB - The path to the BB binary
 * @param command - The command to execute
 * @param args - The arguments to pass
 * @param logger - A log function
 * @param resultParser - An optional handler for detecting success or failure
 * @returns The completed partial witness outputted from the circuit
 */
export function executeBB(
  pathToBB: string,
  command: string,
  args: string[],
  logger: LogFn,
  resultParser = (code: number) => code === 0,
) {
  return new Promise<BB_RESULT.SUCCESS | BB_RESULT.FAILURE>((resolve, reject) => {
    // spawn the bb process
    const acvm = proc.spawn(pathToBB, [command, ...args]);
    acvm.stdout.on('data', data => {
      const message = data.toString('utf-8').replace(/\n$/, '');
      logger(message);
    });
    acvm.stderr.on('data', data => {
      const message = data.toString('utf-8').replace(/\n$/, '');
      logger(message);
    });
    acvm.on('close', (code: number) => {
      if (resultParser(code)) {
        resolve(BB_RESULT.SUCCESS);
      } else {
        reject();
      }
    });
  }).catch(_ => BB_RESULT.FAILURE);
}

const bytecodeHashFilename = 'bytecode_hash';
const bytecodeFilename = 'bytecode';
const proofFileName = 'proof';

/**
 * Used for generating either a proving or verification key, will exit early if the key already exists
 * It assumes the provided working directory is one where the caller wishes to maintain a permanent set of keys
 * It is not considered a temporary directory
 * @param pathToBB - The full path to the bb binary
 * @param workingDirectory - The directory into which the key should be created
 * @param circuitName - An identifier for the circuit
 * @param compiledCircuit - The compiled circuit
 * @param key - The type of key, either 'pk' or 'vk'
 * @param log - A logging function
 * @param force - Force the key to be regenerated even if it already exists
 * @returns An instance of BBResult
 */
export async function generateKeyForNoirCircuit(
  pathToBB: string,
  workingDirectory: string,
  circuitName: string,
  compiledCircuit: NoirCompiledCircuit,
  key: 'vk' | 'pk',
  log: LogFn,
  force = false,
): Promise<BBSuccess | BBFailure> {
  const bytecode = Buffer.from(compiledCircuit.bytecode, 'base64');

  // The key generation is written to e.g. /workingDirectory/pk/BaseParityArtifact/pk
  // The bytecode hash file is also written here as /workingDirectory/pk/BaseParityArtifact/bytecode-hash
  // The bytecode is written to e.g. /workingDirectory/pk/BaseParityArtifact/bytecode
  // The bytecode is removed after the key is generated, leaving just the hash file
  const circuitOutputDirectory = `${workingDirectory}/${key}/${circuitName}`;
  const bytecodeHashPath = `${circuitOutputDirectory}/${bytecodeHashFilename}`;
  const bytecodePath = `${circuitOutputDirectory}/${bytecodeFilename}`;
  const bytecodeHash = sha256(bytecode);

  const outputPath = `${circuitOutputDirectory}/${key}`;

  // ensure the directory exists
  await fs.mkdir(circuitOutputDirectory, { recursive: true });

  // Generate the key if we have been told to, or there is no bytecode hash
  let mustRegenerate =
    force ||
    (await fs
      .access(bytecodeHashPath, fs.constants.R_OK)
      .then(_ => false)
      .catch(_ => true));

  if (!mustRegenerate) {
    // Check to see if the bytecode hash has changed from the stored value
    const data: Buffer = await fs.readFile(bytecodeHashPath).catch(_ => Buffer.alloc(0));
    mustRegenerate = data.length == 0 || !data.equals(bytecodeHash);
  }

  if (!mustRegenerate) {
    // No need to generate, early out
    return { status: BB_RESULT.ALREADY_PRESENT, duration: 0, path: outputPath };
  }

  // Check we have access to bb
  const binaryPresent = await fs
    .access(pathToBB, fs.constants.R_OK)
    .then(_ => true)
    .catch(_ => false);
  if (!binaryPresent) {
    return { status: BB_RESULT.FAILURE, reason: `Failed to find bb binary at ${pathToBB}` };
  }

  // We are now going to generate the key
  try {
    // Write the bytecode to the working directory
    await fs.writeFile(bytecodePath, bytecode);

    // args are the output path and the input bytecode path
    const args = ['-o', outputPath, '-b', bytecodePath];
    const timer = new Timer();
    const result = await executeBB(pathToBB, `write_${key}`, args, log);
    const duration = timer.ms();
    // Cleanup the bytecode file
    await fs.rm(bytecodePath, { force: true });
    if (result == BB_RESULT.SUCCESS) {
      // Store the bytecode hash so we don't need to regenerate at a later time
      await fs.writeFile(bytecodeHashPath, bytecodeHash);
      return { status: BB_RESULT.SUCCESS, duration, path: outputPath };
    }
    // Not a great error message here but it is difficult to decipher what comes from bb
    return { status: BB_RESULT.FAILURE, reason: `Failed to generate key` };
  } catch (error) {
    return { status: BB_RESULT.FAILURE, reason: `${error}` };
  }
}

/**
 * Used for generating proofs of noir circuits.
 * It is assumed that the working directory is a temporary and/or random directory used solely for generating this proof.
 * @param pathToBB - The full path to the bb binary
 * @param workingDirectory - A working directory for use by bb
 * @param circuitName - An identifier for the circuit
 * @param compiledCircuit - The compiled circuit
 * @param inputWitnessFile - The circuit input witness
 * @param log - A logging function
 * @returns An object containing a result indication, the location of the proof and the duration taken
 */
export async function generateProof(
  pathToBB: string,
  workingDirectory: string,
  circuitName: string,
  compiledCircuit: NoirCompiledCircuit,
  inputWitnessFile: string,
  log: LogFn,
): Promise<BBFailure | BBSuccess> {
  // Check that the working directory exists
  try {
    await fs.access(workingDirectory);
  } catch (error) {
    return { status: BB_RESULT.FAILURE, reason: `Working directory ${workingDirectory} does not exist` };
  }

  // The bytecode is written to e.g. /workingDirectory/BaseParityArtifact-bytecode
  const bytecodePath = `${workingDirectory}/${circuitName}-bytecode`;
  const bytecode = Buffer.from(compiledCircuit.bytecode, 'base64');

  // The proof is written to e.g. /workingDirectory/proof
  const outputPath = `${workingDirectory}/${proofFileName}`;

  const binaryPresent = await fs
    .access(pathToBB, fs.constants.R_OK)
    .then(_ => true)
    .catch(_ => false);
  if (!binaryPresent) {
    return { status: BB_RESULT.FAILURE, reason: `Failed to find bb binary at ${pathToBB}` };
  }

  try {
    // Write the bytecode to the working directory
    await fs.writeFile(bytecodePath, bytecode);
    const args = ['-o', outputPath, '-b', bytecodePath, '-w', inputWitnessFile];
    const command = 'prove';
    const timer = new Timer();
    const logFunction = (message: string) => {
      log(`${circuitName} BB out - ${message}`);
    };
    const result = await executeBB(pathToBB, command, args, logFunction);
    const duration = timer.ms();
    // cleanup the bytecode
    await fs.rm(bytecodePath, { force: true });
    if (result == BB_RESULT.SUCCESS) {
      return { status: BB_RESULT.SUCCESS, duration, path: outputPath };
    }
    // Not a great error message here but it is difficult to decipher what comes from bb
    return { status: BB_RESULT.FAILURE, reason: `Failed to generate proof` };
  } catch (error) {
    return { status: BB_RESULT.FAILURE, reason: `${error}` };
  }
}

/**
 * Used for verifying proofs of noir circuits
 * @param pathToBB - The full path to the bb binary
 * @param proofFullPath - The full path to the proof to be verified
 * @param verificationKeyPath - The full path to the circuit verification key
 * @param log - A logging function
 * @returns An object containing a result indication and duration taken
 */
export async function verifyProof(
  pathToBB: string,
  proofFullPath: string,
  verificationKeyPath: string,
  log: LogFn,
): Promise<BBFailure | BBSuccess> {
  const binaryPresent = await fs
    .access(pathToBB, fs.constants.R_OK)
    .then(_ => true)
    .catch(_ => false);
  if (!binaryPresent) {
    return { status: BB_RESULT.FAILURE, reason: `Failed to find bb binary at ${pathToBB}` };
  }

  try {
    const args = ['-p', proofFullPath, '-k', verificationKeyPath];
    const timer = new Timer();
    const result = await executeBB(pathToBB, 'verify', args, log);
    const duration = timer.ms();
    if (result == BB_RESULT.SUCCESS) {
      return { status: BB_RESULT.SUCCESS, duration };
    }
    // Not a great error message here but it is difficult to decipher what comes from bb
    return { status: BB_RESULT.FAILURE, reason: `Failed to verify proof` };
  } catch (error) {
    return { status: BB_RESULT.FAILURE, reason: `${error}` };
  }
}

/**
 * Used for generating all verification keys required by server protocol circuits
 * @param pathToBB - The full path to the bb binary
 * @param workingDirectory - The directory to be used for the keys
 * @param log - A logging function
 */
export async function generateAllServerVks(pathToBB: string, workingDirectory: string, log: LogFn) {
  const bbConfig: BBProverConfig = {
    bbBinaryPath: pathToBB,
    bbWorkingDirectory: workingDirectory,

    // These aren't needed for this
    acvmBinaryPath: '',
    acvmWorkingDirectory: '',
    circuitFilter: [],
  };
  // This will generate all of the server circuit verification keys for us
  try {
    await BBNativeRollupProver.generateVerificationKeys(bbConfig);
  } catch (error) {
    log(`Failed to generate verification keys: ${error}`);
  }
}
