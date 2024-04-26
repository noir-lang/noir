/* eslint-disable require-await */
import {
  type PublicInputsAndProof,
  type PublicKernelNonTailRequest,
  type PublicKernelTailRequest,
  PublicKernelType,
  makePublicInputsAndProof,
} from '@aztec/circuit-types';
import {
  type BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  Fr,
  type KernelCircuitPublicInputs,
  type MergeRollupInputs,
  type NESTED_RECURSIVE_PROOF_LENGTH,
  type ParityPublicInputs,
  type PreviousRollupData,
  Proof,
  type PublicKernelCircuitPublicInputs,
  type RECURSIVE_PROOF_LENGTH,
  RecursiveProof,
  RollupTypes,
  RootParityInput,
  type RootParityInputs,
  type RootRollupInputs,
  type RootRollupPublicInputs,
  type VERIFICATION_KEY_LENGTH_IN_FIELDS,
  VerificationKeyAsFields,
} from '@aztec/circuits.js';
import { randomBytes } from '@aztec/foundation/crypto';
import { createDebugLogger } from '@aztec/foundation/log';
import { type Tuple } from '@aztec/foundation/serialize';
import {
  ServerCircuitArtifacts,
  type ServerProtocolArtifact,
  convertBaseParityInputsToWitnessMap,
  convertBaseParityOutputsFromWitnessMap,
  convertBaseRollupInputsToWitnessMap,
  convertBaseRollupOutputsFromWitnessMap,
  convertMergeRollupInputsToWitnessMap,
  convertMergeRollupOutputsFromWitnessMap,
  convertPublicTailInputsToWitnessMap,
  convertPublicTailOutputFromWitnessMap,
  convertRootParityInputsToWitnessMap,
  convertRootParityOutputsFromWitnessMap,
  convertRootRollupInputsToWitnessMap,
  convertRootRollupOutputsFromWitnessMap,
} from '@aztec/noir-protocol-circuits-types';
import { NativeACVMSimulator } from '@aztec/simulator';

import { type WitnessMap } from '@noir-lang/types';
import * as fs from 'fs/promises';

import {
  BB_RESULT,
  PROOF_FIELDS_FILENAME,
  PROOF_FILENAME,
  VK_FIELDS_FILENAME,
  VK_FILENAME,
  generateKeyForNoirCircuit,
  generateProof,
  verifyProof,
} from '../bb/execute.js';
import { type CircuitProver, KernelArtifactMapping } from './interface.js';

const logger = createDebugLogger('aztec:bb-prover');

const CIRCUITS_WITHOUT_AGGREGATION: Set<ServerProtocolArtifact> = new Set(['BaseParityArtifact']);
const AGGREGATION_OBJECT_SIZE = 16;
const CIRCUIT_SIZE_INDEX = 3;
const CIRCUIT_PUBLIC_INPUTS_INDEX = 4;
const CIRCUIT_RECURSIVE_INDEX = 5;

