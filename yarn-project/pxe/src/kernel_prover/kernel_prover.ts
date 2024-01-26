import { ExecutionResult, NoteAndSlot } from '@aztec/acir-simulator';
import {
  AztecAddress,
  CONTRACT_TREE_HEIGHT,
  CallRequest,
  Fr,
  GrumpkinScalar,
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_READ_REQUESTS_PER_CALL,
  MAX_READ_REQUESTS_PER_TX,
  MembershipWitness,
  NullifierKeyValidationRequestContext,
  PreviousKernelData,
  PrivateCallData,
  PrivateKernelInputsInit,
  PrivateKernelInputsInner,
  PrivateKernelInputsOrdering,
  PrivateKernelPublicInputs,
  ReadRequestMembershipWitness,
  SideEffect,
  SideEffectLinkedToNoteHash,
  SideEffectType,
  TxRequest,
  VK_TREE_HEIGHT,
  VerificationKey,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Tuple, assertLength, mapTuple } from '@aztec/foundation/serialize';
import { pushTestData } from '@aztec/foundation/testing';

import { KernelProofCreator, ProofCreator, ProofOutput, ProofOutputFinal } from './proof_creator.js';
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
  data: NoteAndSlot;
  /**
   * The unique value representing the note.
   */
  commitment: Fr;
}

/**
 * Represents the output data of the Kernel Prover.
 * Provides information about the newly created notes, along with the public inputs and proof.
 */
export interface KernelProverOutput extends ProofOutputFinal {
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
   * Generate a proof for a given transaction request and execution result.
   * The function iterates through the nested executions in the execution result, creates private call data,
   * and generates a proof using the provided ProofCreator instance. It also maintains an index of new notes
   * created during the execution and returns them as a part of the KernelProverOutput.
   *
   * @param txRequest - The authenticated transaction request object.
   * @param executionResult - The execution result object containing nested executions and preimages.
   * @returns A Promise that resolves to a KernelProverOutput object containing proof, public inputs, and output notes.
   */
  async prove(txRequest: TxRequest, executionResult: ExecutionResult): Promise<KernelProverOutput> {
    const executionStack = [executionResult];
    const newNotes: { [commitmentStr: string]: OutputNoteData } = {};
    let firstIteration = true;
    let previousVerificationKey = VerificationKey.makeFake();

    let output: ProofOutput = {
      publicInputs: PrivateKernelPublicInputs.empty(),
      proof: makeEmptyProof(),
    };

    while (executionStack.length) {
      const currentExecution = executionStack.pop()!;
      executionStack.push(...currentExecution.nestedExecutions);

      const privateCallRequests = currentExecution.nestedExecutions.map(result => result.callStackItem.toCallRequest());
      const publicCallRequests = currentExecution.enqueuedPublicFunctionCalls.map(result => result.toCallRequest());

      // Start with the partially filled in read request witnesses from the simulator
      // and fill the non-transient ones in with sibling paths via oracle.
      const readRequestMembershipWitnesses = currentExecution.readRequestPartialWitnesses;
      for (let rr = 0; rr < readRequestMembershipWitnesses.length; rr++) {
        // Pretty sure this check was forever broken. I made some changes to Fr and this started triggering.
        // The conditional makes no sense to me anyway.
        // if (currentExecution.callStackItem.publicInputs.readRequests[rr] == Fr.ZERO) {
        //   throw new Error(
        //     'Number of read requests output from Noir circuit does not match number of read request commitment indices output from simulator.',
        //   );
        // }
        const rrWitness = readRequestMembershipWitnesses[rr];
        if (!rrWitness.isTransient) {
          // Non-transient reads must contain full membership witness with sibling path from commitment to root.
          // Get regular membership witness to fill in sibling path in the read request witness.
          const membershipWitness = await this.oracle.getNoteMembershipWitness(rrWitness.leafIndex.toBigInt());
          rrWitness.siblingPath = membershipWitness.siblingPath;
        }
      }

      // fill in witnesses for remaining/empty read requests
      readRequestMembershipWitnesses.push(
        ...Array(MAX_READ_REQUESTS_PER_CALL - readRequestMembershipWitnesses.length)
          .fill(0)
          .map(() => ReadRequestMembershipWitness.empty(BigInt(0))),
      );

      const privateCallData = await this.createPrivateCallData(
        currentExecution,
        privateCallRequests,
        publicCallRequests,
        readRequestMembershipWitnesses,
      );

      if (firstIteration) {
        const proofInput = new PrivateKernelInputsInit(txRequest, privateCallData);
        pushTestData('private-kernel-inputs-init', proofInput);
        output = await this.proofCreator.createProofInit(proofInput);
      } else {
        const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(previousVerificationKey);
        const previousKernelData = new PreviousKernelData(
          output.publicInputs,
          output.proof,
          previousVerificationKey,
          Number(previousVkMembershipWitness.leafIndex),
          assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
        );
        const proofInput = new PrivateKernelInputsInner(previousKernelData, privateCallData);
        pushTestData('private-kernel-inputs-inner', proofInput);
        output = await this.proofCreator.createProofInner(proofInput);
      }
      (await this.getNewNotes(currentExecution)).forEach(n => {
        newNotes[n.commitment.toString()] = n;
      });
      firstIteration = false;
      previousVerificationKey = privateCallData.vk;
    }

    const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(previousVerificationKey);
    const previousKernelData = new PreviousKernelData(
      output.publicInputs,
      output.proof,
      previousVerificationKey,
      Number(previousVkMembershipWitness.leafIndex),
      assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
    );

    const [sortedCommitments, sortedCommitmentsIndexes] = this.sortSideEffects<
      SideEffect,
      typeof MAX_NEW_COMMITMENTS_PER_TX
    >(output.publicInputs.end.newCommitments);

    const [sortedNullifiers, sortedNullifiersIndexes] = this.sortSideEffects<
      SideEffectLinkedToNoteHash,
      typeof MAX_NEW_NULLIFIERS_PER_TX
    >(output.publicInputs.end.newNullifiers);

    const readCommitmentHints = this.getReadRequestHints(output.publicInputs.end.readRequests, sortedCommitments);

    const nullifierCommitmentHints = this.getNullifierHints(
      mapTuple(sortedNullifiers, n => n.noteHash),
      sortedCommitments,
    );

    const masterNullifierSecretKeys = await this.getMasterNullifierSecretKeys(
      output.publicInputs.end.nullifierKeyValidationRequests,
    );

    const privateInputs = new PrivateKernelInputsOrdering(
      previousKernelData,
      sortedCommitments,
      sortedCommitmentsIndexes,
      readCommitmentHints,
      sortedNullifiers,
      sortedNullifiersIndexes,
      nullifierCommitmentHints,
      masterNullifierSecretKeys,
    );
    pushTestData('private-kernel-inputs-ordering', privateInputs);
    const outputFinal = await this.proofCreator.createProofOrdering(privateInputs);

    // Only return the notes whose commitment is in the commitments of the final proof.
    const finalNewCommitments = outputFinal.publicInputs.end.newCommitments;
    const outputNotes = finalNewCommitments.map(c => newNotes[c.value.toString()]).filter(c => !!c);

    return { ...outputFinal, outputNotes };
  }

