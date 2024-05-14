import { type AppCircuitProofOutput, type KernelProofOutput, type ProofCreator } from '@aztec/circuit-types';
import {
  Fr,
  NESTED_RECURSIVE_PROOF_LENGTH,
  type PrivateCircuitPublicInputs,
  type PrivateKernelCircuitPublicInputs,
  type PrivateKernelInitCircuitPrivateInputs,
  type PrivateKernelInnerCircuitPrivateInputs,
  type PrivateKernelTailCircuitPrivateInputs,
  type PrivateKernelTailCircuitPublicInputs,
  Proof,
  RECURSIVE_PROOF_LENGTH,
  RecursiveProof,
  type VERIFICATION_KEY_LENGTH_IN_FIELDS,
  VerificationKeyAsFields,
} from '@aztec/circuits.js';
import { siloNoteHash } from '@aztec/circuits.js/hash';
import { randomBytes } from '@aztec/foundation/crypto';
import { createDebugLogger } from '@aztec/foundation/log';
import { type Tuple } from '@aztec/foundation/serialize';
import {
  ClientCircuitArtifacts,
  type ClientProtocolArtifact,
  convertPrivateKernelInitInputsToWitnessMap,
  convertPrivateKernelInitOutputsFromWitnessMap,
  convertPrivateKernelInnerInputsToWitnessMap,
  convertPrivateKernelInnerOutputsFromWitnessMap,
  convertPrivateKernelTailForPublicOutputsFromWitnessMap,
  convertPrivateKernelTailInputsToWitnessMap,
  convertPrivateKernelTailOutputsFromWitnessMap,
  convertPrivateKernelTailToPublicInputsToWitnessMap,
} from '@aztec/noir-protocol-circuits-types';
import { type ACVMField, WASMSimulator } from '@aztec/simulator';
import { type NoirCompiledCircuit } from '@aztec/types/noir';

import { serializeWitness } from '@noir-lang/noirc_abi';
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
import {
  AGGREGATION_OBJECT_SIZE,
  CIRCUIT_PUBLIC_INPUTS_INDEX,
  CIRCUIT_RECURSIVE_INDEX,
  CIRCUIT_SIZE_INDEX,
  type VerificationKeyData,
} from './verification_key_data.js';

type PrivateKernelProvingOps = {
  convertOutputs: (outputs: WitnessMap) => PrivateKernelCircuitPublicInputs | PrivateKernelTailCircuitPublicInputs;
};

