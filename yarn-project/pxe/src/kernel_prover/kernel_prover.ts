import { type PrivateKernelProver, type PrivateKernelSimulateOutput } from '@aztec/circuit-types';
import {
  Fr,
  MAX_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_NOTE_HASHES_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  MAX_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  PrivateCallData,
  PrivateKernelCircuitPublicInputs,
  PrivateKernelData,
  PrivateKernelInitCircuitPrivateInputs,
  PrivateKernelInnerCircuitPrivateInputs,
  PrivateKernelTailCircuitPrivateInputs,
  type PrivateKernelTailCircuitPublicInputs,
  type TxRequest,
  VK_TREE_HEIGHT,
  VerificationKeyAsFields,
  getNonEmptyItems,
} from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { assertLength } from '@aztec/foundation/serialize';
import { pushTestData } from '@aztec/foundation/testing';
import {
  ClientCircuitArtifacts,
  PrivateResetTagToArtifactName,
  getVKTreeRoot,
} from '@aztec/noir-protocol-circuits-types';
import { type ExecutionResult, collectNoteHashLeafIndexMap, collectNullifiedNoteHashCounters } from '@aztec/simulator';

import { type WitnessMap } from '@noir-lang/types';

import { buildPrivateKernelResetInputs } from './private_inputs_builders/index.js';
import { type ProvingDataOracle } from './proving_data_oracle.js';

const NULL_PROVE_OUTPUT: PrivateKernelSimulateOutput<PrivateKernelCircuitPublicInputs> = {
  publicInputs: PrivateKernelCircuitPublicInputs.empty(),
  verificationKey: VerificationKeyAsFields.makeEmpty(),
  outputWitness: new Map(),
};
/**
 * The KernelProver class is responsible for generating kernel proofs.
 * It takes a transaction request, its signature, and the simulation result as inputs, and outputs a proof
 * along with output notes. The class interacts with a ProvingDataOracle to fetch membership witnesses and
 * constructs private call data based on the execution results.
 */
export class KernelProver {
  private log = createDebugLogger('aztec:kernel-prover');

  constructor(private oracle: ProvingDataOracle, private proofCreator: PrivateKernelProver) {}

