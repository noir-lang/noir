import { CallContext, CircuitsWasm, FunctionData, TxContext } from '@aztec/circuits.js';
import { computeTxHash } from '@aztec/circuits.js/abis';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { ArrayType, FunctionType, encodeArguments } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { AztecNode, FunctionCall, TxExecutionRequest } from '@aztec/types';

import { WasmBlackBoxFunctionSolver, createBlackBoxSolver } from 'acvm_js';

import { PackedArgsCache } from '../packed_args_cache.js';
import { ClientTxExecutionContext } from './client_execution_context.js';
import { DBOracle, FunctionAbiWithDebugMetadata } from './db_oracle.js';
import { ExecutionResult } from './execution_result.js';
import { computeNoteHashAndNullifierSelector, computeNoteHashAndNullifierSignature } from './function_selectors.js';
import { PrivateFunctionExecution } from './private_execution.js';
import { UnconstrainedFunctionExecution } from './unconstrained_execution.js';

/**
 * The ACIR simulator.
 */
export class AcirSimulator {
  private static solver: WasmBlackBoxFunctionSolver; // ACVM's backend
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
  public static async getSolver(): Promise<WasmBlackBoxFunctionSolver> {
    if (!this.solver) this.solver = await createBlackBoxSolver();
    return this.solver;
  }

  /**
   * Runs a private function.
   * @param request - The transaction request.
   * @param entryPointABI - The ABI of the entry point function.
   * @param contractAddress - The address of the contract (should match request.origin)
   * @param portalContractAddress - The address of the portal contract.
   * @param msgSender - The address calling the function. This can be replaced to simulate a call from another contract or a specific account.
   * @returns The result of the execution.
   */
  public async run(
    request: TxExecutionRequest,
    entryPointABI: FunctionAbiWithDebugMetadata,
    contractAddress: AztecAddress,
    portalContractAddress: EthAddress,
    msgSender = AztecAddress.ZERO,
  ): Promise<ExecutionResult> {
    if (entryPointABI.functionType !== FunctionType.SECRET) {
      throw new Error(`Cannot run ${entryPointABI.functionType} function as secret`);
    }

    if (request.origin !== contractAddress) {
      this.log.warn('Request origin does not match contract address in simulation');
    }

    const curve = await Grumpkin.new();

    const historicBlockData = await this.db.getHistoricBlockData();
    const callContext = new CallContext(
      msgSender,
      contractAddress,
      portalContractAddress,
      false,
      false,
      request.functionData.isConstructor,
    );

    const wasm = await CircuitsWasm.get();
    const txNullifier = computeTxHash(wasm, request.toTxRequest());
    const execution = new PrivateFunctionExecution(
      new ClientTxExecutionContext(
        this.db,
        txNullifier,
        request.txContext,
        historicBlockData,
        await PackedArgsCache.create(request.packedArguments),
      ),
      entryPointABI,
      contractAddress,
      request.functionData,
      request.argsHash,
      callContext,
      curve,
    );

    return execution.run();
  }

  /**
   * Runs an unconstrained function.
   * @param request - The transaction request.
   * @param origin - The sender of the request.
   * @param entryPointABI - The ABI of the entry point function.
   * @param contractAddress - The address of the contract.
   * @param portalContractAddress - The address of the portal contract.
   * @param historicBlockData - Block data containing historic roots.
   * @param aztecNode - The AztecNode instance.
   */
  public async runUnconstrained(
    request: FunctionCall,
    origin: AztecAddress,
    entryPointABI: FunctionAbiWithDebugMetadata,
    contractAddress: AztecAddress,
    portalContractAddress: EthAddress,
    aztecNode?: AztecNode,
  ) {
    if (entryPointABI.functionType !== FunctionType.UNCONSTRAINED) {
      throw new Error(`Cannot run ${entryPointABI.functionType} function as constrained`);
    }

    const historicBlockData = await this.db.getHistoricBlockData();
    const callContext = new CallContext(
      origin,
      contractAddress,
      portalContractAddress,
      false,
      false,
      request.functionData.isConstructor,
    );

    const execution = new UnconstrainedFunctionExecution(
      new ClientTxExecutionContext(
        this.db,
        Fr.ZERO,
        TxContext.empty(),
        historicBlockData,
        await PackedArgsCache.create([]),
      ),
      entryPointABI,
      contractAddress,
      request.functionData,
      request.args,
      callContext,
    );

    return execution.run(aztecNode);
  }

