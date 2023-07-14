import { CallContext, FunctionData, PrivateHistoricTreeRoots, TxContext } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { ArrayType, FunctionAbi, FunctionType, encodeArguments } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { ExecutionRequest, TxExecutionRequest } from '@aztec/types';

import { PackedArgsCache } from '../packed_args_cache.js';
import { ClientTxExecutionContext } from './client_execution_context.js';
import { DBOracle } from './db_oracle.js';
import { ExecutionResult } from './execution_result.js';
import { computeNoteHashAndNullifierSelector, computeNoteHashAndNullifierSignature } from './function_selectors.js';
import { PrivateFunctionExecution } from './private_execution.js';
import { UnconstrainedFunctionExecution } from './unconstrained_execution.js';

/**
 * The ACIR simulator.
 */
export class AcirSimulator {
  private log: DebugLogger;

  constructor(private db: DBOracle) {
    this.log = createDebugLogger('aztec:simulator');
  }

  /**
   * Runs a private function.
   * @param request - The transaction request.
   * @param entryPointABI - The ABI of the entry point function.
   * @param contractAddress - The address of the contract (should match request.origin)
   * @param portalContractAddress - The address of the portal contract.
   * @param historicRoots - The historic roots.
   * @param curve - The curve instance for elliptic curve operations.
   * @param packedArguments - The entrypoint packed arguments
   * @returns The result of the execution.
   */
  public async run(
    request: TxExecutionRequest,
    entryPointABI: FunctionAbi,
    contractAddress: AztecAddress,
    portalContractAddress: EthAddress,
    historicRoots: PrivateHistoricTreeRoots,
  ): Promise<ExecutionResult> {
    if (entryPointABI.functionType !== FunctionType.SECRET) {
      throw new Error(`Cannot run ${entryPointABI.functionType} function as secret`);
    }

    if (request.origin !== contractAddress) {
      this.log(`WARN: Request origin does not match contract address in simulation`);
    }

    const curve = await Grumpkin.new();

    const callContext = new CallContext(
      AztecAddress.ZERO,
      contractAddress,
      portalContractAddress,
      false,
      false,
      request.functionData.isConstructor,
    );

    const execution = new PrivateFunctionExecution(
      new ClientTxExecutionContext(
        this.db,
        request.txContext,
        historicRoots,
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
   * @param entryPointABI - The ABI of the entry point function.
   * @param contractAddress - The address of the contract.
   * @param portalContractAddress - The address of the portal contract.
   * @param historicRoots - The historic roots.
   * @returns The return values of the function.
   */
  public async runUnconstrained(
    request: ExecutionRequest,
    entryPointABI: FunctionAbi,
    contractAddress: AztecAddress,
    portalContractAddress: EthAddress,
    historicRoots: PrivateHistoricTreeRoots,
  ) {
    if (entryPointABI.functionType !== FunctionType.UNCONSTRAINED) {
      throw new Error(`Cannot run ${entryPointABI.functionType} function as constrained`);
    }
    const callContext = new CallContext(
      request.from!,
      contractAddress,
      portalContractAddress,
      false,
      false,
      request.functionData.isConstructor,
    );

    const execution = new UnconstrainedFunctionExecution(
      new ClientTxExecutionContext(this.db, TxContext.empty(), historicRoots, await PackedArgsCache.create([])),
      entryPointABI,
      contractAddress,
      request.functionData,
      request.args,
      callContext,
    );

    return execution.run();
  }

  /**
   * Computes the inner nullifier of a note.
   * @param contractAddress - The address of the contract.
   * @param storageSlot - The storage slot.
   * @param notePreimage - The note preimage.
   * @returns The nullifier.
   */
  public async computeNoteHashAndNullifier(contractAddress: AztecAddress, storageSlot: Fr, notePreimage: Fr[]) {
    const abi = await this.db.getFunctionABI(contractAddress, computeNoteHashAndNullifierSelector);
    if (!abi) {
      throw new Error(
        `Please define an unconstrained function "${computeNoteHashAndNullifierSignature}" in the noir contract.`,
      );
    }

    const preimageLen = (abi.parameters[2].type as ArrayType).length;
    const extendedPreimage = notePreimage.concat(Array(preimageLen - notePreimage.length).fill(Fr.ZERO));

    const execRequest: ExecutionRequest = {
      from: AztecAddress.ZERO,
      to: AztecAddress.ZERO,
      functionData: FunctionData.empty(),
      args: encodeArguments(abi, [contractAddress, storageSlot, extendedPreimage]),
    };

    const [result] = await this.runUnconstrained(
      execRequest,
      abi,
      AztecAddress.ZERO,
      EthAddress.ZERO,
      PrivateHistoricTreeRoots.empty(),
    );

    return {
      noteHash: new Fr(result[0]),
      nullifier: new Fr(result[1]),
    };
  }

  /**
   * Computes the inner note hash of a note, which contains storage slot and the custom note hash.
   * @param contractAddress - The address of the contract.
   * @param storageSlot - The storage slot.
   * @param notePreimage - The note preimage.
   * @param abi - The ABI of the function `compute_note_hash`.
   * @returns The note hash.
   */
  public async computeNoteHash(contractAddress: AztecAddress, storageSlot: Fr, notePreimage: Fr[]) {
    const { noteHash } = await this.computeNoteHashAndNullifier(contractAddress, storageSlot, notePreimage);
    return noteHash;
  }

  /**
   * Computes the inner note hash of a note, which contains storage slot and the custom note hash.
   * @param contractAddress - The address of the contract.
   * @param storageSlot - The storage slot.
   * @param notePreimage - The note preimage.
   * @param abi - The ABI of the function `compute_note_hash`.
   * @returns The note hash.
   */
  public async computeNullifier(contractAddress: AztecAddress, storageSlot: Fr, notePreimage: Fr[]) {
    const { nullifier } = await this.computeNoteHashAndNullifier(contractAddress, storageSlot, notePreimage);
    return nullifier;
  }
}
