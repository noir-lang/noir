import { ExecutionResult, NewNoteData } from '@aztec/acir-simulator';
import {
  CONTRACT_TREE_HEIGHT,
  EcdsaSignature,
  MembershipWitness,
  PRIVATE_CALL_STACK_LENGTH,
  PreviousKernelData,
  PrivateCallData,
  PrivateCallStackItem,
  KernelCircuitPublicInputs,
  SignedTxRequest,
  TxRequest,
  VK_TREE_HEIGHT,
  VerificationKey,
  makeEmptyProof,
  AztecAddress,
  Fr,
} from '@aztec/circuits.js';
import { assertLength } from '@aztec/foundation/serialize';
import { ProofOutput, ProofCreator, KernelProofCreator } from './proof_creator.js';
import { ProvingDataOracle } from './proving_data_oracle.js';

/**
 * Represents an output note data object.
 * Contains the contract address, new note data and commitment for the note,
 * resulting from the execution of a transaction in the Aztec network.
 */
export interface OutputNoteData {
  /**
   * The address of the contract the note was created in.
   */
  contractAddress: AztecAddress;
  /**
   * The encrypted note data for an output note.
   */
  data: NewNoteData;
  /**
   * The unique value representing the note.
   */
  commitment: Fr;
}

/**
 * Represents the output data of the Kernel Prover.
 * Provides information about the newly created notes, along with the public inputs and proof.
 */
export interface KernelProverOutput extends ProofOutput {
  /**
   * An array of output notes containing the contract address, note data, and commitment for each new note.
   */
  outputNotes: OutputNoteData[];
}

/**
 * The KernelProver class is responsible for generating kernel proofs.
 * It takes a transaction request, its signature, and the simulation result as inputs, and outputs a proof
 * along with output notes. The class interacts with a ProvingDataOracle to fetch membership witnesses and
 * constructs private call data based on the execution results.
 */
export class KernelProver {
  constructor(private oracle: ProvingDataOracle, private proofCreator: ProofCreator = new KernelProofCreator()) {}

  /**
   * Generate a proof for a given transaction request, transaction signature, and execution result.
   * The function iterates through the nested executions in the execution result, creates private call data,
   * and generates a proof using the provided ProofCreator instance. It also maintains an index of new notes
   * created during the execution and returns them as a part of the KernelProverOutput.
   *
   * @param txRequest - The transaction request object.
   * @param txSignature - The ECDSA signature of the transaction.
   * @param executionResult - The execution result object containing nested executions and preimages.
   * @returns A Promise that resolves to a KernelProverOutput object containing proof, public inputs, and output notes.
   */
  async prove(
    txRequest: TxRequest,
    txSignature: EcdsaSignature,
    executionResult: ExecutionResult,
  ): Promise<KernelProverOutput> {
    const signedTxRequest = new SignedTxRequest(txRequest, txSignature);
    const executionStack = [executionResult];
    const newNotes: { [commitmentStr: string]: OutputNoteData } = {};
    let firstIteration = true;
    let previousVerificationKey = VerificationKey.makeFake();

    let output: ProofOutput = {
      publicInputs: KernelCircuitPublicInputs.empty(),
      proof: makeEmptyProof(),
    };
    while (executionStack.length) {
      const currentExecution = executionStack.pop()!;
      executionStack.push(...currentExecution.nestedExecutions);
      const privateCallStackPreimages = currentExecution.nestedExecutions.map(result => result.callStackItem);
      if (privateCallStackPreimages.length > PRIVATE_CALL_STACK_LENGTH) {
        throw new Error(
          `Too many items in the call stack. Maximum amount is ${PRIVATE_CALL_STACK_LENGTH}. Got ${privateCallStackPreimages.length}.`,
        );
      }
      privateCallStackPreimages.push(
        ...Array(PRIVATE_CALL_STACK_LENGTH - privateCallStackPreimages.length)
          .fill(0)
          .map(() => PrivateCallStackItem.empty()),
      );

      const privateCallData = await this.createPrivateCallData(currentExecution, privateCallStackPreimages);

      if (firstIteration) {
        output = await this.proofCreator.createProofInit(signedTxRequest, privateCallData);
      } else {
        const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(previousVerificationKey);
        const previousKernelData = new PreviousKernelData(
          output.publicInputs,
          output.proof,
          previousVerificationKey,
          Number(previousVkMembershipWitness.leafIndex),
          assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
        );
        output = await this.proofCreator.createProofInner(previousKernelData, privateCallData);
      }
      (await this.getNewNotes(currentExecution)).forEach(n => {
        newNotes[n.commitment.toString()] = n;
      });
      firstIteration = false;
      previousVerificationKey = privateCallData.vk;
    }

    // Only return the notes whose commitment is in the commitments of the final proof.
    const finalNewCommitments = output.publicInputs.end.newCommitments;
    const outputNotes = finalNewCommitments.map(c => newNotes[c.toString()]).filter(c => !!c);

    return { ...output, outputNotes };
  }

  private async createPrivateCallData(
    { callStackItem, vk }: ExecutionResult,
    privateCallStackPreimages: PrivateCallStackItem[],
  ) {
    const { contractAddress, functionData, publicInputs } = callStackItem;
    const { portalContractAddress } = publicInputs.callContext;

    const contractLeafMembershipWitness = functionData.isConstructor
      ? MembershipWitness.random(CONTRACT_TREE_HEIGHT)
      : await this.oracle.getContractMembershipWitness(contractAddress);

    const functionLeafMembershipWitness = await this.oracle.getFunctionMembershipWitness(
      contractAddress,
      functionData.functionSelectorBuffer,
    );

    // TODO
    // FIXME: https://github.com/AztecProtocol/aztec3-packages/issues/262
    // const acirHash = keccak(Buffer.from(bytecode, 'hex'));
    const acirHash = Fr.fromBuffer(Buffer.alloc(32, 0));

    // TODO
    const proof = makeEmptyProof();

    return new PrivateCallData(
      callStackItem,
      privateCallStackPreimages,
      proof,
      VerificationKey.fromBuffer(vk),
      functionLeafMembershipWitness,
      contractLeafMembershipWitness,
      portalContractAddress,
      acirHash,
    );
  }

  /**
   * Retrieves the new output notes for a given execution result.
   * The function maps over the new note preimages and associates them with their corresponding
   * commitments in the public inputs of the execution result. It also includes the contract address
   * from the call context of the public inputs.
   *
   * @param executionResult - The execution result object containing note preimages and public inputs.
   * @returns An array of OutputNoteData objects, each representing an output note with its associated data.
   */
  private async getNewNotes(executionResult: ExecutionResult): Promise<OutputNoteData[]> {
    const {
      callStackItem: { publicInputs },
      preimages,
    } = executionResult;
    const contractAddress = publicInputs.callContext.storageContractAddress;
    // Assuming that for each new commitment there's an output note added to the execution result.
    const newCommitments = await this.proofCreator.getSiloedCommitments(publicInputs);
    return preimages.newNotes.map((data, i) => ({
      contractAddress,
      data,
      commitment: newCommitments[i],
    }));
  }
}
