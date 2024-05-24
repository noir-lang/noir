/* eslint-disable require-await */
import {
  type PublicInputsAndProof,
  type PublicKernelNonTailRequest,
  type PublicKernelTailRequest,
  PublicKernelType,
  type ServerCircuitProver,
  makePublicInputsAndProof,
} from '@aztec/circuit-types';
import { type CircuitProvingStats, type CircuitWitnessGenerationStats } from '@aztec/circuit-types/stats';
import {
  AGGREGATION_OBJECT_LENGTH,
  type BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  Fr,
  type KernelCircuitPublicInputs,
  type MergeRollupInputs,
  NESTED_RECURSIVE_PROOF_LENGTH,
  Proof,
  type PublicKernelCircuitPublicInputs,
  RECURSIVE_PROOF_LENGTH,
  RecursiveProof,
  RootParityInput,
  type RootParityInputs,
  type RootRollupInputs,
  type RootRollupPublicInputs,
  type VerificationKeyAsFields,
  type VerificationKeyData,
  makeRecursiveProofFromBinary,
} from '@aztec/circuits.js';
import { runInDirectory } from '@aztec/foundation/fs';
import { createDebugLogger } from '@aztec/foundation/log';
import { Timer } from '@aztec/foundation/timer';
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
  type BBSuccess,
  BB_RESULT,
  PROOF_FIELDS_FILENAME,
  PROOF_FILENAME,
  generateKeyForNoirCircuit,
  generateProof,
  verifyProof,
  writeProofAsFields,
} from '../bb/execute.js';
import { PublicKernelArtifactMapping } from '../mappings/mappings.js';
import { mapProtocolArtifactNameToCircuitName } from '../stats.js';
import { extractVkData } from '../verification_key/verification_key_data.js';

const logger = createDebugLogger('aztec:bb-prover');

const CIRCUITS_WITHOUT_AGGREGATION: Set<ServerProtocolArtifact> = new Set(['BaseParityArtifact']);

export type BBProverConfig = {
  bbBinaryPath: string;
  bbWorkingDirectory: string;
  acvmBinaryPath: string;
  acvmWorkingDirectory: string;
  // list of circuits supported by this prover. defaults to all circuits if empty
  circuitFilter?: ServerProtocolArtifact[];
};

/**
 * Prover implementation that uses barretenberg native proving
 */
export class BBNativeRollupProver implements ServerCircuitProver {
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
    const { circuitOutput, proof } = await this.createRecursiveProof(
      inputs,
      'BaseParityArtifact',
      RECURSIVE_PROOF_LENGTH,
      convertBaseParityInputsToWitnessMap,
      convertBaseParityOutputsFromWitnessMap,
    );

    const verificationKey = await this.getVerificationKeyDataForCircuit('BaseParityArtifact');