  /**
   * Generate a proof for a given transaction request and execution result.
   * The function iterates through the nested executions in the execution result, creates private call data,
   * and generates a proof using the provided ProofCreator instance. It also maintains an index of new notes
   * created during the execution and returns them as a part of the KernelProverOutput.
   *
   * @param txRequest - The authenticated transaction request object.
   * @param executionResult - The execution result object containing nested executions and preimages.
   * @returns A Promise that resolves to a KernelProverOutput object containing proof, public inputs, and output notes.
   * TODO(#7368) this should be refactored to not recreate the ACIR bytecode now that it operates on a program stack
   */
  async prove(
    txRequest: TxRequest,
    executionResult: ExecutionResult,
  ): Promise<PrivateKernelSimulateOutput<PrivateKernelTailCircuitPublicInputs>> {
    const executionStack = [executionResult];
    let firstIteration = true;

    let output = NULL_PROVE_OUTPUT;

    const noteHashLeafIndexMap = collectNoteHashLeafIndexMap(executionResult);
    const noteHashNullifierCounterMap = collectNullifiedNoteHashCounters(executionResult);
    // vector of gzipped bincode acirs
    const acirs: Buffer[] = [];
    const witnessStack: WitnessMap[] = [];

    while (executionStack.length) {
      if (!firstIteration && this.needsReset(executionStack, output)) {
        const resetInputs = await this.getPrivateKernelResetInputs(
          executionStack,
          output,
          noteHashLeafIndexMap,
          noteHashNullifierCounterMap,
        );
        output = await this.proofCreator.simulateProofReset(resetInputs);
        // TODO(#7368) consider refactoring this redundant bytecode pushing
        acirs.push(
          Buffer.from(ClientCircuitArtifacts[PrivateResetTagToArtifactName[resetInputs.sizeTag]].bytecode, 'base64'),
        );
        witnessStack.push(output.outputWitness);
      }
      const currentExecution = executionStack.pop()!;
      executionStack.push(...[...currentExecution.nestedExecutions].reverse());

      const functionName = await this.oracle.getDebugFunctionName(
        currentExecution.callStackItem.contractAddress,
        currentExecution.callStackItem.functionData.selector,
      );

      const appVk = await this.proofCreator.computeAppCircuitVerificationKey(currentExecution.acir, functionName);
      // TODO(#7368): This used to be associated with getDebugFunctionName
      // TODO(#7368): Is there any way to use this with client IVC proving?
      acirs.push(currentExecution.acir);
      witnessStack.push(currentExecution.partialWitness);

      const privateCallData = await this.createPrivateCallData(currentExecution, appVk.verificationKey);

      if (firstIteration) {
        const proofInput = new PrivateKernelInitCircuitPrivateInputs(txRequest, getVKTreeRoot(), privateCallData);
        pushTestData('private-kernel-inputs-init', proofInput);
        output = await this.proofCreator.simulateProofInit(proofInput);
        acirs.push(Buffer.from(ClientCircuitArtifacts.PrivateKernelInitArtifact.bytecode, 'base64'));
        witnessStack.push(output.outputWitness);
      } else {
        const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(output.verificationKey);
        const previousKernelData = new PrivateKernelData(
          output.publicInputs,
          output.verificationKey,
          Number(previousVkMembershipWitness.leafIndex),
          assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
        );
        const proofInput = new PrivateKernelInnerCircuitPrivateInputs(previousKernelData, privateCallData);
        pushTestData('private-kernel-inputs-inner', proofInput);
        output = await this.proofCreator.simulateProofInner(proofInput);
        acirs.push(Buffer.from(ClientCircuitArtifacts.PrivateKernelInnerArtifact.bytecode, 'base64'));
        witnessStack.push(output.outputWitness);
      }
      firstIteration = false;
    }

    if (this.somethingToReset(output)) {
      const resetInputs = await this.getPrivateKernelResetInputs(
        executionStack,
        output,
        noteHashLeafIndexMap,
        noteHashNullifierCounterMap,
      );
      output = await this.proofCreator.simulateProofReset(resetInputs);
      // TODO(#7368) consider refactoring this redundant bytecode pushing
      acirs.push(
        Buffer.from(ClientCircuitArtifacts[PrivateResetTagToArtifactName[resetInputs.sizeTag]].bytecode, 'base64'),
      );
      witnessStack.push(output.outputWitness);
    }
    const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(output.verificationKey);
    const previousKernelData = new PrivateKernelData(
      output.publicInputs,
      output.verificationKey,
      Number(previousVkMembershipWitness.leafIndex),
      assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
    );

    this.log.debug(
      `Calling private kernel tail with hwm ${previousKernelData.publicInputs.minRevertibleSideEffectCounter}`,
    );

    const privateInputs = new PrivateKernelTailCircuitPrivateInputs(previousKernelData);

    pushTestData('private-kernel-inputs-ordering', privateInputs);
    const tailOutput = await this.proofCreator.simulateProofTail(privateInputs);
    acirs.push(
      Buffer.from(
        privateInputs.isForPublic()
          ? ClientCircuitArtifacts.PrivateKernelTailToPublicArtifact.bytecode
          : ClientCircuitArtifacts.PrivateKernelTailArtifact.bytecode,
        'base64',
      ),
    );
    witnessStack.push(tailOutput.outputWitness);

    // TODO(#7368) how do we 'bincode' encode these inputs?
    const ivcProof = await this.proofCreator.createClientIvcProof(acirs, witnessStack);
    tailOutput.clientIvcProof = ivcProof;
    return tailOutput;
  }