  private sortSideEffects<T extends SideEffectType, K extends number>(
    sideEffects: Tuple<T, K>,
  ): [Tuple<T, K>, Tuple<number, K>] {
    const sorted = sideEffects
      .map((sideEffect, index) => ({ sideEffect, index }))
      .sort((a, b) => {
        // Empty ones go to the right
        if (a.sideEffect.isEmpty()) {
          return 1;
        }
        return Number(a.sideEffect.counter.toBigInt() - b.sideEffect.counter.toBigInt());
      });

    const originalToSorted = sorted.map(() => 0);
    sorted.forEach(({ index }, i) => {
      originalToSorted[index] = i;
    });

    return [sorted.map(({ sideEffect }) => sideEffect) as Tuple<T, K>, originalToSorted as Tuple<number, K>];
  }

  private async createPrivateCallData(
    { callStackItem, vk }: ExecutionResult,
    privateCallRequests: CallRequest[],
    publicCallRequests: CallRequest[],
    readRequestMembershipWitnesses: ReadRequestMembershipWitness[],
  ) {
    const { contractAddress, functionData, publicInputs } = callStackItem;
    const { portalContractAddress } = publicInputs.callContext;

    // Pad with empty items to reach max/const length expected by circuit.
    const privateCallStack = padArrayEnd(
      privateCallRequests,
      CallRequest.empty(),
      MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
    );
    const publicCallStack = padArrayEnd(publicCallRequests, CallRequest.empty(), MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL);

    const contractLeafMembershipWitness = functionData.isConstructor
      ? MembershipWitness.random(CONTRACT_TREE_HEIGHT)
      : await this.oracle.getContractMembershipWitness(contractAddress);

    const functionLeafMembershipWitness = await this.oracle.getFunctionMembershipWitness(
      contractAddress,
      functionData.selector,
    );

    // TODO(#262): Use real acir hash
    // const acirHash = keccak(Buffer.from(bytecode, 'hex'));
    const acirHash = Fr.fromBuffer(Buffer.alloc(32, 0));

    // TODO
    const proof = makeEmptyProof();

    return new PrivateCallData(
      callStackItem,
      privateCallStack,
      publicCallStack,
      proof,
      VerificationKey.fromBuffer(vk),
      functionLeafMembershipWitness,
      contractLeafMembershipWitness,
      makeTuple(MAX_READ_REQUESTS_PER_CALL, i => readRequestMembershipWitnesses[i], 0),
      portalContractAddress.toField(),
      acirHash,
    );
  }