    return new RootParityInput(proof, verificationKey.keyAsFields, circuitOutput);
  }

  /**
   * Simulates the root parity circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs of the parity circuit.
   */
  public async getRootParityProof(
    inputs: RootParityInputs,
  ): Promise<RootParityInput<typeof NESTED_RECURSIVE_PROOF_LENGTH>> {
    const { circuitOutput, proof } = await this.createRecursiveProof(
      inputs,
      'RootParityArtifact',
      NESTED_RECURSIVE_PROOF_LENGTH,
      convertRootParityInputsToWitnessMap,
      convertRootParityOutputsFromWitnessMap,
    );

    const verificationKey = await this.getVerificationKeyDataForCircuit('RootParityArtifact');

    return new RootParityInput(proof, verificationKey.keyAsFields, circuitOutput);
  }

  /**
   * Requests that a public kernel circuit be executed and the proof generated
   * @param kernelRequest - The object encapsulating the request for a proof
   * @returns The requested circuit's public inputs and proof
   */
  public async getPublicKernelProof(
    kernelRequest: PublicKernelNonTailRequest,
  ): Promise<PublicInputsAndProof<PublicKernelCircuitPublicInputs>> {
    const kernelOps = PublicKernelArtifactMapping[kernelRequest.type];
    if (kernelOps === undefined) {
      throw new Error(`Unable to prove kernel type ${PublicKernelType[kernelRequest.type]}`);
    }

    // We may need to convert the recursive proof into fields format
    kernelRequest.inputs.previousKernel.proof = await this.ensureValidProof(
      kernelRequest.inputs.previousKernel.proof,
      kernelOps.artifact,
      kernelRequest.inputs.previousKernel.vk,
    );

    await this.verifyWithKey(
      kernelRequest.inputs.previousKernel.vk,
      kernelRequest.inputs.previousKernel.proof.binaryProof,
    );

    const { circuitOutput, proof } = await this.createRecursiveProof(
      kernelRequest.inputs,
      kernelOps.artifact,
      NESTED_RECURSIVE_PROOF_LENGTH,
      kernelOps.convertInputs,
      kernelOps.convertOutputs,
    );

    const verificationKey = await this.getVerificationKeyDataForCircuit(kernelOps.artifact);

    return makePublicInputsAndProof(circuitOutput, proof, verificationKey);
  }

  /**
   * Requests that the public kernel tail circuit be executed and the proof generated
   * @param kernelRequest - The object encapsulating the request for a proof
   * @returns The requested circuit's public inputs and proof
   */
  public async getPublicTailProof(
    kernelRequest: PublicKernelTailRequest,
  ): Promise<PublicInputsAndProof<KernelCircuitPublicInputs>> {
    const { circuitOutput, proof } = await this.createRecursiveProof(
      kernelRequest.inputs,
      'PublicKernelTailArtifact',
      NESTED_RECURSIVE_PROOF_LENGTH,
      convertPublicTailInputsToWitnessMap,
      convertPublicTailOutputFromWitnessMap,
    );

    const verificationKey = await this.getVerificationKeyDataForCircuit('PublicKernelTailArtifact');

    return makePublicInputsAndProof(circuitOutput, proof, verificationKey);
  }

  /**
   * Simulates the base rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getBaseRollupProof(
    input: BaseRollupInputs,
  ): Promise<PublicInputsAndProof<BaseOrMergeRollupPublicInputs>> {
    // We may need to convert the recursive proof into fields format
    input.kernelData.proof = await this.ensureValidProof(
      input.kernelData.proof,
      'BaseRollupArtifact',
      input.kernelData.vk,
    );

    const { circuitOutput, proof } = await this.createRecursiveProof(
      input,
      'BaseRollupArtifact',
      NESTED_RECURSIVE_PROOF_LENGTH,
      convertBaseRollupInputsToWitnessMap,
      convertBaseRollupOutputsFromWitnessMap,
    );

    const verificationKey = await this.getVerificationKeyDataForCircuit('BaseRollupArtifact');

    return makePublicInputsAndProof(circuitOutput, proof, verificationKey);
  }
  /**
   * Simulates the merge rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getMergeRollupProof(
    input: MergeRollupInputs,
  ): Promise<PublicInputsAndProof<BaseOrMergeRollupPublicInputs>> {
    const { circuitOutput, proof } = await this.createRecursiveProof(
      input,
      'MergeRollupArtifact',
      NESTED_RECURSIVE_PROOF_LENGTH,
      convertMergeRollupInputsToWitnessMap,
      convertMergeRollupOutputsFromWitnessMap,
    );

    const verificationKey = await this.getVerificationKeyDataForCircuit('MergeRollupArtifact');

    return makePublicInputsAndProof(circuitOutput, proof, verificationKey);
  }

  /**
   * Simulates the root rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getRootRollupProof(input: RootRollupInputs): Promise<PublicInputsAndProof<RootRollupPublicInputs>> {
    const { circuitOutput, proof } = await this.createProof(
      input,
      'RootRollupArtifact',
      convertRootRollupInputsToWitnessMap,
      convertRootRollupOutputsFromWitnessMap,
    );

    const recursiveProof = makeRecursiveProofFromBinary(proof, NESTED_RECURSIVE_PROOF_LENGTH);

    const verificationKey = await this.getVerificationKeyDataForCircuit('RootRollupArtifact');

    await this.verifyProof('RootRollupArtifact', proof);

    return makePublicInputsAndProof(circuitOutput, recursiveProof, verificationKey);
  }

  private async generateProofWithBB<
    Input extends { toBuffer: () => Buffer },
    Output extends { toBuffer: () => Buffer },
  >(
    input: Input,
    circuitType: ServerProtocolArtifact,
    convertInput: (input: Input) => WitnessMap,
    convertOutput: (outputWitness: WitnessMap) => Output,
    workingDirectory: string,
  ): Promise<{ circuitOutput: Output; vkData: VerificationKeyData; provingResult: BBSuccess }> {
    // Have the ACVM write the partial witness here
    const outputWitnessFile = `${workingDirectory}/partial-witness.gz`;

    // Generate the partial witness using the ACVM
    // A further temp directory will be created beneath ours and then cleaned up after the partial witness has been copied to our specified location
    const simulator = new NativeACVMSimulator(
      this.config.acvmWorkingDirectory,
      this.config.acvmBinaryPath,
      outputWitnessFile,
    );

    const artifact = ServerCircuitArtifacts[circuitType];

    logger.debug(`Generating witness data for ${circuitType}`);

    const inputWitness = convertInput(input);
    const timer = new Timer();
    const outputWitness = await simulator.simulateCircuit(inputWitness, artifact);
    const witnessGenerationDuration = timer.ms();
    const output = convertOutput(outputWitness);
    logger.debug(`Generated witness`, {
      circuitName: mapProtocolArtifactNameToCircuitName(circuitType),
      duration: witnessGenerationDuration,
      inputSize: input.toBuffer().length,
      outputSize: output.toBuffer().length,
      eventName: 'circuit-witness-generation',
    } satisfies CircuitWitnessGenerationStats);

    // Now prove the circuit from the generated witness
    logger.debug(`Proving ${circuitType}...`);

    const provingResult = await generateProof(
      this.config.bbBinaryPath,
      workingDirectory,
      circuitType,
      Buffer.from(artifact.bytecode, 'base64'),
      outputWitnessFile,
      logger.debug,
    );

    if (provingResult.status === BB_RESULT.FAILURE) {
      logger.error(`Failed to generate proof for ${circuitType}: ${provingResult.reason}`);
      throw new Error(provingResult.reason);
    }

    // Ensure our vk cache is up to date
    const vkData = await this.updateVerificationKeyAfterProof(provingResult.vkPath!, circuitType);

    return {
      circuitOutput: output,
      vkData,
      provingResult,
    };
  }

  private async createProof<Input extends { toBuffer: () => Buffer }, Output extends { toBuffer: () => Buffer }>(
    input: Input,
    circuitType: ServerProtocolArtifact,
    convertInput: (input: Input) => WitnessMap,
    convertOutput: (outputWitness: WitnessMap) => Output,
  ): Promise<{ circuitOutput: Output; proof: Proof }> {
    const operation = async (bbWorkingDirectory: string) => {
      const {
        provingResult,
        vkData,
        circuitOutput: output,
      } = await this.generateProofWithBB(input, circuitType, convertInput, convertOutput, bbWorkingDirectory);

      // Read the binary proof
      const rawProof = await fs.readFile(`${provingResult.proofPath!}/${PROOF_FILENAME}`);

      const proof = new Proof(rawProof, vkData.numPublicInputs);
      logger.info(
        `Generated proof for ${circuitType} in ${provingResult.duration} ms, size: ${proof.buffer.length} fields`,
        {
          circuitName: mapProtocolArtifactNameToCircuitName(circuitType),
          // does not include reading the proof from disk
          duration: provingResult.duration,
          proofSize: proof.buffer.length,
          eventName: 'circuit-proving',
          // circuitOutput is the partial witness that became the input to the proof
          inputSize: output.toBuffer().length,
          circuitSize: vkData.circuitSize,
          numPublicInputs: vkData.numPublicInputs,
        } satisfies CircuitProvingStats,
      );

      return { circuitOutput: output, proof };
    };
    return await runInDirectory(this.config.bbWorkingDirectory, operation);
  }

  /**
   * Executes a circuit and returns it's outputs and corresponding proof with embedded aggregation object
   * @param witnessMap - The input witness
   * @param circuitType - The type of circuit to be executed
   * @param proofLength - The length of the proof to be generated. This is a dummy parameter to aid in type checking
   * @param convertInput - Function for mapping the input object to a witness map.
   * @param convertOutput - Function for parsing the output witness to it's corresponding object
   * @returns The circuits output object and it's proof
   */
  private async createRecursiveProof<
    PROOF_LENGTH extends number,
    CircuitInputType extends { toBuffer: () => Buffer },
    CircuitOutputType extends { toBuffer: () => Buffer },
  >(
    input: CircuitInputType,
    circuitType: ServerProtocolArtifact,
    proofLength: PROOF_LENGTH,
    convertInput: (input: CircuitInputType) => WitnessMap,
    convertOutput: (outputWitness: WitnessMap) => CircuitOutputType,
  ): Promise<{ circuitOutput: CircuitOutputType; proof: RecursiveProof<PROOF_LENGTH> }> {
    const operation = async (bbWorkingDirectory: string) => {
      const {
        provingResult,
        vkData,
        circuitOutput: output,
      } = await this.generateProofWithBB(input, circuitType, convertInput, convertOutput, bbWorkingDirectory);

      // Read the proof as fields
      const proof = await this.readProofAsFields(provingResult.proofPath!, circuitType, proofLength);

      logger.info(
        `Generated proof for ${circuitType} in ${provingResult.duration} ms, size: ${proof.proof.length} fields`,
        {
          circuitName: mapProtocolArtifactNameToCircuitName(circuitType),
          circuitSize: vkData.circuitSize,
          duration: provingResult.duration,
          inputSize: output.toBuffer().length,
          proofSize: proof.binaryProof.buffer.length,
          eventName: 'circuit-proving',
          numPublicInputs: vkData.numPublicInputs,
        } satisfies CircuitProvingStats,
      );

      return {
        circuitOutput: output,
        proof,
      };
    };
    return await runInDirectory(this.config.bbWorkingDirectory, operation);
  }

  /**
   * Verifies a proof, will generate the verification key if one is not cached internally
   * @param circuitType - The type of circuit whose proof is to be verified
   * @param proof - The proof to be verified
   */
  public async verifyProof(circuitType: ServerProtocolArtifact, proof: Proof) {
    const operation = async (bbWorkingDirectory: string) => {
      const proofFileName = `${bbWorkingDirectory}/proof`;
      const verificationKeyPath = `${bbWorkingDirectory}/vk`;
      const verificationKey = await this.getVerificationKeyDataForCircuit(circuitType);

      await fs.writeFile(proofFileName, proof.buffer);
      await fs.writeFile(verificationKeyPath, verificationKey.keyAsBytes);

      const logFunction = (message: string) => {
        logger.debug(`${circuitType} BB out - ${message}`);
      };

      const result = await verifyProof(this.config.bbBinaryPath, proofFileName, verificationKeyPath!, logFunction);

      if (result.status === BB_RESULT.FAILURE) {
        const errorMessage = `Failed to verify ${circuitType} proof!`;
        throw new Error(errorMessage);
      }

      logger.info(`Successfully verified ${circuitType} proof in ${result.duration} ms`);
    };

    await runInDirectory(this.config.bbWorkingDirectory, operation);
  }

  /**
   * Verifies a proof using a provided verification key
   * @param circuitType - The type of circuit whose proof is to be verified
   * @param proof - The proof to be verified
   */
  public async verifyWithKey(verificationKey: VerificationKeyData, proof: Proof) {
    const operation = async (bbWorkingDirectory: string) => {
      const proofFileName = `${bbWorkingDirectory}/proof`;
      const verificationKeyPath = `${bbWorkingDirectory}/vk`;

      await fs.writeFile(proofFileName, proof.buffer);
      await fs.writeFile(verificationKeyPath, verificationKey.keyAsBytes);

      const logFunction = (message: string) => {
        logger.debug(`BB out - ${message}`);
      };

      const result = await verifyProof(this.config.bbBinaryPath, proofFileName, verificationKeyPath!, logFunction);

      if (result.status === BB_RESULT.FAILURE) {
        const errorMessage = `Failed to verify proof from key!`;
        throw new Error(errorMessage);
      }

      logger.debug(`Successfully verified proof from key in ${result.duration} ms`);
    };

    await runInDirectory(this.config.bbWorkingDirectory, operation);
  }

  /**
   * Returns the verification key for a circuit, will generate it if not cached internally
   * @param circuitType - The type of circuit for which the verification key is required
   * @returns The verification key
   */
  public async getVerificationKeyForCircuit(circuitType: ServerProtocolArtifact): Promise<VerificationKeyAsFields> {
    const vkData = await this.getVerificationKeyDataForCircuit(circuitType);
    return vkData.clone().keyAsFields;
  }

  /**
   * Will check a recursive proof argument for validity of it's 'fields' format of proof and convert if required
   * @param proof - The input proof that may need converting
   * @returns - The valid proof
   */
  public async ensureValidProof(
    proof: RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>,
    circuit: ServerProtocolArtifact,
    vk: VerificationKeyData,
  ) {
    // If the 'fields' proof is already valid then simply return
    // This will be false for proofs coming from clients
    if (proof.fieldsValid) {
      return proof;
    }

    const operation = async (bbWorkingDirectory: string) => {
      const numPublicInputs = vk.numPublicInputs - AGGREGATION_OBJECT_LENGTH;
      const proofFullFilename = `${bbWorkingDirectory}/${PROOF_FILENAME}`;
      const vkFullFilename = `${bbWorkingDirectory}/vk`;

      logger.debug(
        `Converting proof to fields format for circuit ${circuit}, directory ${bbWorkingDirectory}, num public inputs: ${vk.numPublicInputs}, proof length ${proof.binaryProof.buffer.length}, vk length ${vk.keyAsBytes.length}`,
      );

      await fs.writeFile(proofFullFilename, proof.binaryProof.buffer);
      await fs.writeFile(vkFullFilename, vk.keyAsBytes);

      const logFunction = (message: string) => {
        logger.debug(`${circuit} BB out - ${message}`);
      };

      const result = await writeProofAsFields(
        this.config.bbBinaryPath,
        bbWorkingDirectory,
        PROOF_FILENAME,
        vkFullFilename,
        logFunction,
      );

      if (result.status === BB_RESULT.FAILURE) {
        const errorMessage = `Failed to convert ${circuit} proof to fields, ${result.reason}`;
        throw new Error(errorMessage);
      }

      const proofString = await fs.readFile(`${bbWorkingDirectory}/${PROOF_FIELDS_FILENAME}`, { encoding: 'utf-8' });
      const json = JSON.parse(proofString);
      const fields = json.slice(numPublicInputs).map(Fr.fromString);
      return new RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>(
        fields,
        new Proof(proof.binaryProof.buffer, vk.numPublicInputs),
        true,
      );
    };
    return await runInDirectory(this.config.bbWorkingDirectory, operation);
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
        return extractVkData(result.vkPath!);
      });
      this.verificationKeys.set(circuitType, promise);
    }
    const vk = await promise;
    return vk.clone();
  }

  /**
   * Ensures our verification key cache includes the key data located at the specified directory
   * @param filePath - The directory containing the verification key data files
   * @param circuitType - The type of circuit to which the verification key corresponds
   */
  private async updateVerificationKeyAfterProof(
    filePath: string,
    circuitType: ServerProtocolArtifact,
  ): Promise<VerificationKeyData> {
    let promise = this.verificationKeys.get(circuitType);
    if (!promise) {
      promise = extractVkData(filePath);
      this.verificationKeys.set(circuitType, promise);
    }
    return promise;
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
    proofLength: PROOF_LENGTH,
  ): Promise<RecursiveProof<PROOF_LENGTH>> {
    const [binaryProof, proofString] = await Promise.all([
      fs.readFile(`${filePath}/${PROOF_FILENAME}`),
      fs.readFile(`${filePath}/${PROOF_FIELDS_FILENAME}`, { encoding: 'utf-8' }),
    ]);
    const json = JSON.parse(proofString);
    const vkData = await this.verificationKeys.get(circuitType);
    if (!vkData) {
      throw new Error(`Invalid verification key for ${circuitType}`);
    }
    const numPublicInputs = CIRCUITS_WITHOUT_AGGREGATION.has(circuitType)
      ? vkData.numPublicInputs
      : vkData.numPublicInputs - AGGREGATION_OBJECT_LENGTH;
    const fieldsWithoutPublicInputs = json.slice(numPublicInputs).map(Fr.fromString);
    logger.debug(
      `Circuit type: ${circuitType}, complete proof length: ${json.length}, without public inputs: ${fieldsWithoutPublicInputs.length}, num public inputs: ${numPublicInputs}, circuit size: ${vkData.circuitSize}, is recursive: ${vkData.isRecursive}, raw length: ${binaryProof.length}`,
    );
    const proof = new RecursiveProof<PROOF_LENGTH>(
      fieldsWithoutPublicInputs,
      new Proof(binaryProof, numPublicInputs),
      true,
    );
    if (proof.proof.length !== proofLength) {
      throw new Error("Proof length doesn't match expected length");
    }

    return proof;
  }
}
