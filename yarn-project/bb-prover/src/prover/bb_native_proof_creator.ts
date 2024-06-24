import { type AppCircuitProofOutput, type KernelProofOutput, type ProofCreator } from '@aztec/circuit-types';
import { type CircuitProvingStats, type CircuitWitnessGenerationStats } from '@aztec/circuit-types/stats';
import {
  AGGREGATION_OBJECT_LENGTH,
  Fr,
  NESTED_RECURSIVE_PROOF_LENGTH,
  type PrivateCircuitPublicInputs,
  type PrivateKernelCircuitPublicInputs,
  type PrivateKernelInitCircuitPrivateInputs,
  type PrivateKernelInnerCircuitPrivateInputs,
  type PrivateKernelResetCircuitPrivateInputsVariants,
  type PrivateKernelTailCircuitPrivateInputs,
  type PrivateKernelTailCircuitPublicInputs,
  Proof,
  RECURSIVE_PROOF_LENGTH,
  RecursiveProof,
  type VerificationKeyAsFields,
  type VerificationKeyData,
} from '@aztec/circuits.js';
import { siloNoteHash } from '@aztec/circuits.js/hash';
import { runInDirectory } from '@aztec/foundation/fs';
import { createDebugLogger } from '@aztec/foundation/log';
import { Timer } from '@aztec/foundation/timer';
import {
  ClientCircuitArtifacts,
  type ClientProtocolArtifact,
  PrivateResetTagToArtifactName,
  convertPrivateKernelInitInputsToWitnessMap,
  convertPrivateKernelInitOutputsFromWitnessMap,
  convertPrivateKernelInnerInputsToWitnessMap,
  convertPrivateKernelInnerOutputsFromWitnessMap,
  convertPrivateKernelResetInputsToWitnessMap,
  convertPrivateKernelResetOutputsFromWitnessMap,
  convertPrivateKernelTailForPublicOutputsFromWitnessMap,
  convertPrivateKernelTailInputsToWitnessMap,
  convertPrivateKernelTailOutputsFromWitnessMap,
  convertPrivateKernelTailToPublicInputsToWitnessMap,
} from '@aztec/noir-protocol-circuits-types';
import { WASMSimulator } from '@aztec/simulator';
import { type NoirCompiledCircuit } from '@aztec/types/noir';

import { serializeWitness } from '@noir-lang/noirc_abi';
import { type WitnessMap } from '@noir-lang/types';
import * as fs from 'fs/promises';
import { join } from 'path';

