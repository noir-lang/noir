import { CallContext, PrivateHistoricTreeRoots, TxRequest } from '@aztec/circuits.js';
import { FunctionAbi, FunctionType } from '@aztec/foundation/abi';
import { DBOracle } from './db_oracle.js';
import { PrivateFunctionExecution, ExecutionResult } from './private_execution.js';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { pedersenCompressInputs, pedersenCompressWithHashIndex } from '@aztec/barretenberg.js/crypto';
import { UnconstrainedFunctionExecution } from './unconstrained_execution.js';
import { ClientTxExecutionContext } from './client_execution_context.js';
import { Fr } from '@aztec/foundation/fields';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';

export const NOTE_PEDERSEN_CONSTANT = new Fr(2n);
export const MAPPING_SLOT_PEDERSEN_CONSTANT = new Fr(4n);
export const NULLIFIER_PEDERSEN_CONSTANT = new Fr(5n);

const OUTER_NULLIFIER_GENERATOR_INDEX = 7;

/**
 * The ACIR simulator.
 */
export class AcirSimulator {
  constructor(private db: DBOracle) {}

  /**
   * Runs a private function.
   * @param request - The transaction request.
   * @param entryPointABI - The ABI of the entry point function.
   * @param contractAddress - The address of the contract.
   * @param portalContractAddress - The address of the portal contract.
   * @param historicRoots - The historic roots.
   * @returns The result of the execution.
   */
  public run(
    request: TxRequest,
    entryPointABI: FunctionAbi,
    contractAddress: AztecAddress,
    portalContractAddress: EthAddress,
    historicRoots: PrivateHistoricTreeRoots,
  ): Promise<ExecutionResult> {
    if (entryPointABI.functionType !== FunctionType.SECRET) {
      throw new Error(`Cannot run ${entryPointABI.functionType} function as secret`);
    }

    const callContext = new CallContext(
      request.from,
      contractAddress,
      portalContractAddress,
      false,
      false,
      request.functionData.isConstructor,
    );

    const execution = new PrivateFunctionExecution(
      new ClientTxExecutionContext(this.db, request, historicRoots),
      entryPointABI,
      contractAddress,
      request.functionData,
      request.args,
      callContext,
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
  public runUnconstrained(
    request: TxRequest,
    entryPointABI: FunctionAbi,
    contractAddress: AztecAddress,
    portalContractAddress: EthAddress,
    historicRoots: PrivateHistoricTreeRoots,
  ) {
    if (entryPointABI.functionType !== FunctionType.UNCONSTRAINED) {
      throw new Error(`Cannot run ${entryPointABI.functionType} function as constrained`);
    }
    const callContext = new CallContext(
      request.from,
      contractAddress,
      portalContractAddress,
      false,
      false,
      request.functionData.isConstructor,
    );

    const execution = new UnconstrainedFunctionExecution(
      new ClientTxExecutionContext(this.db, request, historicRoots),
      entryPointABI,
      contractAddress,
      request.functionData,
      request.args,
      callContext,
    );

    return execution.run();
  }

  // TODO Should be run as unconstrained function
  /**
   * Computes the hash of a note.
   * @param notePreimage - The note preimage.
   * @param bbWasm - The WASM instance.
   * @returns The note hash.
   */
  public computeNoteHash(notePreimage: Fr[], bbWasm: BarretenbergWasm) {
    return pedersenCompressInputs(bbWasm, [NOTE_PEDERSEN_CONSTANT.toBuffer(), ...notePreimage.map(x => x.toBuffer())]);
  }

  // TODO Should be run as unconstrained function
  /**
   * Computes the nullifier of a note.
   * @param notePreimage - The note preimage.
   * @param privateKey - The private key of the owner.
   * @param bbWasm - The WASM instance.
   * @returns The nullifier.
   */
  public computeNullifier(notePreimage: Fr[], privateKey: Buffer, bbWasm: BarretenbergWasm) {
    const noteHash = this.computeNoteHash(notePreimage, bbWasm);
    return pedersenCompressInputs(bbWasm, [NULLIFIER_PEDERSEN_CONSTANT.toBuffer(), noteHash, privateKey]);
  }

  // TODO Should be run as unconstrained function
  /**
   * Computes a nullifier siloed to a contract.
   * @param contractAddress - The address of the contract.
   * @param notePreimage - The note preimage.
   * @param privateKey - The private key of the owner.
   * @param bbWasm - The WASM instance.
   * @returns The siloed nullifier.
   */
  public computeSiloedNullifier(
    contractAddress: AztecAddress,
    notePreimage: Fr[],
    privateKey: Buffer,
    bbWasm: BarretenbergWasm,
  ) {
    const nullifier = this.computeNullifier(notePreimage, privateKey, bbWasm);
    return pedersenCompressWithHashIndex(
      bbWasm,
      [contractAddress.toBuffer(), nullifier],
      OUTER_NULLIFIER_GENERATOR_INDEX,
    );
  }
}
