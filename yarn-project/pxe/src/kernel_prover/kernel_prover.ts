import { type KernelProofOutput, type ProofCreator } from '@aztec/circuit-types';
import {
  CallRequest,
  Fr,
  MAX_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  NESTED_RECURSIVE_PROOF_LENGTH,
  PrivateCallData,
  PrivateKernelCircuitPublicInputs,
  PrivateKernelData,
  PrivateKernelInitCircuitPrivateInputs,
  PrivateKernelInnerCircuitPrivateInputs,
  PrivateKernelTailCircuitPrivateInputs,
  type PrivateKernelTailCircuitPublicInputs,
  type RECURSIVE_PROOF_LENGTH,
  type RecursiveProof,
  type TxRequest,
  VK_TREE_HEIGHT,
  VerificationKeyAsFields,
  getNonEmptyItems,
  makeRecursiveProof,
} from '@aztec/circuits.js';
import { padArrayEnd } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { assertLength } from '@aztec/foundation/serialize';
import { pushTestData } from '@aztec/foundation/testing';
import { type ExecutionResult, collectNoteHashLeafIndexMap, collectNullifiedNoteHashCounters } from '@aztec/simulator';

import {
  buildPrivateKernelInitHints,
  buildPrivateKernelInnerHints,
  buildPrivateKernelResetInputs,
} from './private_inputs_builders/index.js';
import { type ProvingDataOracle } from './proving_data_oracle.js';

/**
 * The KernelProver class is responsible for generating kernel proofs.
 * It takes a transaction request, its signature, and the simulation result as inputs, and outputs a proof
 * along with output notes. The class interacts with a ProvingDataOracle to fetch membership witnesses and
 * constructs private call data based on the execution results.
 */
export class KernelProver {
  private log = createDebugLogger('aztec:kernel-prover');

  constructor(private oracle: ProvingDataOracle, private proofCreator: ProofCreator) {}

  /**
   * Generate a proof for a given transaction request and execution result.
   * The function iterates through the nested executions in the execution result, creates private call data,
   * and generates a proof using the provided ProofCreator instance. It also maintains an index of new notes
   * created during the execution and returns them as a part of the KernelProverOutput.
   *
   * @param txRequest - The authenticated transaction request object.
   * @param executionResult - The execution result object containing nested executions and preimages.
   * @returns A Promise that resolves to a KernelProverOutput object containing proof, public inputs, and output notes.
   */
  async prove(
    txRequest: TxRequest,
    executionResult: ExecutionResult,
  ): Promise<KernelProofOutput<PrivateKernelTailCircuitPublicInputs>> {
    const executionStack = [executionResult];
    let firstIteration = true;

    let output: KernelProofOutput<PrivateKernelCircuitPublicInputs> = {
      publicInputs: PrivateKernelCircuitPublicInputs.empty(),
      proof: makeRecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>(NESTED_RECURSIVE_PROOF_LENGTH),
      verificationKey: VerificationKeyAsFields.makeEmpty(),
    };

    const noteHashLeafIndexMap = collectNoteHashLeafIndexMap(executionResult);
    const noteHashNullifierCounterMap = collectNullifiedNoteHashCounters(executionResult);

    while (executionStack.length) {
      if (!firstIteration && this.needsReset(executionStack, output)) {
        output = await this.runReset(executionStack, output, noteHashLeafIndexMap, noteHashNullifierCounterMap);
      }
      const currentExecution = executionStack.pop()!;
      executionStack.push(...[...currentExecution.nestedExecutions].reverse());

      const publicCallRequests = currentExecution.enqueuedPublicFunctionCalls.map(result => result.toCallRequest());
      const publicTeardownCallRequest = currentExecution.publicTeardownFunctionCall.isEmpty()
        ? CallRequest.empty()
        : currentExecution.publicTeardownFunctionCall.toCallRequest();

      const functionName = await this.oracle.getDebugFunctionName(
        currentExecution.callStackItem.contractAddress,
        currentExecution.callStackItem.functionData.selector,
      );

      const proofOutput = await this.proofCreator.createAppCircuitProof(
        currentExecution.partialWitness,
        currentExecution.acir,
        functionName,
      );

      const privateCallData = await this.createPrivateCallData(
        currentExecution,
        publicCallRequests,
        publicTeardownCallRequest,
        proofOutput.proof,
        proofOutput.verificationKey,
      );

      if (firstIteration) {
        const hints = buildPrivateKernelInitHints(
          currentExecution.callStackItem.publicInputs,
          noteHashNullifierCounterMap,
        );
        const proofInput = new PrivateKernelInitCircuitPrivateInputs(txRequest, privateCallData, hints);
        pushTestData('private-kernel-inputs-init', proofInput);
        output = await this.proofCreator.createProofInit(proofInput);
      } else {
        const hints = buildPrivateKernelInnerHints(
          currentExecution.callStackItem.publicInputs,
          noteHashNullifierCounterMap,
        );
        const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(output.verificationKey);
        const previousKernelData = new PrivateKernelData(
          output.publicInputs,
          output.proof,
          output.verificationKey,
          Number(previousVkMembershipWitness.leafIndex),
          assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
        );
        const proofInput = new PrivateKernelInnerCircuitPrivateInputs(previousKernelData, privateCallData, hints);
        pushTestData('private-kernel-inputs-inner', proofInput);
        output = await this.proofCreator.createProofInner(proofInput);
      }
      firstIteration = false;
    }

    if (this.somethingToReset(output)) {
      output = await this.runReset(executionStack, output, noteHashLeafIndexMap, noteHashNullifierCounterMap);
    }
    const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(output.verificationKey);
    const previousKernelData = new PrivateKernelData(
      output.publicInputs,
      output.proof,
      output.verificationKey,
      Number(previousVkMembershipWitness.leafIndex),
      assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
    );

    this.log.debug(
      `Calling private kernel tail with hwm ${previousKernelData.publicInputs.minRevertibleSideEffectCounter}`,
    );

    const privateInputs = new PrivateKernelTailCircuitPrivateInputs(previousKernelData);

    pushTestData('private-kernel-inputs-ordering', privateInputs);
    return await this.proofCreator.createProofTail(privateInputs);
  }