import {
  BB_RESULT,
  PROOF_FIELDS_FILENAME,
  PROOF_FILENAME,
  generateKeyForNoirCircuit,
  generateProof,
  verifyProof,
} from '../bb/execute.js';
import { mapProtocolArtifactNameToCircuitName } from '../stats.js';
import { extractVkData } from '../verification_key/verification_key_data.js';

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
  ): Promise<KernelProofOutput<PrivateKernelCircuitPublicInputs>> {
    return await this.createSafeProof(
      inputs,
      'PrivateKernelInitArtifact',
      convertPrivateKernelInitInputsToWitnessMap,
      convertPrivateKernelInitOutputsFromWitnessMap,
    );
  }

  public async createProofInner(
    inputs: PrivateKernelInnerCircuitPrivateInputs,
  ): Promise<KernelProofOutput<PrivateKernelCircuitPublicInputs>> {
    return await this.createSafeProof(
      inputs,
      'PrivateKernelInnerArtifact',
      convertPrivateKernelInnerInputsToWitnessMap,
      convertPrivateKernelInnerOutputsFromWitnessMap,
    );
  }

  public async createProofReset(
    inputs: PrivateKernelResetCircuitPrivateInputsVariants,
  ): Promise<KernelProofOutput<PrivateKernelCircuitPublicInputs>> {
    return await this.createSafeProof(
      inputs,
      PrivateResetTagToArtifactName[inputs.sizeTag],
      convertPrivateKernelResetInputsToWitnessMap,
      output => convertPrivateKernelResetOutputsFromWitnessMap(output, inputs.sizeTag),
    );
  }

  public async createProofTail(
    inputs: PrivateKernelTailCircuitPrivateInputs,
  ): Promise<KernelProofOutput<PrivateKernelTailCircuitPublicInputs>> {
    if (!inputs.isForPublic()) {
      return await this.createSafeProof(
        inputs,
        'PrivateKernelTailArtifact',
        convertPrivateKernelTailInputsToWitnessMap,
        convertPrivateKernelTailOutputsFromWitnessMap,
      );
    }
    return await this.createSafeProof(
      inputs,
      'PrivateKernelTailToPublicArtifact',
      convertPrivateKernelTailToPublicInputsToWitnessMap,
      convertPrivateKernelTailForPublicOutputsFromWitnessMap,
    );
  }

  public async createAppCircuitProof(
    partialWitness: WitnessMap,
    bytecode: Buffer,
    appCircuitName?: string,
  ): Promise<AppCircuitProofOutput> {
    const operation = async (directory: string) => {
      this.log.debug(`Proving app circuit`);
      const proofOutput = await this.createProof(directory, partialWitness, bytecode, 'App', appCircuitName);
      if (proofOutput.proof.proof.length != RECURSIVE_PROOF_LENGTH) {
        throw new Error(`Incorrect proof length`);
      }
      const proof = proofOutput.proof as RecursiveProof<typeof RECURSIVE_PROOF_LENGTH>;
      const output: AppCircuitProofOutput = {
        proof,
        verificationKey: proofOutput.verificationKey,
      };
      return output;
    };

    return await runInDirectory(this.bbWorkingDirectory, operation);
  }

  /**
   * Verifies a proof, will generate the verification key if one is not cached internally
   * @param circuitType - The type of circuit whose proof is to be verified
   * @param proof - The proof to be verified
   */
  public async verifyProofForProtocolCircuit(circuitType: ClientProtocolArtifact, proof: Proof) {
    const verificationKey = await this.getVerificationKeyDataForCircuit(circuitType);

    this.log.debug(`Verifying with key: ${verificationKey.keyAsFields.hash.toString()}`);

    const logFunction = (message: string) => {
      this.log.debug(`${circuitType} BB out - ${message}`);
    };

    const result = await this.verifyProofFromKey(verificationKey.keyAsBytes, proof, logFunction);

    if (result.status === BB_RESULT.FAILURE) {
      const errorMessage = `Failed to verify ${circuitType} proof!`;
      throw new Error(errorMessage);
    }

    this.log.info(`Successfully verified ${circuitType} proof in ${Math.ceil(result.durationMs)} ms`);
  }

  private async verifyProofFromKey(
    verificationKey: Buffer,
    proof: Proof,
    logFunction: (message: string) => void = () => {},
  ) {
    const operation = async (bbWorkingDirectory: string) => {
      const proofFileName = `${bbWorkingDirectory}/proof`;
      const verificationKeyPath = `${bbWorkingDirectory}/vk`;

      await fs.writeFile(proofFileName, proof.buffer);
      await fs.writeFile(verificationKeyPath, verificationKey);
      return await verifyProof(this.bbBinaryPath, proofFileName, verificationKeyPath!, logFunction);
    };
    return await runInDirectory(this.bbWorkingDirectory, operation);
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
        return extractVkData(result.vkPath!);
      });
      this.verificationKeys.set(circuitType, promise);
    }
    return await promise;
  }

  /**
   * Ensures our verification key cache includes the key data located at the specified directory
   * @param filePath - The directory containing the verification key data files
   * @param circuitType - The type of circuit to which the verification key corresponds
   */
  private async updateVerificationKeyAfterProof(filePath: string, circuitType: ClientProtocolArtifact) {
    let promise = this.verificationKeys.get(circuitType);
    if (!promise) {
      promise = extractVkData(filePath);
      this.log.debug(`Updated verification key for circuit: ${circuitType}`);
      this.verificationKeys.set(circuitType, promise);
    }
    return await promise;
  }

  private async createSafeProof<I extends { toBuffer: () => Buffer }, O extends { toBuffer: () => Buffer }>(
    inputs: I,
    circuitType: ClientProtocolArtifact,
    convertInputs: (inputs: I) => WitnessMap,
    convertOutputs: (outputs: WitnessMap) => O,
  ): Promise<KernelProofOutput<O>> {
    const operation = async (directory: string) => {
      return await this.generateWitnessAndCreateProof(inputs, circuitType, directory, convertInputs, convertOutputs);
    };
    return await runInDirectory(this.bbWorkingDirectory, operation);
  }

  private async generateWitnessAndCreateProof<
    I extends { toBuffer: () => Buffer },
    O extends { toBuffer: () => Buffer },
  >(
    inputs: I,
    circuitType: ClientProtocolArtifact,
    directory: string,
    convertInputs: (inputs: I) => WitnessMap,
    convertOutputs: (outputs: WitnessMap) => O,
  ): Promise<KernelProofOutput<O>> {
    this.log.debug(`Generating witness for ${circuitType}`);
    const compiledCircuit: NoirCompiledCircuit = ClientCircuitArtifacts[circuitType];

    const witnessMap = convertInputs(inputs);
    const timer = new Timer();
    const outputWitness = await this.simulator.simulateCircuit(witnessMap, compiledCircuit);
    const output = convertOutputs(outputWitness);

    this.log.debug(`Generated witness for ${circuitType}`, {
      eventName: 'circuit-witness-generation',
      circuitName: mapProtocolArtifactNameToCircuitName(circuitType),
      duration: timer.ms(),
      inputSize: inputs.toBuffer().length,
      outputSize: output.toBuffer().length,
    } satisfies CircuitWitnessGenerationStats);

    const proofOutput = await this.createProof(
      directory,
      outputWitness,
      Buffer.from(compiledCircuit.bytecode, 'base64'),
      circuitType,
    );
    if (proofOutput.proof.proof.length != NESTED_RECURSIVE_PROOF_LENGTH) {
      throw new Error(`Incorrect proof length`);
    }
    const nestedProof = proofOutput.proof as RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>;

    const kernelOutput: KernelProofOutput<O> = {
      publicInputs: output,
      proof: nestedProof,
      verificationKey: proofOutput.verificationKey,
    };
    return kernelOutput;
  }

  private async createProof(
    directory: string,
    partialWitness: WitnessMap,
    bytecode: Buffer,
    circuitType: ClientProtocolArtifact | 'App',
    appCircuitName?: string,
  ): Promise<{
    proof: RecursiveProof<typeof RECURSIVE_PROOF_LENGTH> | RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>;
    verificationKey: VerificationKeyAsFields;
  }> {
    const compressedBincodedWitness = serializeWitness(partialWitness);

    const inputsWitnessFile = join(directory, 'witness.gz');

    await fs.writeFile(inputsWitnessFile, compressedBincodedWitness);

    this.log.debug(`Written ${inputsWitnessFile}`);

    const dbgCircuitName = appCircuitName ? `(${appCircuitName})` : '';
    this.log.info(`Proving ${circuitType}${dbgCircuitName} circuit...`);

    const timer = new Timer();

    const provingResult = await generateProof(
      this.bbBinaryPath,
      directory,
      circuitType,
      bytecode,
      inputsWitnessFile,
      this.log.debug,
    );

    if (provingResult.status === BB_RESULT.FAILURE) {
      this.log.error(`Failed to generate proof for ${circuitType}${dbgCircuitName}: ${provingResult.reason}`);
      throw new Error(provingResult.reason);
    }

    this.log.info(`Generated ${circuitType}${dbgCircuitName} circuit proof in ${Math.ceil(timer.ms())} ms`);

    if (circuitType === 'App') {
      const vkData = await extractVkData(directory);
      const proof = await this.readProofAsFields<typeof RECURSIVE_PROOF_LENGTH>(directory, circuitType, vkData);

      this.log.debug(`Generated proof`, {
        eventName: 'circuit-proving',
        circuitName: 'app-circuit',
        duration: provingResult.durationMs,
        inputSize: compressedBincodedWitness.length,
        proofSize: proof.binaryProof.buffer.length,
        appCircuitName,
        circuitSize: vkData.circuitSize,
        numPublicInputs: vkData.numPublicInputs,
      } as CircuitProvingStats);

      return { proof, verificationKey: vkData.keyAsFields };
    }

    const vkData = await this.updateVerificationKeyAfterProof(directory, circuitType);

    const proof = await this.readProofAsFields<typeof NESTED_RECURSIVE_PROOF_LENGTH>(directory, circuitType, vkData);

    await this.verifyProofForProtocolCircuit(circuitType, proof.binaryProof);

    this.log.debug(`Generated proof`, {
      circuitName: mapProtocolArtifactNameToCircuitName(circuitType),
      duration: provingResult.durationMs,
      eventName: 'circuit-proving',
      inputSize: compressedBincodedWitness.length,
      proofSize: proof.binaryProof.buffer.length,
      circuitSize: vkData.circuitSize,
      numPublicInputs: vkData.numPublicInputs,
    } as CircuitProvingStats);

    return { proof, verificationKey: vkData.keyAsFields };
  }

  /**
   * Parses and returns the proof data stored at the specified directory
   * @param filePath - The directory containing the proof data
   * @param circuitType - The type of circuit proven
   * @returns The proof
   */
  private async readProofAsFields<PROOF_LENGTH extends number>(
    filePath: string,
    circuitType: ClientProtocolArtifact | 'App',
    vkData: VerificationKeyData,
  ): Promise<RecursiveProof<PROOF_LENGTH>> {
    const [binaryProof, proofString] = await Promise.all([
      fs.readFile(`${filePath}/${PROOF_FILENAME}`),
      fs.readFile(`${filePath}/${PROOF_FIELDS_FILENAME}`, { encoding: 'utf-8' }),
    ]);
    const json = JSON.parse(proofString);
    const fields = json.map(Fr.fromString);
    const numPublicInputs =
      circuitType === 'App' ? vkData.numPublicInputs : vkData.numPublicInputs - AGGREGATION_OBJECT_LENGTH;
    const fieldsWithoutPublicInputs = fields.slice(numPublicInputs);
    this.log.debug(
      `Circuit type: ${circuitType}, complete proof length: ${fields.length}, without public inputs: ${fieldsWithoutPublicInputs.length}, num public inputs: ${numPublicInputs}, circuit size: ${vkData.circuitSize}, is recursive: ${vkData.isRecursive}, raw length: ${binaryProof.length}`,
    );
    const proof = new RecursiveProof<PROOF_LENGTH>(
      fieldsWithoutPublicInputs,
      new Proof(binaryProof, vkData.numPublicInputs),
      true,
    );
    return proof;
  }
}