  /**
   * Retrieves the new output notes for a given execution result.
   * The function maps over the new notes and associates them with their corresponding
   * commitments in the public inputs of the execution result. It also includes the contract address
   * from the call context of the public inputs.
   *
   * @param executionResult - The execution result object containing notes and public inputs.
   * @returns An array of OutputNoteData objects, each representing an output note with its associated data.
   */
  private async getNewNotes(executionResult: ExecutionResult): Promise<OutputNoteData[]> {
    const {
      callStackItem: { publicInputs },
      newNotes,
    } = executionResult;
    const contractAddress = publicInputs.callContext.storageContractAddress;
    // Assuming that for each new commitment there's an output note added to the execution result.
    const newCommitments = await this.proofCreator.getSiloedCommitments(publicInputs);
    return newNotes.map((data, i) => ({
      contractAddress,
      data,
      commitment: newCommitments[i],
    }));
  }

  /**
   * Performs the matching between an array of read request and an array of commitments. This produces
   * hints for the private kernel ordering circuit to efficiently match a read request with the corresponding
   * commitment.
   *
   * @param readRequests - The array of read requests.
   * @param commitments - The array of commitments.
   * @returns An array of hints where each element is the index of the commitment in commitments array
   *  corresponding to the read request. In other words we have readRequests[i] == commitments[hints[i]].
   */
  private getReadRequestHints(
    readRequests: Tuple<SideEffect, typeof MAX_READ_REQUESTS_PER_TX>,
    commitments: Tuple<SideEffect, typeof MAX_NEW_COMMITMENTS_PER_TX>,
  ): Tuple<Fr, typeof MAX_READ_REQUESTS_PER_TX> {
    const hints = makeTuple(MAX_READ_REQUESTS_PER_TX, Fr.zero);
    for (let i = 0; i < MAX_READ_REQUESTS_PER_TX && !readRequests[i].isEmpty(); i++) {
      const equalToRR = (cmt: SideEffect) => cmt.value.equals(readRequests[i].value);
      const result = commitments.findIndex(equalToRR);
      if (result == -1) {
        throw new Error(
          `The read request at index ${i} with value ${readRequests[i].toString()} does not match to any commitment.`,
        );
      } else {
        hints[i] = new Fr(result);
      }
    }
    return hints;
  }

  /**
   *  Performs the matching between an array of nullified commitments and an array of commitments. This produces
   * hints for the private kernel ordering circuit to efficiently match a nullifier with the corresponding
   * commitment.
   *
   * @param nullifiedCommitments - The array of nullified commitments.
   * @param commitments - The array of commitments.
   * @returns An array of hints where each element is the index of the commitment in commitments array
   *  corresponding to the nullified commitments. In other words we have nullifiedCommitments[i] == commitments[hints[i]].
   */
  private getNullifierHints(
    nullifiedCommitments: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    commitments: Tuple<SideEffect, typeof MAX_NEW_COMMITMENTS_PER_TX>,
  ): Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX> {
    const hints = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, Fr.zero);
    for (let i = 0; i < MAX_NEW_NULLIFIERS_PER_TX; i++) {
      if (!nullifiedCommitments[i].isZero()) {
        const equalToCommitment = (cmt: SideEffect) => cmt.value.equals(nullifiedCommitments[i]);
        const result = commitments.findIndex(equalToCommitment);
        if (result == -1) {
          throw new Error(
            `The nullified commitment at index ${i} with value ${nullifiedCommitments[
              i
            ].toString()} does not match to any commitment.`,
          );
        } else {
          hints[i] = new Fr(result);
        }
      }
    }
    return hints;
  }

  private async getMasterNullifierSecretKeys(
    nullifierKeyValidationRequests: Tuple<
      NullifierKeyValidationRequestContext,
      typeof MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX
    >,
  ) {
    const keys = makeTuple(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, GrumpkinScalar.zero);
    for (let i = 0; i < nullifierKeyValidationRequests.length; ++i) {
      const request = nullifierKeyValidationRequests[i];
      if (request.isEmpty()) {
        break;
      }
      keys[i] = await this.oracle.getMasterNullifierSecretKey(request.publicKey);
    }
    return keys;
  }
}