  private needsReset(executionStack: ExecutionResult[], output: KernelProofOutput<PrivateKernelCircuitPublicInputs>) {
    const nextIteration = executionStack[executionStack.length - 1];
    return (
      getNonEmptyItems(nextIteration.callStackItem.publicInputs.newNoteHashes).length +
        getNonEmptyItems(output.publicInputs.end.newNoteHashes).length >
        MAX_NEW_NOTE_HASHES_PER_TX ||
      getNonEmptyItems(nextIteration.callStackItem.publicInputs.newNullifiers).length +
        getNonEmptyItems(output.publicInputs.end.newNullifiers).length >
        MAX_NEW_NULLIFIERS_PER_TX ||
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

  private somethingToReset(output: KernelProofOutput<PrivateKernelCircuitPublicInputs>) {
    return (
      getNonEmptyItems(output.publicInputs.validationRequests.noteHashReadRequests).length > 0 ||
      getNonEmptyItems(output.publicInputs.validationRequests.nullifierReadRequests).length > 0 ||
      getNonEmptyItems(output.publicInputs.validationRequests.scopedKeyValidationRequestsAndGenerators).length > 0 ||
      output.publicInputs.end.newNoteHashes.find(noteHash => noteHash.nullifierCounter !== 0) ||
      output.publicInputs.end.newNullifiers.find(nullifier => !nullifier.nullifiedNoteHash.equals(Fr.zero()))
    );
  }

  private async runReset(
    executionStack: ExecutionResult[],
    output: KernelProofOutput<PrivateKernelCircuitPublicInputs>,
    noteHashLeafIndexMap: Map<bigint, bigint>,
    noteHashNullifierCounterMap: Map<number, number>,
  ): Promise<KernelProofOutput<PrivateKernelCircuitPublicInputs>> {
    const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(output.verificationKey);
    const previousKernelData = new PrivateKernelData(
      output.publicInputs,
      output.proof,
      output.verificationKey,
      Number(previousVkMembershipWitness.leafIndex),
      assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
    );

    return this.proofCreator.createProofReset(
      await buildPrivateKernelResetInputs(
        executionStack,
        previousKernelData,
        noteHashLeafIndexMap,
        noteHashNullifierCounterMap,
        this.oracle,
      ),
    );
  }

  private async createPrivateCallData(
    { callStackItem }: ExecutionResult,
    publicCallRequests: CallRequest[],
    publicTeardownCallRequest: CallRequest,
    proof: RecursiveProof<typeof RECURSIVE_PROOF_LENGTH>,
    vk: VerificationKeyAsFields,
  ) {
    const { contractAddress, functionData } = callStackItem;

    const publicCallStack = padArrayEnd(publicCallRequests, CallRequest.empty(), MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL);

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
      publicCallStack,
      publicTeardownCallRequest,
      proof,
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
