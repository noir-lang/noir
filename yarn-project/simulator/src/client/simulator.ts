import { type AztecNode, type FunctionCall, type Note, type TxExecutionRequest } from '@aztec/circuit-types';
import { CallContext } from '@aztec/circuits.js';
import {
  type ArrayType,
  type FunctionArtifact,
  FunctionSelector,
  FunctionType,
  type NoteSelector,
  encodeArguments,
} from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { type DebugLogger, createDebugLogger } from '@aztec/foundation/log';

import { createSimulationError } from '../common/errors.js';
import { PackedValuesCache } from '../common/packed_values_cache.js';
import { ClientExecutionContext } from './client_execution_context.js';
import { type DBOracle } from './db_oracle.js';
import { ExecutionNoteCache } from './execution_note_cache.js';
import { type ExecutionResult } from './execution_result.js';
import { executePrivateFunction } from './private_execution.js';
import { executeUnconstrainedFunction } from './unconstrained_execution.js';
import { ViewDataOracle } from './view_data_oracle.js';

/**
 * The ACIR simulator.
 */
export class AcirSimulator {
  private log: DebugLogger;

  constructor(private db: DBOracle, private node: AztecNode) {
    this.log = createDebugLogger('aztec:simulator');
  }

  /**
   * Runs a private function.
   * @param request - The transaction request.
   * @param entryPointArtifact - The artifact of the entry point function.
   * @param contractAddress - The address of the contract (should match request.origin)
   * @param msgSender - The address calling the function. This can be replaced to simulate a call from another contract or a specific account.
   * @returns The result of the execution.
   */
  public async run(
    request: TxExecutionRequest,
    entryPointArtifact: FunctionArtifact,
    contractAddress: AztecAddress,
    msgSender = AztecAddress.ZERO,
  ): Promise<ExecutionResult> {
    if (entryPointArtifact.functionType !== FunctionType.PRIVATE) {
      throw new Error(`Cannot run ${entryPointArtifact.functionType} function as private`);
    }

    if (request.origin !== contractAddress) {
      this.log.warn(
        `Request origin does not match contract address in simulation. Request origin: ${request.origin}, contract address: ${contractAddress}`,
      );
    }

    const header = await this.db.getHeader();

    // reserve the first side effect for the tx hash (inserted by the private kernel)
    const startSideEffectCounter = 1;

    const callContext = new CallContext(
      msgSender,
      contractAddress,
      FunctionSelector.fromNameAndParameters(entryPointArtifact.name, entryPointArtifact.parameters),
      false,
      entryPointArtifact.isStatic,
    );
    const context = new ClientExecutionContext(
      contractAddress,
      request.firstCallArgsHash,
      request.txContext,
      callContext,
      header,
      request.authWitnesses,
      PackedValuesCache.create(request.argsOfCalls),
      new ExecutionNoteCache(),
      this.db,
      this.node,
      startSideEffectCounter,
    );

    try {
      const executionResult = await executePrivateFunction(
        context,
        entryPointArtifact,
        contractAddress,
        request.functionSelector,
      );
      return executionResult;
    } catch (err) {
      throw createSimulationError(err instanceof Error ? err : new Error('Unknown error during private execution'));
    }
  }

  /**
   * Runs an unconstrained function.
   * @param request - The transaction request.
   * @param entryPointArtifact - The artifact of the entry point function.
   * @param contractAddress - The address of the contract.
   * @param aztecNode - The AztecNode instance.
   */
  public async runUnconstrained(
    request: FunctionCall,
    entryPointArtifact: FunctionArtifact,
    contractAddress: AztecAddress,
  ) {
    if (entryPointArtifact.functionType !== FunctionType.UNCONSTRAINED) {
      throw new Error(`Cannot run ${entryPointArtifact.functionType} function as unconstrained`);
    }

    const context = new ViewDataOracle(contractAddress, [], this.db, this.node);

    try {
      return await executeUnconstrainedFunction(
        context,
        entryPointArtifact,
        contractAddress,
        request.selector,
        request.args,
      );
    } catch (err) {
      throw createSimulationError(err instanceof Error ? err : new Error('Unknown error during private execution'));
    }
  }

  /**
   * Computes the inner nullifier of a note.
   * @param contractAddress - The address of the contract.
   * @param nonce - The nonce of the note hash.
   * @param storageSlot - The storage slot.
   * @param noteTypeId - The note type identifier.
   * @param computeNullifier - A flag indicating whether to compute the nullifier or just return 0.
   * @param note - The note.
   * @returns The nullifier.
   */
  public async computeNoteHashAndOptionallyANullifier(
    contractAddress: AztecAddress,
    nonce: Fr,
    storageSlot: Fr,
    noteTypeId: NoteSelector,
    computeNullifier: boolean,
    note: Note,
  ) {
    const artifact: FunctionArtifact | undefined = await this.db.getFunctionArtifactByName(
      contractAddress,
      'compute_note_hash_and_optionally_a_nullifier',
    );
    if (!artifact) {
      throw new Error(
        `Mandatory implementation of "compute_note_hash_and_optionally_a_nullifier" missing in noir contract ${contractAddress.toString()}.`,
      );
    }

    if (artifact.parameters.length != 6) {
      throw new Error(
        `Expected 6 parameters in mandatory implementation of "compute_note_hash_and_optionally_a_nullifier", but found ${
          artifact.parameters.length
        } in noir contract ${contractAddress.toString()}.`,
      );
    }

    const maxNoteFields = (artifact.parameters[artifact.parameters.length - 1].type as ArrayType).length;
    if (maxNoteFields < note.items.length) {
      throw new Error(
        `The note being processed has ${note.items.length} fields, while "compute_note_hash_and_optionally_a_nullifier" can only handle a maximum of ${maxNoteFields} fields. Please reduce the number of fields in your note.`,
      );
    }

    const extendedNoteItems = note.items.concat(Array(maxNoteFields - note.items.length).fill(Fr.ZERO));

    const execRequest: FunctionCall = {
      name: artifact.name,
      to: contractAddress,
      selector: FunctionSelector.empty(),
      type: FunctionType.UNCONSTRAINED,
      isStatic: artifact.isStatic,
      args: encodeArguments(artifact, [
        contractAddress,
        nonce,
        storageSlot,
        noteTypeId,
        computeNullifier,
        extendedNoteItems,
      ]),
      returnTypes: artifact.returnTypes,
    };

    const [innerNoteHash, uniqueNoteHash, siloedNoteHash, innerNullifier] = (await this.runUnconstrained(
      execRequest,
      artifact,
      contractAddress,
    )) as bigint[];

    return {
      innerNoteHash: new Fr(innerNoteHash),
      uniqueNoteHash: new Fr(uniqueNoteHash),
      siloedNoteHash: new Fr(siloedNoteHash),
      innerNullifier: new Fr(innerNullifier),
    };
  }

  /**
   * Computes the inner note hash of a note, which contains storage slot and the custom note hash.
   * @param contractAddress - The address of the contract.
   * @param storageSlot - The storage slot.
   * @param noteTypeId - The note type identifier.
   * @param note - The note.
   * @returns The note hash.
   */
  public async computeInnerNoteHash(
    contractAddress: AztecAddress,
    storageSlot: Fr,
    noteTypeId: NoteSelector,
    note: Note,
  ) {
    const { innerNoteHash } = await this.computeNoteHashAndOptionallyANullifier(
      contractAddress,
      Fr.ZERO,
      storageSlot,
      noteTypeId,
      false,
      note,
    );
    return innerNoteHash;
  }
}