const PrivateKernelArtifactMapping: Record<ClientProtocolArtifact, PrivateKernelProvingOps> = {
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
    const witnessMap = convertPrivateKernelInitInputsToWitnessMap(inputs);
    return await this.createSafeProof(witnessMap, 'PrivateKernelInitArtifact');
  }

  public async createProofInner(
    inputs: PrivateKernelInnerCircuitPrivateInputs,
  ): Promise<KernelProofOutput<PrivateKernelCircuitPublicInputs>> {
    const witnessMap = convertPrivateKernelInnerInputsToWitnessMap(inputs);
    return await this.createSafeProof(witnessMap, 'PrivateKernelInnerArtifact');
  }

  public async createProofTail(
    inputs: PrivateKernelTailCircuitPrivateInputs,
  ): Promise<KernelProofOutput<PrivateKernelTailCircuitPublicInputs>> {
    if (!inputs.isForPublic()) {
      const witnessMap = convertPrivateKernelTailInputsToWitnessMap(inputs);
      return await this.createSafeProof(witnessMap, 'PrivateKernelTailArtifact');
    }
    const witnessMap = convertPrivateKernelTailToPublicInputsToWitnessMap(inputs);
    return await this.createSafeProof(witnessMap, 'PrivateKernelTailToPublicArtifact');
  }

  public async createAppCircuitProof(
    partialWitness: Map<number, ACVMField>,
    bytecode: Buffer,
  ): Promise<AppCircuitProofOutput> {
    const directory = `${this.bbWorkingDirectory}/${randomBytes(8).toString('hex')}`;
    await fs.mkdir(directory, { recursive: true });
    this.log.debug(`Created directory: ${directory}`);
    try {
      this.log.debug(`Proving app circuit`);
      const proofOutput = await this.createProof(directory, partialWitness, bytecode, 'App');
      if (proofOutput.proof.proof.length != RECURSIVE_PROOF_LENGTH) {
        throw new Error(`Incorrect proof length`);
      }
      const proof = proofOutput.proof as RecursiveProof<typeof RECURSIVE_PROOF_LENGTH>;
      const output: AppCircuitProofOutput = {
        proof,
        verificationKey: proofOutput.verificationKey,
      };
      return output;
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
  public async verifyProofForProtocolCircuit(circuitType: ClientProtocolArtifact, proof: Proof) {
    const verificationKey = await this.getVerificationKeyDataForCircuit(circuitType);

    this.log.debug(`Verifying with key: ${verificationKey.hash.toString()}`);

    const logFunction = (message: string) => {
      this.log.debug(`${circuitType} BB out - ${message}`);
    };

    const result = await this.verifyProofFromKey(verificationKey.keyAsBytes, proof, logFunction);

    if (result.status === BB_RESULT.FAILURE) {
      const errorMessage = `Failed to verify ${circuitType} proof!`;
      throw new Error(errorMessage);
    }

    this.log.info(`Successfully verified ${circuitType} proof in ${result.duration} ms`);
  }

  private async verifyProofFromKey(
    verificationKey: Buffer,
    proof: Proof,
    logFunction: (message: string) => void = () => {},
  ) {
    // Create random directory to be used for temp files
    const bbWorkingDirectory = `${this.bbWorkingDirectory}/${randomBytes(8).toString('hex')}`;
    await fs.mkdir(bbWorkingDirectory, { recursive: true });

    const proofFileName = `${bbWorkingDirectory}/proof`;
    const verificationKeyPath = `${bbWorkingDirectory}/vk`;

    await fs.writeFile(proofFileName, proof.buffer);
    await fs.writeFile(verificationKeyPath, verificationKey);

    try {
      return await verifyProof(this.bbBinaryPath, proofFileName, verificationKeyPath!, logFunction);
    } finally {
      await fs.rm(bbWorkingDirectory, { recursive: true, force: true });
    }
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
    return await promise;
  }

  private async createSafeProof<T>(
    inputs: WitnessMap,
    circuitType: ClientProtocolArtifact,
  ): Promise<KernelProofOutput<T>> {
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
  ): Promise<KernelProofOutput<T>> {
    this.log.debug(`Generating witness for ${circuitType}`);
    const compiledCircuit: NoirCompiledCircuit = ClientCircuitArtifacts[circuitType];

    const outputWitness = await this.simulator.simulateCircuit(inputs, compiledCircuit);

    this.log.debug(`Generated witness for ${circuitType}`);

    const publicInputs = PrivateKernelArtifactMapping[circuitType].convertOutputs(outputWitness) as T;

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

    const kernelOutput: KernelProofOutput<T> = {
      publicInputs,
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
  ): Promise<{
    proof: RecursiveProof<typeof RECURSIVE_PROOF_LENGTH> | RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>;
    verificationKey: VerificationKeyAsFields;
  }> {
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

    if (circuitType === 'App') {
      const vkData = await this.convertVk(directory);
      const proof = await this.readProofAsFields<typeof RECURSIVE_PROOF_LENGTH>(directory, circuitType, vkData);
      return { proof, verificationKey: new VerificationKeyAsFields(vkData.keyAsFields, vkData.hash) };
    }

    const vkData = await this.updateVerificationKeyAfterProof(directory, circuitType);
    const proof = await this.readProofAsFields<typeof NESTED_RECURSIVE_PROOF_LENGTH>(directory, circuitType, vkData);
    return { proof, verificationKey: new VerificationKeyAsFields(vkData.keyAsFields, vkData.hash) };
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
      circuitType === 'App' ? vkData.numPublicInputs : vkData.numPublicInputs - AGGREGATION_OBJECT_SIZE;
    const fieldsWithoutPublicInputs = fields.slice(numPublicInputs);
    this.log.debug(
      `Circuit type: ${circuitType}, complete proof length: ${fields.length}, without public inputs: ${fieldsWithoutPublicInputs.length}, num public inputs: ${numPublicInputs}, circuit size: ${vkData.circuitSize}, is recursive: ${vkData.isRecursive}, raw length: ${binaryProof.length}`,
    );
    const proof = new RecursiveProof<PROOF_LENGTH>(fieldsWithoutPublicInputs, new Proof(binaryProof));
    return proof;
  }
}
