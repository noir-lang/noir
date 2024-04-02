import {
  type AztecAddress,
  CallRequest,
  Fr,
  type MAX_NEW_NOTE_HASHES_PER_TX,
  type MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  NoteHashReadRequestMembershipWitness,
  PrivateCallData,
  PrivateKernelInitCircuitPrivateInputs,
  PrivateKernelInnerCircuitPrivateInputs,
  PrivateKernelInnerCircuitPublicInputs,
  PrivateKernelInnerData,
  PrivateKernelTailCircuitPrivateInputs,
  type SideEffect,
  type SideEffectLinkedToNoteHash,
  type TxRequest,
  VK_TREE_HEIGHT,
  VerificationKey,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { padArrayEnd } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { assertLength, mapTuple } from '@aztec/foundation/serialize';
import { pushTestData } from '@aztec/foundation/testing';
import { type ExecutionResult, type NoteAndSlot } from '@aztec/simulator';

import { HintsBuilder } from './hints_builder.js';
import { KernelProofCreator, type ProofCreator, type ProofOutput, type ProofOutputFinal } from './proof_creator.js';
import { type ProvingDataOracle } from './proving_data_oracle.js';

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
  private log = createDebugLogger('aztec:kernel-prover');
  private hintsBuilder: HintsBuilder;

  constructor(private oracle: ProvingDataOracle, private proofCreator: ProofCreator = new KernelProofCreator()) {
    this.hintsBuilder = new HintsBuilder(oracle);
  }

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
      publicInputs: PrivateKernelInnerCircuitPublicInputs.empty(),
      proof: makeEmptyProof(),
    };

    while (executionStack.length) {
      const currentExecution = executionStack.pop()!;
      executionStack.push(...currentExecution.nestedExecutions);

      const privateCallRequests = currentExecution.nestedExecutions.map(result =>
        result.callStackItem.toCallRequest(currentExecution.callStackItem.publicInputs.callContext),
      );
      const publicCallRequests = currentExecution.enqueuedPublicFunctionCalls.map(result => result.toCallRequest());

      // Start with the partially filled in read request witnesses from the simulator
      // and fill the non-transient ones in with sibling paths via oracle.
      const noteHashReadRequestMembershipWitnesses = currentExecution.noteHashReadRequestPartialWitnesses;
      for (let rr = 0; rr < noteHashReadRequestMembershipWitnesses.length; rr++) {
        // Pretty sure this check was forever broken. I made some changes to Fr and this started triggering.
        // The conditional makes no sense to me anyway.
        // if (currentExecution.callStackItem.publicInputs.readRequests[rr] == Fr.ZERO) {
        //   throw new Error(
        //     'Number of read requests output from Noir circuit does not match number of read request commitment indices output from simulator.',
        //   );
        // }
        const rrWitness = noteHashReadRequestMembershipWitnesses[rr];
        if (!rrWitness.isTransient) {
          // Non-transient reads must contain full membership witness with sibling path from commitment to root.
          // Get regular membership witness to fill in sibling path in the read request witness.
          const membershipWitness = await this.oracle.getNoteMembershipWitness(rrWitness.leafIndex.toBigInt());
          rrWitness.siblingPath = membershipWitness.siblingPath;
        }
      }

      // fill in witnesses for remaining/empty read requests
      noteHashReadRequestMembershipWitnesses.push(
        ...Array(MAX_NOTE_HASH_READ_REQUESTS_PER_CALL - noteHashReadRequestMembershipWitnesses.length)
          .fill(0)
          .map(() => NoteHashReadRequestMembershipWitness.empty(BigInt(0))),
      );

      const privateCallData = await this.createPrivateCallData(
        currentExecution,
        privateCallRequests,
        publicCallRequests,
        noteHashReadRequestMembershipWitnesses,
      );

      if (firstIteration) {
        const proofInput = new PrivateKernelInitCircuitPrivateInputs(txRequest, privateCallData);
        pushTestData('private-kernel-inputs-init', proofInput);
        output = await this.proofCreator.createProofInit(proofInput);
      } else {
        const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(previousVerificationKey);
        const previousKernelData = new PrivateKernelInnerData(
          output.publicInputs,
          output.proof,
          previousVerificationKey,
          Number(previousVkMembershipWitness.leafIndex),
          assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
        );
        const proofInput = new PrivateKernelInnerCircuitPrivateInputs(previousKernelData, privateCallData);
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
    const previousKernelData = new PrivateKernelInnerData(
      output.publicInputs,
      output.proof,
      previousVerificationKey,
      Number(previousVkMembershipWitness.leafIndex),
      assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
    );

    const [sortedNoteHashes, sortedNoteHashesIndexes] = this.hintsBuilder.sortSideEffects<
      SideEffect,
      typeof MAX_NEW_NOTE_HASHES_PER_TX
    >(output.publicInputs.end.newNoteHashes);

    const [sortedNullifiers, sortedNullifiersIndexes] = this.hintsBuilder.sortSideEffects<
      SideEffectLinkedToNoteHash,
      typeof MAX_NEW_NULLIFIERS_PER_TX
    >(output.publicInputs.end.newNullifiers);

    const readNoteHashHints = this.hintsBuilder.getNoteHashReadRequestHints(
      output.publicInputs.validationRequests.noteHashReadRequests,
      sortedNoteHashes,
    );

    const nullifierReadRequestHints = await this.hintsBuilder.getNullifierReadRequestHints(
      output.publicInputs.validationRequests.nullifierReadRequests,
      output.publicInputs.end.newNullifiers,
    );

    const nullifierNoteHashHints = this.hintsBuilder.getNullifierHints(
      mapTuple(sortedNullifiers, n => n.noteHash),
      sortedNoteHashes,
    );

    const masterNullifierSecretKeys = await this.hintsBuilder.getMasterNullifierSecretKeys(
      output.publicInputs.validationRequests.nullifierKeyValidationRequests,
    );

    this.log.debug(
      `Calling private kernel tail with hwm ${previousKernelData.publicInputs.minRevertibleSideEffectCounter}`,
    );

    const privateInputs = new PrivateKernelTailCircuitPrivateInputs(
      previousKernelData,
      sortedNoteHashes,
      sortedNoteHashesIndexes,
      readNoteHashHints,
      sortedNullifiers,
      sortedNullifiersIndexes,
      nullifierReadRequestHints,
      nullifierNoteHashHints,
      masterNullifierSecretKeys,
    );
    pushTestData('private-kernel-inputs-ordering', privateInputs);
    const outputFinal = await this.proofCreator.createProofTail(privateInputs);

    // Only return the notes whose commitment is in the commitments of the final proof.
    const finalNewCommitments = outputFinal.publicInputs.end.newNoteHashes;
    const outputNotes = finalNewCommitments.map(c => newNotes[c.value.toString()]).filter(c => !!c);

    return { ...outputFinal, outputNotes };
  }

  private async createPrivateCallData(
    { callStackItem, vk }: ExecutionResult,
    privateCallRequests: CallRequest[],
    publicCallRequests: CallRequest[],
    noteHashReadRequestMembershipWitnesses: NoteHashReadRequestMembershipWitness[],
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
    // const acirHash = keccak(Buffer.from(bytecode, 'hex'));
    const acirHash = Fr.fromBuffer(Buffer.alloc(32, 0));

    // TODO
    const proof = makeEmptyProof();

    return PrivateCallData.from({
      callStackItem,
      privateCallStack,
      publicCallStack,
      proof,
      vk: VerificationKey.fromBuffer(vk),
      publicKeysHash,
      contractClassArtifactHash,
      contractClassPublicBytecodeCommitment,
      saltedInitializationHash,
      functionLeafMembershipWitness,
      noteHashReadRequestMembershipWitnesses: makeTuple(
        MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
        i => noteHashReadRequestMembershipWitnesses[i],
        0,
      ),
      portalContractAddress: portalContractAddress.toField(),
      acirHash,
    });
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
    const newNoteHashes = await this.proofCreator.getSiloedCommitments(publicInputs);
    return newNotes.map((data, i) => ({
      contractAddress,
      data,
      commitment: newNoteHashes[i],
    }));
  }
}