  private needsReset(
    executionStack: ExecutionResult[],
    output: PrivateKernelSimulateOutput<PrivateKernelCircuitPublicInputs>,
  ) {
    const nextIteration = executionStack[executionStack.length - 1];
    return (
      getNonEmptyItems(nextIteration.callStackItem.publicInputs.noteHashes).length +
        getNonEmptyItems(output.publicInputs.end.noteHashes).length >
        MAX_NOTE_HASHES_PER_TX ||
      getNonEmptyItems(nextIteration.callStackItem.publicInputs.nullifiers).length +
        getNonEmptyItems(output.publicInputs.end.nullifiers).length >
        MAX_NULLIFIERS_PER_TX ||
      getNonEmptyItems(nextIteration.callStackItem.publicInputs.noteEncryptedLogsHashes).length +
        getNonEmptyItems(output.publicInputs.end.noteEncryptedLogsHashes).length >
        MAX_NOTE_ENCRYPTED_LOGS_PER_TX ||
      getNonEmptyItems(nextIteration.callStackItem.publicInputs.noteHashReadRequests).length +
        getNonEmptyItems(output.publicInputs.validationRequests.noteHashReadRequests).length >
        MAX_NOTE_HASH_READ_REQUESTS_PER_TX ||
      getNonEmptyItems(nextIteration.callStackItem.publicInputs.nullifierReadRequests).length +
        getNonEmptyItems(output.publicInputs.validationRequests.nullifierReadRequests).length >
        MAX_NULLIFIER_READ_REQUESTS_PER_TX ||
      getNonEmptyItems(nextIteration.callStackItem.publicInputs.keyValidationRequestsAndGenerators).length +
        getNonEmptyItems(output.publicInputs.validationRequests.scopedKeyValidationRequestsAndGenerators).length >
        MAX_KEY_VALIDATION_REQUESTS_PER_TX
    );
  }

  private somethingToReset(output: PrivateKernelSimulateOutput<PrivateKernelCircuitPublicInputs>) {
    return (
      getNonEmptyItems(output.publicInputs.validationRequests.noteHashReadRequests).length > 0 ||
      getNonEmptyItems(output.publicInputs.validationRequests.nullifierReadRequests).length > 0 ||
      getNonEmptyItems(output.publicInputs.validationRequests.scopedKeyValidationRequestsAndGenerators).length > 0 ||
      output.publicInputs.end.nullifiers.find(nullifier => !nullifier.nullifiedNoteHash.equals(Fr.zero()))
    );
  }

  private async getPrivateKernelResetInputs(
    executionStack: ExecutionResult[],
    output: PrivateKernelSimulateOutput<PrivateKernelCircuitPublicInputs>,
    noteHashLeafIndexMap: Map<bigint, bigint>,
    noteHashNullifierCounterMap: Map<number, number>,
  ) {
    const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(output.verificationKey);
    const previousKernelData = new PrivateKernelData(
      output.publicInputs,
      output.verificationKey,
      Number(previousVkMembershipWitness.leafIndex),
      assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
    );

    return await buildPrivateKernelResetInputs(
      executionStack,
      previousKernelData,
      noteHashLeafIndexMap,
      noteHashNullifierCounterMap,
      this.oracle,
    );
  }

  private async createPrivateCallData({ callStackItem }: ExecutionResult, vk: VerificationKeyAsFields) {
    const { contractAddress, functionData } = callStackItem;

    const functionLeafMembershipWitness = await this.oracle.getFunctionMembershipWitness(
      contractAddress,
      functionData.selector,
    );
    const { contractClassId, publicKeysHash, saltedInitializationHash } = await this.oracle.getContractAddressPreimage(
      contractAddress,
    );
    const { artifactHash: contractClassArtifactHash, publicBytecodeCommitment: contractClassPublicBytecodeCommitment } =
      await this.oracle.getContractClassIdPreimage(contractClassId);

    // TODO(#262): Use real acir hash
    // const acirHash = keccak256(Buffer.from(bytecode, 'hex'));
    const acirHash = Fr.fromBuffer(Buffer.alloc(32, 0));

    return PrivateCallData.from({
      callStackItem,
      vk,
      publicKeysHash,
      contractClassArtifactHash,
      contractClassPublicBytecodeCommitment,
      saltedInitializationHash,
      functionLeafMembershipWitness,
      acirHash,
    });
  }
}