  /**
   * Computes the inner nullifier of a note.
   * @param contractAddress - The address of the contract.
   * @param nonce - The nonce of the note hash.
   * @param storageSlot - The storage slot.
   * @param notePreimage - The note preimage.
   * @returns The nullifier.
   */
  public async computeNoteHashAndNullifier(
    contractAddress: AztecAddress,
    nonce: Fr,
    storageSlot: Fr,
    notePreimage: Fr[],
  ) {
    try {
      const abi = await this.db.getFunctionABI(contractAddress, computeNoteHashAndNullifierSelector);

      const preimageLen = (abi.parameters[3].type as ArrayType).length;
      const extendedPreimage = notePreimage.concat(Array(preimageLen - notePreimage.length).fill(Fr.ZERO));

      const execRequest: FunctionCall = {
        to: AztecAddress.ZERO,
        functionData: FunctionData.empty(),
        args: encodeArguments(abi, [contractAddress, nonce, storageSlot, extendedPreimage]),
      };

      const [innerNoteHash, siloedNoteHash, uniqueSiloedNoteHash, innerNullifier] = (await this.runUnconstrained(
        execRequest,
        AztecAddress.ZERO,
        abi,
        AztecAddress.ZERO,
        EthAddress.ZERO,
      )) as bigint[];

      return {
        innerNoteHash: new Fr(innerNoteHash),
        siloedNoteHash: new Fr(siloedNoteHash),
        uniqueSiloedNoteHash: new Fr(uniqueSiloedNoteHash),
        innerNullifier: new Fr(innerNullifier),
      };
    } catch (e) {
      throw new Error(
        `Mandatory implementation of "${computeNoteHashAndNullifierSignature}" missing in noir contract ${contractAddress.toString()}.`,
      );
    }
  }

  /**
   * Computes the inner note hash of a note, which contains storage slot and the custom note hash.
   * @param contractAddress - The address of the contract.
   * @param storageSlot - The storage slot.
   * @param notePreimage - The note preimage.
   * @param abi - The ABI of the function `compute_note_hash`.
   * @returns The note hash.
   */
  public async computeInnerNoteHash(contractAddress: AztecAddress, storageSlot: Fr, notePreimage: Fr[]) {
    const { innerNoteHash } = await this.computeNoteHashAndNullifier(
      contractAddress,
      Fr.ZERO,
      storageSlot,
      notePreimage,
    );
    return innerNoteHash;
  }

  /**
   * Computes the unique note hash of a note.
   * @param contractAddress - The address of the contract.
   * @param nonce - The nonce of the note hash.
   * @param storageSlot - The storage slot.
   * @param notePreimage - The note preimage.
   * @param abi - The ABI of the function `compute_note_hash`.
   * @returns The note hash.
   */
  public async computeUniqueSiloedNoteHash(
    contractAddress: AztecAddress,
    nonce: Fr,
    storageSlot: Fr,
    notePreimage: Fr[],
  ) {
    const { uniqueSiloedNoteHash } = await this.computeNoteHashAndNullifier(
      contractAddress,
      nonce,
      storageSlot,
      notePreimage,
    );
    return uniqueSiloedNoteHash;
  }

  /**
   * Computes the siloed note hash of a note.
   * @param contractAddress - The address of the contract.
   * @param nonce - The nonce of the note hash.
   * @param storageSlot - The storage slot.
   * @param notePreimage - The note preimage.
   * @param abi - The ABI of the function `compute_note_hash`.
   * @returns The note hash.
   */
  public async computeSiloedNoteHash(contractAddress: AztecAddress, nonce: Fr, storageSlot: Fr, notePreimage: Fr[]) {
    const { siloedNoteHash } = await this.computeNoteHashAndNullifier(
      contractAddress,
      nonce,
      storageSlot,
      notePreimage,
    );
    return siloedNoteHash;
  }

  /**
   * Computes the inner note hash of a note, which contains storage slot and the custom note hash.
   * @param contractAddress - The address of the contract.
   * @param nonce - The nonce of the unique note hash.
   * @param storageSlot - The storage slot.
   * @param notePreimage - The note preimage.
   * @param abi - The ABI of the function `compute_note_hash`.
   * @returns The note hash.
   */
  public async computeInnerNullifier(contractAddress: AztecAddress, nonce: Fr, storageSlot: Fr, notePreimage: Fr[]) {
    const { innerNullifier } = await this.computeNoteHashAndNullifier(
      contractAddress,
      nonce,
      storageSlot,
      notePreimage,
    );
    return innerNullifier;
  }
}
