import { AztecNode, FunctionCall, Note, TxExecutionRequest } from '@aztec/circuit-types';
import { CallContext, FunctionData } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import {
  ArrayType,
  FunctionArtifactWithDebugMetadata,
  FunctionSelector,
  FunctionType,
  encodeArguments,
} from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';

import { WasmBlackBoxFunctionSolver, createBlackBoxSolver } from '@noir-lang/acvm_js';

import { createSimulationError } from '../common/errors.js';
import { PackedArgsCache } from '../common/packed_args_cache.js';
import { ClientExecutionContext } from './client_execution_context.js';
import { DBOracle } from './db_oracle.js';
import { ExecutionNoteCache } from './execution_note_cache.js';
import { ExecutionResult } from './execution_result.js';
import { executePrivateFunction } from './private_execution.js';
import { executeUnconstrainedFunction } from './unconstrained_execution.js';
import { ViewDataOracle } from './view_data_oracle.js';

/**
 * The ACIR simulator.
 */
export class AcirSimulator {
  private static solver: Promise<WasmBlackBoxFunctionSolver>; // ACVM's backend
  private log: DebugLogger;

  constructor(private db: DBOracle) {
    this.log = createDebugLogger('aztec:simulator');
  }

  /**
   * Gets or initializes the ACVM WasmBlackBoxFunctionSolver.
   *
   * @remarks
   *
   * Occurs only once across all instances of AcirSimulator.
   * Speeds up execution by only performing setup tasks (like pedersen
   * generator initialization) one time.
   * TODO(https://github.com/AztecProtocol/aztec-packages/issues/1627):
   * determine whether this requires a lock
   *
   * @returns ACVM WasmBlackBoxFunctionSolver
   */
  public static getSolver(): Promise<WasmBlackBoxFunctionSolver> {
    if (!this.solver) {
      this.solver = createBlackBoxSolver();
    }
    return this.solver;
  }

