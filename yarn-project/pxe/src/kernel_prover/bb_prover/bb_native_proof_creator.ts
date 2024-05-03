import {
  Fr,
  type PrivateCircuitPublicInputs,
  type PrivateKernelCircuitPublicInputs,
  type PrivateKernelInitCircuitPrivateInputs,
  type PrivateKernelInnerCircuitPrivateInputs,
  type PrivateKernelTailCircuitPrivateInputs,
  type PrivateKernelTailCircuitPublicInputs,
  Proof,
  type VERIFICATION_KEY_LENGTH_IN_FIELDS,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { siloNoteHash } from '@aztec/circuits.js/hash';
import { randomBytes, sha256 } from '@aztec/foundation/crypto';
import { type LogFn, createDebugLogger } from '@aztec/foundation/log';
import { type Tuple } from '@aztec/foundation/serialize';
import { Timer } from '@aztec/foundation/timer';
import {
  ClientCircuitArtifacts,
  type ClientProtocolArtifact,
  convertPrivateKernelInitInputsToWitnessMap,
  convertPrivateKernelInitOutputsFromWitnessMap,
  convertPrivateKernelInnerInputsToWitnessMap,
  convertPrivateKernelInnerOutputsFromWitnessMap,
  convertPrivateKernelTailForPublicOutputsFromWitnessMap,
  convertPrivateKernelTailOutputsFromWitnessMap,
  executeTail,
  executeTailForPublic,
} from '@aztec/noir-protocol-circuits-types';
import { type ACVMField, WASMSimulator } from '@aztec/simulator';
import { type NoirCompiledCircuit } from '@aztec/types/noir';

import { type WitnessMap } from '@noir-lang/acvm_js';
import { serializeWitness } from '@noir-lang/noirc_abi';
import * as proc from 'child_process';
import * as fs from 'fs/promises';

import { type ProofCreator, type ProofOutput } from '../interface/proof_creator.js';

/**
 * Temporary implementation of ProofCreator using the native bb binary.
 * Will be replaced by the WASM equivalent once ready
 */

const VK_FILENAME = 'vk';
const VK_FIELDS_FILENAME = 'vk_fields.json';
const PROOF_FILENAME = 'proof';
//const PROOF_FIELDS_FILENAME = 'proof_fields.json';

//const AGGREGATION_OBJECT_SIZE = 16;
const CIRCUIT_SIZE_INDEX = 3;
const CIRCUIT_PUBLIC_INPUTS_INDEX = 4;
const CIRCUIT_RECURSIVE_INDEX = 5;

enum BB_RESULT {
  SUCCESS,
  FAILURE,
  ALREADY_PRESENT,
}

type BBSuccess = {
  status: BB_RESULT.SUCCESS | BB_RESULT.ALREADY_PRESENT;
  duration: number;
  pkPath?: string;
  vkPath?: string;
  proofPath?: string;
};

type BBFailure = {
  status: BB_RESULT.FAILURE;
  reason: string;
};

/**
 * Invokes the Barretenberg binary with the provided command and args
 * @param pathToBB - The path to the BB binary
 * @param command - The command to execute
 * @param args - The arguments to pass
 * @param logger - A log function
 * @param resultParser - An optional handler for detecting success or failure
 * @returns The completed partial witness outputted from the circuit
 */
function executeBB(
  pathToBB: string,
  command: string,
  args: string[],
  logger: LogFn,
  resultParser = (code: number) => code === 0,
) {
  return new Promise<BB_RESULT.SUCCESS | BB_RESULT.FAILURE>((resolve, reject) => {
    // spawn the bb process
    const bb = proc.spawn(pathToBB, [command, ...args]);
    bb.stdout.on('data', data => {
      const message = data.toString('utf-8').replace(/\n$/, '');
      logger(message);
    });
    bb.stderr.on('data', data => {
      const message = data.toString('utf-8').replace(/\n$/, '');
      logger(message);
    });
    bb.on('close', (code: number) => {
      if (resultParser(code)) {
        resolve(BB_RESULT.SUCCESS);
      } else {
        reject();
      }
    });
  }).catch(_ => BB_RESULT.FAILURE);
}

/**
 * Used for generating proofs of noir circuits.
 * It is assumed that the working directory is a temporary and/or random directory used solely for generating this proof.
 * @param pathToBB - The full path to the bb binary
 * @param workingDirectory - A working directory for use by bb
 * @param circuitName - An identifier for the circuit
 * @param bytecode - The compiled circuit bytecode
 * @param inputWitnessFile - The circuit input witness
 * @param log - A logging function
 * @returns An object containing a result indication, the location of the proof and the duration taken
 */
export async function generateProof(
  pathToBB: string,
  workingDirectory: string,
  circuitName: string,
  bytecode: Buffer,
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

  // The proof is written to e.g. /workingDirectory/proof
  const outputPath = `${workingDirectory}`;

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
    const args = ['-o', outputPath, '-b', bytecodePath, '-w', inputWitnessFile, '-v'];
    const timer = new Timer();
    const logFunction = (message: string) => {
      log(`${circuitName} BB out - ${message}`);
    };
    const result = await executeBB(pathToBB, 'prove_output_all', args, logFunction);
    const duration = timer.ms();
    // cleanup the bytecode
    await fs.rm(bytecodePath, { force: true });
    if (result == BB_RESULT.SUCCESS) {
      return {
        status: BB_RESULT.SUCCESS,
        duration,
        proofPath: `${outputPath}`,
        pkPath: undefined,
        vkPath: `${outputPath}`,
      };
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
async function verifyProof(
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

const bytecodeHashFilename = 'bytecode_hash';
const bytecodeFilename = 'bytecode';

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

  const outputPath = `${circuitOutputDirectory}`;

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
    return {
      status: BB_RESULT.ALREADY_PRESENT,
      duration: 0,
      pkPath: key === 'pk' ? outputPath : undefined,
      vkPath: key === 'vk' ? outputPath : undefined,
      proofPath: undefined,
    };
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
    let result = await executeBB(pathToBB, `write_${key}`, args, log);
    // If we succeeded and the type of key if verification, have bb write the 'fields' version too
    if (result == BB_RESULT.SUCCESS && key === 'vk') {
      const asFieldsArgs = ['-k', `${outputPath}/${VK_FILENAME}`, '-o', `${outputPath}/${VK_FIELDS_FILENAME}`, '-v'];
      result = await executeBB(pathToBB, `vk_as_fields`, asFieldsArgs, log);
    }
    const duration = timer.ms();
    // Cleanup the bytecode file
    await fs.rm(bytecodePath, { force: true });
    if (result == BB_RESULT.SUCCESS) {
      // Store the bytecode hash so we don't need to regenerate at a later time
      await fs.writeFile(bytecodeHashPath, bytecodeHash);
      return {
        status: BB_RESULT.SUCCESS,
        duration,
        pkPath: key === 'pk' ? outputPath : undefined,
        vkPath: key === 'vk' ? outputPath : undefined,
        proofPath: undefined,
      };
    }
    // Not a great error message here but it is difficult to decipher what comes from bb
    return { status: BB_RESULT.FAILURE, reason: `Failed to generate key` };
  } catch (error) {
    return { status: BB_RESULT.FAILURE, reason: `${error}` };
  }
}

/**
 * Used for verifying proofs of noir circuits
 * @param pathToBB - The full path to the bb binary
 * @param verificationKeyPath - The directory containing the binary verification key
 * @param verificationKeyFilename - The filename of the verification key
 * @param log - A logging function
 * @returns An object containing a result indication and duration taken
 */
// async function writeVkAsFields(
//   pathToBB: string,
//   verificationKeyPath: string,
//   verificationKeyFilename: string,
//   log: LogFn,
// ): Promise<BBFailure | BBSuccess> {
//   const binaryPresent = await fs
//     .access(pathToBB, fs.constants.R_OK)
//     .then(_ => true)
//     .catch(_ => false);
//   if (!binaryPresent) {
//     return { status: BB_RESULT.FAILURE, reason: `Failed to find bb binary at ${pathToBB}` };
//   }

//   try {
//     const args = ['-k', `${verificationKeyPath}/${verificationKeyFilename}`, '-v'];
//     const timer = new Timer();
//     const result = await executeBB(pathToBB, 'vk_as_fields', args, log);
//     const duration = timer.ms();
//     if (result == BB_RESULT.SUCCESS) {
//       return { status: BB_RESULT.SUCCESS, duration, vkPath: verificationKeyPath };
//     }
//     // Not a great error message here but it is difficult to decipher what comes from bb
//     return { status: BB_RESULT.FAILURE, reason: `Failed to create vk as fields` };
//   } catch (error) {
//     return { status: BB_RESULT.FAILURE, reason: `${error}` };
//   }
// }

/**
 * Used for verifying proofs of noir circuits
 * @param pathToBB - The full path to the bb binary
 * @param proofPath - The directory containing the binary proof
 * @param proofFileName - The filename of the proof
 * @param log - A logging function
 * @returns An object containing a result indication and duration taken
 */
// async function writeProofAsFields(
//   pathToBB: string,
//   proofPath: string,
//   proofFileName: string,
//   log: LogFn,
// ): Promise<BBFailure | BBSuccess> {
//   const binaryPresent = await fs
//     .access(pathToBB, fs.constants.R_OK)
//     .then(_ => true)
//     .catch(_ => false);
//   if (!binaryPresent) {
//     return { status: BB_RESULT.FAILURE, reason: `Failed to find bb binary at ${pathToBB}` };
//   }

//   try {
//     const args = ['-p', `${proofPath}/${proofFileName}`, '-v'];
//     const timer = new Timer();
//     const result = await executeBB(pathToBB, 'proof_as_fields', args, log);
//     const duration = timer.ms();
//     if (result == BB_RESULT.SUCCESS) {
//       return { status: BB_RESULT.SUCCESS, duration, proofPath: proofPath };
//     }
//     // Not a great error message here but it is difficult to decipher what comes from bb
//     return { status: BB_RESULT.FAILURE, reason: `Failed to create proof as fields` };
//   } catch (error) {
//     return { status: BB_RESULT.FAILURE, reason: `${error}` };
//   }
// }

type PrivateKernelProvingOps = {
  convertOutputs: (outputs: WitnessMap) => PrivateKernelCircuitPublicInputs | PrivateKernelTailCircuitPublicInputs;
};

const KernelArtifactMapping: Record<ClientProtocolArtifact, PrivateKernelProvingOps> = {
  PrivateKernelInitArtifact: {
    convertOutputs: convertPrivateKernelInitOutputsFromWitnessMap,
  },
  PrivateKernelInnerArtifact: {
    convertOutputs: convertPrivateKernelInnerOutputsFromWitnessMap,
  },
  PrivateKernelTailArtifact: {
    convertOutputs: convertPrivateKernelTailOutputsFromWitnessMap,
  },
  PrivateKernelTailToPublicArtifact: {
    convertOutputs: convertPrivateKernelTailForPublicOutputsFromWitnessMap,
  },
};

type VerificationKeyData = {
  hash: Fr;
  keyAsFields: Tuple<Fr, typeof VERIFICATION_KEY_LENGTH_IN_FIELDS>;
  keyAsBytes: Buffer;
  numPublicInputs: number;
  circuitSize: number;
  isRecursive: boolean;
};

/**
 * This proof creator implementation uses the native bb binary.
 * This is a temporary implementation until we make the WASM version work.
 */
export class BBNativeProofCreator implements ProofCreator {
  private simulator = new WASMSimulator();

  private verificationKeys: Map<ClientProtocolArtifact, Promise<VerificationKeyData>> = new Map<
    ClientProtocolArtifact,
    Promise<VerificationKeyData>
  >();

  constructor(
    private bbBinaryPath: string,
    private bbWorkingDirectory: string,
    private log = createDebugLogger('aztec:bb-native-prover'),
  ) {}

  public getSiloedCommitments(publicInputs: PrivateCircuitPublicInputs) {
    const contractAddress = publicInputs.callContext.storageContractAddress;

    return Promise.resolve(
      publicInputs.newNoteHashes.map(commitment => siloNoteHash(contractAddress, commitment.value)),
    );
  }

  public async createProofInit(
    inputs: PrivateKernelInitCircuitPrivateInputs,
  ): Promise<ProofOutput<PrivateKernelCircuitPublicInputs>> {
    const witnessMap = convertPrivateKernelInitInputsToWitnessMap(inputs);
    return await this.createSafeProof(witnessMap, 'PrivateKernelInitArtifact');
  }

  public async createProofInner(
    inputs: PrivateKernelInnerCircuitPrivateInputs,
  ): Promise<ProofOutput<PrivateKernelCircuitPublicInputs>> {
    const witnessMap = convertPrivateKernelInnerInputsToWitnessMap(inputs);
    return await this.createSafeProof(witnessMap, 'PrivateKernelInnerArtifact');
  }

  public async createProofTail(
    inputs: PrivateKernelTailCircuitPrivateInputs,
  ): Promise<ProofOutput<PrivateKernelTailCircuitPublicInputs>> {
    // if (!inputs.isForPublic()) {
    //   const witnessMap = convertPrivateKernelTailInputsToWitnessMap(inputs);
    //   return await this.createSafeProof(witnessMap, 'PrivateKernelTailArtifact');
    // }

    if (!inputs.isForPublic()) {
      const result = await executeTail(inputs);
      return {
        publicInputs: result,
        proof: makeEmptyProof(),
      };
    }
    // const witnessMap = convertPrivateKernelTailToPublicInputsToWitnessMap(inputs);
    // return await this.createSafeProof(witnessMap, 'PrivateKernelTailToPublicArtifact');
    const result = await executeTailForPublic(inputs);
    return {
      publicInputs: result,
      proof: makeEmptyProof(),
    };
  }

  public async createAppCircuitProof(partialWitness: Map<number, ACVMField>, bytecode: Buffer): Promise<Proof> {
    const directory = `${this.bbWorkingDirectory}/${randomBytes(8).toString('hex')}`;
    await fs.mkdir(directory, { recursive: true });
    this.log.debug(`Created directory: ${directory}`);
    try {
      this.log.debug(`Proving app circuit`);
      const proof = await this.createProof(directory, partialWitness, bytecode, 'App');
      return new Proof(proof);
    } finally {
      await fs.rm(directory, { recursive: true, force: true });
      this.log.debug(`Deleted directory: ${directory}`);
    }
  }

  /**
   * Verifies a proof, will generate the verification key if one is not cached internally
   * @param circuitType - The type of circuit whose proof is to be verified
   * @param proof - The proof to be verified
   */
  public async verifyProof(circuitType: ClientProtocolArtifact, proof: Proof) {
    // Create random directory to be used for temp files
    const bbWorkingDirectory = `${this.bbWorkingDirectory}/${randomBytes(8).toString('hex')}`;
    await fs.mkdir(bbWorkingDirectory, { recursive: true });

    const proofFileName = `${bbWorkingDirectory}/proof`;
    const verificationKeyPath = `${bbWorkingDirectory}/vk`;
    const verificationKey = await this.getVerificationKeyDataForCircuit(circuitType);

    this.log.debug(`Verifying with key: ${verificationKey.hash.toString()}`);

    await fs.writeFile(proofFileName, proof.buffer);
    await fs.writeFile(verificationKeyPath, verificationKey.keyAsBytes);

    const logFunction = (message: string) => {
      this.log.debug(`${circuitType} BB out - ${message}`);
    };

    const result = await verifyProof(this.bbBinaryPath, proofFileName, verificationKeyPath!, logFunction);

    await fs.rm(bbWorkingDirectory, { recursive: true, force: true });

    if (result.status === BB_RESULT.FAILURE) {
      const errorMessage = `Failed to verify ${circuitType} proof!`;
      throw new Error(errorMessage);
    }

    this.log.info(`Successfully verified ${circuitType} proof in ${result.duration} ms`);
  }

  /**
   * Returns the verification key data for a circuit, will generate and cache it if not cached internally
   * @param circuitType - The type of circuit for which the verification key is required
   * @returns The verification key data
   */
  private async getVerificationKeyDataForCircuit(circuitType: ClientProtocolArtifact): Promise<VerificationKeyData> {
    let promise = this.verificationKeys.get(circuitType);
    if (!promise) {
      promise = generateKeyForNoirCircuit(
        this.bbBinaryPath,
        this.bbWorkingDirectory,
        circuitType,
        ClientCircuitArtifacts[circuitType],
        'vk',
        this.log.debug,
      ).then(result => {
        if (result.status === BB_RESULT.FAILURE) {
          throw new Error(`Failed to generate verification key for ${circuitType}, ${result.reason}`);
        }
        return this.convertVk(result.vkPath!);
      });
      this.verificationKeys.set(circuitType, promise);
    }
    return await promise;
  }

  /**
   * Reads the verification key data stored at the specified location and parses into a VerificationKeyData
   * @param filePath - The directory containing the verification key data files
   * @returns The verification key data
   */
  private async convertVk(filePath: string): Promise<VerificationKeyData> {
    const [rawFields, rawBinary] = await Promise.all([
      fs.readFile(`${filePath}/${VK_FIELDS_FILENAME}`, { encoding: 'utf-8' }),
      fs.readFile(`${filePath}/${VK_FILENAME}`),
    ]);
    const fieldsJson = JSON.parse(rawFields);
    const fields = fieldsJson.map(Fr.fromString);
    // The first item is the hash, this is not part of the actual VK
    const vkHash = fields[0];
    const actualVk = fields.slice(1);
    const vk: VerificationKeyData = {
      hash: vkHash,
      keyAsFields: actualVk as Tuple<Fr, typeof VERIFICATION_KEY_LENGTH_IN_FIELDS>,
      keyAsBytes: rawBinary,
      numPublicInputs: Number(actualVk[CIRCUIT_PUBLIC_INPUTS_INDEX]),
      circuitSize: Number(actualVk[CIRCUIT_SIZE_INDEX]),
      isRecursive: actualVk[CIRCUIT_RECURSIVE_INDEX] == Fr.ONE,
    };
    return vk;
  }

  /**
   * Ensures our verification key cache includes the key data located at the specified directory
   * @param filePath - The directory containing the verification key data files
   * @param circuitType - The type of circuit to which the verification key corresponds
   */
  private async updateVerificationKeyAfterProof(filePath: string, circuitType: ClientProtocolArtifact) {
    let promise = this.verificationKeys.get(circuitType);
    if (!promise) {
      promise = this.convertVk(filePath);
      this.log.debug(`Updated verification key for circuit: ${circuitType}`);
      this.verificationKeys.set(circuitType, promise);
    }
    await promise;
  }

  private async createSafeProof<T>(inputs: WitnessMap, circuitType: ClientProtocolArtifact): Promise<ProofOutput<T>> {
    const directory = `${this.bbWorkingDirectory}/${randomBytes(8).toString('hex')}`;
    await fs.mkdir(directory, { recursive: true });
    this.log.debug(`Created directory: ${directory}`);
    try {
      return await this.generateWitnessAndCreateProof(inputs, circuitType, directory);
    } finally {
      await fs.rm(directory, { recursive: true, force: true });
      this.log.debug(`Deleted directory: ${directory}`);
    }
  }

  private async generateWitnessAndCreateProof<T>(
    inputs: WitnessMap,
    circuitType: ClientProtocolArtifact,
    directory: string,
  ): Promise<ProofOutput<T>> {
    this.log.debug(`Generating witness for ${circuitType}`);
    const compiledCircuit: NoirCompiledCircuit = ClientCircuitArtifacts[circuitType];

    const outputWitness = await this.simulator.simulateCircuit(inputs, compiledCircuit);

    this.log.debug(`Generated witness for ${circuitType}`);

    const publicInputs = KernelArtifactMapping[circuitType].convertOutputs(outputWitness) as T;

    const proofBuffer = await this.createProof(
      directory,
      outputWitness,
      Buffer.from(compiledCircuit.bytecode, 'base64'),
      circuitType,
    );

    const proofOutput: ProofOutput<T> = {
      publicInputs,
      proof: new Proof(proofBuffer),
    };
    return proofOutput;
  }

  private async createProof(
    directory: string,
    partialWitness: WitnessMap,
    bytecode: Buffer,
    circuitType: ClientProtocolArtifact | 'App',
  ) {
    const compressedBincodedWitness = serializeWitness(partialWitness);

    const inputsWitnessFile = `${directory}/witness.gz`;

    await fs.writeFile(inputsWitnessFile, compressedBincodedWitness);

    this.log.debug(`Written ${inputsWitnessFile}`);

    const provingResult = await generateProof(
      this.bbBinaryPath,
      directory,
      circuitType,
      bytecode,
      inputsWitnessFile,
      this.log.debug,
    );

    if (provingResult.status === BB_RESULT.FAILURE) {
      this.log.error(`Failed to generate proof for ${circuitType}: ${provingResult.reason}`);
      throw new Error(provingResult.reason);
    }

    if (circuitType !== 'App') {
      await this.updateVerificationKeyAfterProof(directory, circuitType);
    }
    const proofFile = `${directory}/${PROOF_FILENAME}`;
    return await fs.readFile(proofFile);
  }

  /**
   * Parses and returns the proof data stored at the specified directory
   * @param filePath - The directory containing the proof data
   * @param circuitType - The type of circuit proven
   * @returns The proof
   */
  // private async readProofAsFields<PROOF_LENGTH extends number>(
  //   filePath: string,
  //   circuitType: ClientProtocolArtifact,
  // ): Promise<RecursiveProof<PROOF_LENGTH>> {
  //   const [binaryProof, proofString] = await Promise.all([
  //     fs.readFile(`${filePath}/${PROOF_FILENAME}`),
  //     fs.readFile(`${filePath}/${PROOF_FIELDS_FILENAME}`, { encoding: 'utf-8' }),
  //   ]);
  //   const json = JSON.parse(proofString);
  //   const fields = json.map(Fr.fromString);
  //   const vkData = await this.verificationKeys.get(circuitType);
  //   if (!vkData) {
  //     throw new Error(`Invalid verification key for ${circuitType}`);
  //   }
  //   const numPublicInputs = CIRCUITS_WITHOUT_AGGREGATION.has(circuitType)
  //     ? vkData.numPublicInputs
  //     : vkData.numPublicInputs - AGGREGATION_OBJECT_SIZE;
  //   const fieldsWithoutPublicInputs = fields.slice(numPublicInputs);
  //   logger.debug(
  //     `Circuit type: ${circuitType}, complete proof length: ${fields.length}, without public inputs: ${fieldsWithoutPublicInputs.length}, num public inputs: ${numPublicInputs}, circuit size: ${vkData.circuitSize}, is recursive: ${vkData.isRecursive}, raw length: ${binaryProof.length}`,
  //   );
  //   const proof = new RecursiveProof<PROOF_LENGTH>(fieldsWithoutPublicInputs, new Proof(binaryProof));
  //   return proof;
  // }
}