export type BBProverConfig = {
  bbBinaryPath: string;
  bbWorkingDirectory: string;
  acvmBinaryPath: string;
  acvmWorkingDirectory: string;
  // list of circuits supported by this prover. defaults to all circuits if empty
  circuitFilter?: ServerProtocolArtifact[];
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
 * Prover implementation that uses barretenberg native proving
 */
export class BBNativeRollupProver implements CircuitProver {
  private verificationKeys: Map<ServerProtocolArtifact, Promise<VerificationKeyData>> = new Map<
    ServerProtocolArtifact,
    Promise<VerificationKeyData>
  >();
  constructor(private config: BBProverConfig) {}

  static async new(config: BBProverConfig) {
    await fs.access(config.acvmBinaryPath, fs.constants.R_OK);
    await fs.mkdir(config.acvmWorkingDirectory, { recursive: true });
    await fs.access(config.bbBinaryPath, fs.constants.R_OK);
    await fs.mkdir(config.bbWorkingDirectory, { recursive: true });
    logger.info(`Using native BB at ${config.bbBinaryPath} and working directory ${config.bbWorkingDirectory}`);
    logger.info(`Using native ACVM at ${config.acvmBinaryPath} and working directory ${config.acvmWorkingDirectory}`);

    return new BBNativeRollupProver(config);
  }

  /**
   * Simulates the base parity circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs of the parity circuit.
   */
  public async getBaseParityProof(inputs: BaseParityInputs): Promise<RootParityInput<typeof RECURSIVE_PROOF_LENGTH>> {
    const witnessMap = convertBaseParityInputsToWitnessMap(inputs);

    const [circuitOutput, proof] = await this.createRecursiveProof<typeof RECURSIVE_PROOF_LENGTH, ParityPublicInputs>(
      witnessMap,
      'BaseParityArtifact',
      convertBaseParityOutputsFromWitnessMap,
    );

    const verificationKey = await this.getVerificationKeyDataForCircuit('BaseParityArtifact');

    const vk = new VerificationKeyAsFields(verificationKey.keyAsFields, verificationKey.hash);

    return new RootParityInput(proof, vk, circuitOutput);
  }

  /**
   * Simulates the root parity circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs of the parity circuit.
   */
  public async getRootParityProof(
    inputs: RootParityInputs,
  ): Promise<RootParityInput<typeof NESTED_RECURSIVE_PROOF_LENGTH>> {
    const witnessMap = convertRootParityInputsToWitnessMap(inputs);

    const [circuitOutput, proof] = await this.createRecursiveProof<
      typeof NESTED_RECURSIVE_PROOF_LENGTH,
      ParityPublicInputs
    >(witnessMap, 'RootParityArtifact', convertRootParityOutputsFromWitnessMap);

    const verificationKey = await this.getVerificationKeyDataForCircuit('RootParityArtifact');

    const vk = new VerificationKeyAsFields(verificationKey.keyAsFields, verificationKey.hash);

    return new RootParityInput(proof, vk, circuitOutput);
  }

  /**
   * Requests that a public kernel circuit be executed and the proof generated
   * @param kernelRequest - The object encapsulating the request for a proof
   * @returns The requested circuit's public inputs and proof
   */
  public async getPublicKernelProof(
    kernelRequest: PublicKernelNonTailRequest,
  ): Promise<PublicInputsAndProof<PublicKernelCircuitPublicInputs>> {
    const kernelOps = KernelArtifactMapping[kernelRequest.type];
    if (kernelOps === undefined) {
      throw new Error(`Unable to prove kernel type ${PublicKernelType[kernelRequest.type]}`);
    }
    const witnessMap = kernelOps.convertInputs(kernelRequest.inputs);

    const [outputWitness, proof] = await this.createProof(witnessMap, kernelOps.artifact);

    const result = kernelOps.convertOutputs(outputWitness);
    return makePublicInputsAndProof(result, proof);
  }

  /**
   * Requests that the public kernel tail circuit be executed and the proof generated
   * @param kernelRequest - The object encapsulating the request for a proof
   * @returns The requested circuit's public inputs and proof
   */
  public async getPublicTailProof(
    kernelRequest: PublicKernelTailRequest,
  ): Promise<PublicInputsAndProof<KernelCircuitPublicInputs>> {
    const witnessMap = convertPublicTailInputsToWitnessMap(kernelRequest.inputs);

    const [outputWitness, proof] = await this.createProof(witnessMap, 'PublicKernelTailArtifact');

    const result = convertPublicTailOutputFromWitnessMap(outputWitness);
    return makePublicInputsAndProof(result, proof);
  }

  /**
   * Simulates the base rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getBaseRollupProof(
    input: BaseRollupInputs,
  ): Promise<PublicInputsAndProof<BaseOrMergeRollupPublicInputs>> {
    const witnessMap = convertBaseRollupInputsToWitnessMap(input);

    const [outputWitness, proof] = await this.createProof(witnessMap, 'BaseRollupArtifact');

    const result = convertBaseRollupOutputsFromWitnessMap(outputWitness);

    return makePublicInputsAndProof(result, proof);
  }
  /**
   * Simulates the merge rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getMergeRollupProof(
    input: MergeRollupInputs,
  ): Promise<PublicInputsAndProof<BaseOrMergeRollupPublicInputs>> {
    // verify both inputs
    await Promise.all(input.previousRollupData.map(prev => this.verifyPreviousRollupProof(prev)));

    const witnessMap = convertMergeRollupInputsToWitnessMap(input);

    const [outputWitness, proof] = await this.createProof(witnessMap, 'MergeRollupArtifact');

    const result = convertMergeRollupOutputsFromWitnessMap(outputWitness);

    return makePublicInputsAndProof(result, proof);
  }

  /**
   * Simulates the root rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getRootRollupProof(input: RootRollupInputs): Promise<PublicInputsAndProof<RootRollupPublicInputs>> {
    // verify the inputs
    await Promise.all(input.previousRollupData.map(prev => this.verifyPreviousRollupProof(prev)));

    const witnessMap = convertRootRollupInputsToWitnessMap(input);

    const [outputWitness, proof] = await this.createProof(witnessMap, 'RootRollupArtifact');

    await this.verifyProof('RootRollupArtifact', proof);

    const result = convertRootRollupOutputsFromWitnessMap(outputWitness);
    return makePublicInputsAndProof(result, proof);
  }

  // TODO(@PhilWindle): Delete when no longer required
  public async createProof(witnessMap: WitnessMap, circuitType: ServerProtocolArtifact): Promise<[WitnessMap, Proof]> {
    // Create random directory to be used for temp files
    const bbWorkingDirectory = `${this.config.bbWorkingDirectory}/${randomBytes(8).toString('hex')}`;
    await fs.mkdir(bbWorkingDirectory, { recursive: true });

    await fs.access(bbWorkingDirectory);

    // Have the ACVM write the partial witness here
    const outputWitnessFile = `${bbWorkingDirectory}/partial-witness.gz`;

    // Generate the partial witness using the ACVM
    // A further temp directory will be created beneath ours and then cleaned up after the partial witness has been copied to our specified location
    const simulator = new NativeACVMSimulator(
      this.config.acvmWorkingDirectory,
      this.config.acvmBinaryPath,
      outputWitnessFile,
    );

    const artifact = ServerCircuitArtifacts[circuitType];

    logger.debug(`Generating witness data for ${circuitType}`);

    const outputWitness = await simulator.simulateCircuit(witnessMap, artifact);

    // Now prove the circuit from the generated witness
    logger.debug(`Proving ${circuitType}...`);

    const provingResult = await generateProof(
      this.config.bbBinaryPath,
      bbWorkingDirectory,
      circuitType,
      artifact,
      outputWitnessFile,
      logger.debug,
    );

    if (provingResult.status === BB_RESULT.FAILURE) {
      logger.error(`Failed to generate proof for ${circuitType}: ${provingResult.reason}`);
      throw new Error(provingResult.reason);
    }

    // Ensure our vk cache is up to date
    await this.updateVerificationKeyAfterProof(provingResult.vkPath!, circuitType);

    // Read the proof and then cleanup up our temporary directory
    const proof = await fs.readFile(`${provingResult.proofPath!}/${PROOF_FILENAME}`);

    await fs.rm(bbWorkingDirectory, { recursive: true, force: true });

    logger.info(`Generated proof for ${circuitType} in ${provingResult.duration} ms, size: ${proof.length} fields`);

    return [outputWitness, new Proof(proof)];
  }

  /**
   * Executes a circuit and returns it's outputs and corresponding proof with embedded aggregation object
   * @param witnessMap - The input witness
   * @param circuitType - The type of circuit to be executed
   * @param convertOutput - Function for parsing the output witness to it's corresponding object
   * @returns The circuits output object and it's proof
   */
  public async createRecursiveProof<PROOF_LENGTH extends number, CircuitOutputType>(
    witnessMap: WitnessMap,
    circuitType: ServerProtocolArtifact,
    convertOutput: (outputWitness: WitnessMap) => CircuitOutputType,
  ): Promise<[CircuitOutputType, RecursiveProof<PROOF_LENGTH>]> {
    // Create random directory to be used for temp files
    const bbWorkingDirectory = `${this.config.bbWorkingDirectory}/${randomBytes(8).toString('hex')}`;
    await fs.mkdir(bbWorkingDirectory, { recursive: true });

    await fs.access(bbWorkingDirectory);

    try {
      // Have the ACVM write the partial witness here
      const outputWitnessFile = `${bbWorkingDirectory}/partial-witness.gz`;

      // Generate the partial witness using the ACVM
      // A further temp directory will be created beneath ours and then cleaned up after the partial witness has been copied to our specified location
      const simulator = new NativeACVMSimulator(
        this.config.acvmWorkingDirectory,
        this.config.acvmBinaryPath,
        outputWitnessFile,
      );

      const artifact = ServerCircuitArtifacts[circuitType];

      logger.debug(`Generating witness data for ${circuitType}`);

      const outputWitness = await simulator.simulateCircuit(witnessMap, artifact);

      const outputType = convertOutput(outputWitness);

      // Now prove the circuit from the generated witness
      logger.debug(`Proving ${circuitType}...`);

      const provingResult = await generateProof(
        this.config.bbBinaryPath,
        bbWorkingDirectory,
        circuitType,
        artifact,
        outputWitnessFile,
        logger.debug,
      );

      if (provingResult.status === BB_RESULT.FAILURE) {
        logger.error(`Failed to generate proof for ${circuitType}: ${provingResult.reason}`);
        throw new Error(provingResult.reason);
      }

      // Ensure our vk cache is up to date
      await this.updateVerificationKeyAfterProof(provingResult.vkPath!, circuitType);

      // Read the proof and then cleanup up our temporary directory
      const proof = await this.readProofAsFields<PROOF_LENGTH>(provingResult.proofPath!, circuitType);

      logger.info(
        `Generated proof for ${circuitType} in ${provingResult.duration} ms, size: ${proof.proof.length} fields`,
      );

      return [outputType, proof];
    } finally {
      await fs.rm(bbWorkingDirectory, { recursive: true, force: true });
    }
  }

  /**
   * Verifies a proof, will generate the verification key if one is not cached internally
   * @param circuitType - The type of circuit whose proof is to be verified
   * @param proof - The proof to be verified
   */
  public async verifyProof(circuitType: ServerProtocolArtifact, proof: Proof) {
    // Create random directory to be used for temp files
    const bbWorkingDirectory = `${this.config.bbWorkingDirectory}/${randomBytes(8).toString('hex')}`;
    await fs.mkdir(bbWorkingDirectory, { recursive: true });

    const proofFileName = `${bbWorkingDirectory}/proof`;
    const verificationKeyPath = `${bbWorkingDirectory}/vk`;
    const verificationKey = await this.getVerificationKeyDataForCircuit(circuitType);

    logger.debug(`Verifying with key: ${verificationKey.hash.toString()}`);

    await fs.writeFile(proofFileName, proof.buffer);
    await fs.writeFile(verificationKeyPath, verificationKey.keyAsBytes);

    const logFunction = (message: string) => {
      logger.debug(`${circuitType} BB out - ${message}`);
    };

    const result = await verifyProof(this.config.bbBinaryPath, proofFileName, verificationKeyPath!, logFunction);

    await fs.rm(bbWorkingDirectory, { recursive: true, force: true });

    if (result.status === BB_RESULT.FAILURE) {
      const errorMessage = `Failed to verify ${circuitType} proof!`;
      throw new Error(errorMessage);
    }

    logger.info(`Successfully verified ${circuitType} proof in ${result.duration} ms`);
  }

  /**
   * Returns the verification key for a circuit, will generate it if not cached internally
   * @param circuitType - The type of circuit for which the verification key is required
   * @returns The verification key
   */
  public async getVerificationKeyForCircuit(circuitType: ServerProtocolArtifact): Promise<VerificationKeyAsFields> {
    const vkData = await this.getVerificationKeyDataForCircuit(circuitType);
    return new VerificationKeyAsFields(vkData.keyAsFields, vkData.hash);
  }

  private async verifyPreviousRollupProof(previousRollupData: PreviousRollupData) {
    const proof = previousRollupData.proof;
    const circuitType =
      previousRollupData.baseOrMergeRollupPublicInputs.rollupType === RollupTypes.Base
        ? 'BaseRollupArtifact'
        : 'MergeRollupArtifact';
    await this.verifyProof(circuitType, proof);
  }

  /**
   * Returns the verification key data for a circuit, will generate and cache it if not cached internally
   * @param circuitType - The type of circuit for which the verification key is required
   * @returns The verification key data
   */
  private async getVerificationKeyDataForCircuit(circuitType: ServerProtocolArtifact): Promise<VerificationKeyData> {
    let promise = this.verificationKeys.get(circuitType);
    if (!promise) {
      promise = generateKeyForNoirCircuit(
        this.config.bbBinaryPath,
        this.config.bbWorkingDirectory,
        circuitType,
        ServerCircuitArtifacts[circuitType],
        'vk',
        logger.debug,
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
  private async updateVerificationKeyAfterProof(filePath: string, circuitType: ServerProtocolArtifact) {
    let promise = this.verificationKeys.get(circuitType);
    if (!promise) {
      promise = this.convertVk(filePath);
      this.verificationKeys.set(circuitType, promise);
    }
    await promise;
  }

  /**
   * Parses and returns the proof data stored at the specified directory
   * @param filePath - The directory containing the proof data
   * @param circuitType - The type of circuit proven
   * @returns The proof
   */
  private async readProofAsFields<PROOF_LENGTH extends number>(
    filePath: string,
    circuitType: ServerProtocolArtifact,
  ): Promise<RecursiveProof<PROOF_LENGTH>> {
    const [binaryProof, proofString] = await Promise.all([
      fs.readFile(`${filePath}/${PROOF_FILENAME}`),
      fs.readFile(`${filePath}/${PROOF_FIELDS_FILENAME}`, { encoding: 'utf-8' }),
    ]);
    const json = JSON.parse(proofString);
    const fields = json.map(Fr.fromString);
    const vkData = await this.verificationKeys.get(circuitType);
    if (!vkData) {
      throw new Error(`Invalid verification key for ${circuitType}`);
    }
    const numPublicInputs = CIRCUITS_WITHOUT_AGGREGATION.has(circuitType)
      ? vkData.numPublicInputs
      : vkData.numPublicInputs - AGGREGATION_OBJECT_SIZE;
    const fieldsWithoutPublicInputs = fields.slice(numPublicInputs);
    logger.debug(
      `Circuit type: ${circuitType}, complete proof length: ${fields.length}, without public inputs: ${fieldsWithoutPublicInputs.length}, num public inputs: ${numPublicInputs}, circuit size: ${vkData.circuitSize}, is recursive: ${vkData.isRecursive}, raw length: ${binaryProof.length}`,
    );
    const proof = new RecursiveProof<PROOF_LENGTH>(fieldsWithoutPublicInputs, new Proof(binaryProof));
    return proof;
  }
}
