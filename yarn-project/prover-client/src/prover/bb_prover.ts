/* eslint-disable require-await */
import {
  type BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  type MergeRollupInputs,
  type ParityPublicInputs,
  type PreviousRollupData,
  Proof,
  RollupTypes,
  type RootParityInputs,
  type RootRollupInputs,
  type RootRollupPublicInputs,
} from '@aztec/circuits.js';
import { randomBytes } from '@aztec/foundation/crypto';
import { createDebugLogger } from '@aztec/foundation/log';
import {
  ServerCircuitArtifacts,
  type ServerProtocolArtifact,
  convertBaseParityInputsToWitnessMap,
  convertBaseParityOutputsFromWitnessMap,
  convertBaseRollupInputsToWitnessMap,
  convertBaseRollupOutputsFromWitnessMap,
  convertMergeRollupInputsToWitnessMap,
  convertMergeRollupOutputsFromWitnessMap,
  convertRootParityInputsToWitnessMap,
  convertRootParityOutputsFromWitnessMap,
  convertRootRollupInputsToWitnessMap,
  convertRootRollupOutputsFromWitnessMap,
} from '@aztec/noir-protocol-circuits-types';
import { NativeACVMSimulator } from '@aztec/simulator';

import { type WitnessMap } from '@noir-lang/types';
import * as fs from 'fs/promises';

import { BB_RESULT, generateKeyForNoirCircuit, generateProof, verifyProof } from '../bb/execute.js';
import { type CircuitProver } from './interface.js';

const logger = createDebugLogger('aztec:bb-prover');

export type BBProverConfig = {
  bbBinaryPath: string;
  bbWorkingDirectory: string;
  acvmBinaryPath: string;
  acvmWorkingDirectory: string;
};

/**
 * Prover implementation that uses barretenberg native proving
 */
export class BBNativeRollupProver implements CircuitProver {
  private verificationKeyDirectories: Map<ServerProtocolArtifact, string> = new Map<ServerProtocolArtifact, string>();
  constructor(private config: BBProverConfig) {}

  static async new(config: BBProverConfig) {
    await fs.access(config.acvmBinaryPath, fs.constants.R_OK);
    await fs.mkdir(config.acvmWorkingDirectory, { recursive: true });
    await fs.access(config.bbBinaryPath, fs.constants.R_OK);
    await fs.mkdir(config.bbWorkingDirectory, { recursive: true });
    logger.info(`Using native BB at ${config.bbBinaryPath} and working directory ${config.bbWorkingDirectory}`);
    logger.info(`Using native ACVM at ${config.acvmBinaryPath} and working directory ${config.acvmWorkingDirectory}`);

    const prover = new BBNativeRollupProver(config);
    await prover.init();
    return prover;
  }

  /**
   * Simulates the base parity circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs of the parity circuit.
   */
  public async getBaseParityProof(inputs: BaseParityInputs): Promise<[ParityPublicInputs, Proof]> {
    const witnessMap = convertBaseParityInputsToWitnessMap(inputs);

    const [outputWitness, proof] = await this.createProof(witnessMap, 'BaseParityArtifact');

    const result = convertBaseParityOutputsFromWitnessMap(outputWitness);

    return Promise.resolve([result, proof]);
  }

  /**
   * Simulates the root parity circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs of the parity circuit.
   */
  public async getRootParityProof(inputs: RootParityInputs): Promise<[ParityPublicInputs, Proof]> {
    // verify all base parity inputs
    await Promise.all(inputs.children.map(child => this.verifyProof('BaseParityArtifact', child.proof)));

    const witnessMap = convertRootParityInputsToWitnessMap(inputs);

    const [outputWitness, proof] = await this.createProof(witnessMap, 'RootParityArtifact');

    const result = convertRootParityOutputsFromWitnessMap(outputWitness);

    return Promise.resolve([result, proof]);
  }