  /**
   * Runs a private function.
   * @param request - The transaction request.
   * @param entryPointArtifact - The artifact of the entry point function.
   * @param contractAddress - The address of the contract (should match request.origin)
   * @param portalContractAddress - The address of the portal contract.
   * @param msgSender - The address calling the function. This can be replaced to simulate a call from another contract or a specific account.
   * @returns The result of the execution.
   */
  public async run(
    request: TxExecutionRequest,
    entryPointArtifact: FunctionArtifactWithDebugMetadata,
    contractAddress: AztecAddress,
    portalContractAddress: EthAddress,
    msgSender = AztecAddress.ZERO,
  ): Promise<ExecutionResult> {
    if (entryPointArtifact.functionType !== FunctionType.SECRET) {
      throw new Error(`Cannot run ${entryPointArtifact.functionType} function as secret`);
    }

    if (request.origin !== contractAddress) {
      this.log.warn(
        `Request origin does not match contract address in simulation. Request origin: ${request.origin}, contract address: ${contractAddress}`,
      );
    }

    const curve = new Grumpkin();

    const blockHeader = await this.db.getBlockHeader();
    const callContext = new CallContext(
      msgSender,
      contractAddress,
      portalContractAddress,
      FunctionSelector.fromNameAndParameters(entryPointArtifact.name, entryPointArtifact.parameters),
      false,
      false,
      request.functionData.isConstructor,
      // TODO: when contract deployment is done in-app, we should only reserve one counter for the tx hash
      2, // 2 counters are reserved for tx hash and contract deployment nullifier
    );
    const context = new ClientExecutionContext(
      contractAddress,
      request.argsHash,
      request.txContext,
      callContext,
      blockHeader,
      request.authWitnesses,
      PackedArgsCache.create(request.packedArguments),
      new ExecutionNoteCache(),
      this.db,
      curve,
    );

    try {
      const executionResult = await executePrivateFunction(
        context,
        entryPointArtifact,
        contractAddress,
        request.functionData,
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
    entryPointArtifact: FunctionArtifactWithDebugMetadata,
    contractAddress: AztecAddress,
    aztecNode?: AztecNode,
  ) {
    if (entryPointArtifact.functionType !== FunctionType.UNCONSTRAINED) {
      throw new Error(`Cannot run ${entryPointArtifact.functionType} function as constrained`);
    }

    const blockHeader = await this.db.getBlockHeader();
    const context = new ViewDataOracle(contractAddress, blockHeader, [], this.db, aztecNode);

    try {
      return await executeUnconstrainedFunction(
        context,
        entryPointArtifact,
        contractAddress,
        request.functionData,
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
   * @param note - The note.
   * @returns The nullifier.
   */
  public async computeNoteHashAndNullifier(contractAddress: AztecAddress, nonce: Fr, storageSlot: Fr, note: Note) {
    const artifact: FunctionArtifactWithDebugMetadata | undefined = await this.db.getFunctionArtifactByName(
      contractAddress,
      'compute_note_hash_and_nullifier',
    );
    if (!artifact) {
      throw new Error(
        `Mandatory implementation of "compute_note_hash_and_nullifier" missing in noir contract ${contractAddress.toString()}.`,
      );
    }

    const maxNoteFields = (artifact.parameters[artifact.parameters.length - 1].type as ArrayType).length;
    if (maxNoteFields < note.items.length) {
      throw new Error(
        `The note being processed has ${note.items.length} fields, while "compute_note_hash_and_nullifier" can only handle a maximum of ${maxNoteFields} fields. Please consider increasing the allowed field size to accommodate all notes generated from the contract.`,
      );
    }

    const extendedNoteItems = note.items.concat(Array(maxNoteFields - note.items.length).fill(Fr.ZERO));

    const execRequest: FunctionCall = {
      to: contractAddress,
      functionData: FunctionData.empty(),
      args: encodeArguments(artifact, [contractAddress, nonce, storageSlot, extendedNoteItems]),
    };

    const [innerNoteHash, siloedNoteHash, uniqueSiloedNoteHash, innerNullifier] = (await this.runUnconstrained(
      execRequest,
      artifact,
      contractAddress,
    )) as bigint[];

    return {
      innerNoteHash: new Fr(innerNoteHash),
      siloedNoteHash: new Fr(siloedNoteHash),
      uniqueSiloedNoteHash: new Fr(uniqueSiloedNoteHash),
      innerNullifier: new Fr(innerNullifier),
    };
  }

  /**
   * Computes the inner note hash of a note, which contains storage slot and the custom note hash.
   * @param contractAddress - The address of the contract.
   * @param storageSlot - The storage slot.
   * @param note - The note.
   * @returns The note hash.
   */
  public async computeInnerNoteHash(contractAddress: AztecAddress, storageSlot: Fr, note: Note) {
    const { innerNoteHash } = await this.computeNoteHashAndNullifier(contractAddress, Fr.ZERO, storageSlot, note);
    return innerNoteHash;
  }

  /**
   * Computes the unique note hash of a note.
   * @param contractAddress - The address of the contract.
   * @param nonce - The nonce of the note hash.
   * @param storageSlot - The storage slot.
   * @param note - The note.
   * @returns The note hash.
   */
  public async computeUniqueSiloedNoteHash(contractAddress: AztecAddress, nonce: Fr, storageSlot: Fr, note: Note) {
    const { uniqueSiloedNoteHash } = await this.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, note);
    return uniqueSiloedNoteHash;
  }

  /**
   * Computes the siloed note hash of a note.
   * @param contractAddress - The address of the contract.
   * @param nonce - The nonce of the note hash.
   * @param storageSlot - The storage slot.
   * @param note - The note.
   * @returns The note hash.
   */
  public async computeSiloedNoteHash(contractAddress: AztecAddress, nonce: Fr, storageSlot: Fr, note: Note) {
    const { siloedNoteHash } = await this.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, note);
    return siloedNoteHash;
  }

  /**
   * Computes the inner note hash of a note, which contains storage slot and the custom note hash.
   * @param contractAddress - The address of the contract.
   * @param nonce - The nonce of the unique note hash.
   * @param storageSlot - The storage slot.
   * @param note - The note.
   * @returns The note hash.
   */
  public async computeInnerNullifier(contractAddress: AztecAddress, nonce: Fr, storageSlot: Fr, note: Note) {
    const { innerNullifier } = await this.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, note);
    return innerNullifier;
  }
}