  /**
   * Simulates the base rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getBaseRollupProof(input: BaseRollupInputs): Promise<[BaseOrMergeRollupPublicInputs, Proof]> {
    const witnessMap = convertBaseRollupInputsToWitnessMap(input);

    const [outputWitness, proof] = await this.createProof(witnessMap, 'BaseRollupArtifact');

    const result = convertBaseRollupOutputsFromWitnessMap(outputWitness);

    return Promise.resolve([result, proof]);
  }
  /**
   * Simulates the merge rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getMergeRollupProof(input: MergeRollupInputs): Promise<[BaseOrMergeRollupPublicInputs, Proof]> {
    // verify both inputs
    await Promise.all(input.previousRollupData.map(prev => this.verifyPreviousRollupProof(prev)));

    const witnessMap = convertMergeRollupInputsToWitnessMap(input);

    const [outputWitness, proof] = await this.createProof(witnessMap, 'MergeRollupArtifact');

    const result = convertMergeRollupOutputsFromWitnessMap(outputWitness);

    return Promise.resolve([result, proof]);
  }

  /**
   * Simulates the root rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getRootRollupProof(input: RootRollupInputs): Promise<[RootRollupPublicInputs, Proof]> {
    // verify the inputs
    await Promise.all(input.previousRollupData.map(prev => this.verifyPreviousRollupProof(prev)));

    const witnessMap = convertRootRollupInputsToWitnessMap(input);

    const [outputWitness, proof] = await this.createProof(witnessMap, 'RootRollupArtifact');

    await this.verifyProof('RootRollupArtifact', proof);

    const result = convertRootRollupOutputsFromWitnessMap(outputWitness);
    return Promise.resolve([result, proof]);
  }

  private async init() {
    const promises = [];
    for (const circuitName in ServerCircuitArtifacts) {
      const verificationKeyPromise = generateKeyForNoirCircuit(
        this.config.bbBinaryPath,
        this.config.bbWorkingDirectory,
        circuitName,
        ServerCircuitArtifacts[circuitName as ServerProtocolArtifact],
        'vk',
        logger.debug,
      ).then(result => {
        if (result.status == BB_RESULT.FAILURE) {
          logger.error(`Failed to generate verification key for circuit ${circuitName}`);
          return;
        }
        logger.info(`Generated verification key for circuit ${circuitName} at ${result.path!}`);
        this.verificationKeyDirectories.set(circuitName as ServerProtocolArtifact, result.path!);
      });
      promises.push(verificationKeyPromise);
    }
    await Promise.all(promises);
  }

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

    // Read the proof and then cleanup up our temporary directory
    const proofBuffer = await fs.readFile(provingResult.path!);

    await fs.rm(bbWorkingDirectory, { recursive: true, force: true });

    logger.info(
      `Generated proof for ${circuitType} in ${provingResult.duration} ms, size: ${proofBuffer.length} bytes`,
    );

    return [outputWitness, new Proof(proofBuffer)];
  }

  public async verifyProof(circuitType: ServerProtocolArtifact, proof: Proof) {
    // Create random directory to be used for temp files
    const bbWorkingDirectory = `${this.config.bbWorkingDirectory}/${randomBytes(8).toString('hex')}`;
    await fs.mkdir(bbWorkingDirectory, { recursive: true });

    const proofFileName = `${bbWorkingDirectory}/proof`;
    const verificationKeyPath = this.verificationKeyDirectories.get(circuitType);

    await fs.writeFile(proofFileName, proof.buffer);

    const result = await verifyProof(this.config.bbBinaryPath, proofFileName, verificationKeyPath!, logger.debug);

    await fs.rm(bbWorkingDirectory, { recursive: true, force: true });

    if (result.status === BB_RESULT.FAILURE) {
      const errorMessage = `Failed to verify ${circuitType} proof!`;
      throw new Error(errorMessage);
    }

    logger.info(`Successfully verified ${circuitType} proof in ${result.duration} ms`);
  }

  private async verifyPreviousRollupProof(previousRollupData: PreviousRollupData) {
    const proof = previousRollupData.proof;
    const circuitType =
      previousRollupData.baseOrMergeRollupPublicInputs.rollupType === RollupTypes.Base
        ? 'BaseRollupArtifact'
        : 'MergeRollupArtifact';
    await this.verifyProof(circuitType, proof);
  }
}
