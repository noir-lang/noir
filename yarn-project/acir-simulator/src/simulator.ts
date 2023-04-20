import { AztecAddress, EthAddress, Fr } from '@aztec/foundation';
import { CallContext, PrivateHistoricTreeRoots, TxRequest } from '@aztec/circuits.js';
import { FunctionAbi } from '@aztec/noir-contracts';
import { DBOracle } from './db_oracle.js';
import { Execution, ExecutionResult } from './execution.js';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { pedersenCompressInputs, pedersenCompressWithHashIndex } from '@aztec/barretenberg.js/crypto';

export const NOTE_PEDERSEN_CONSTANT = new Fr(2n);
export const MAPPING_SLOT_PEDERSEN_CONSTANT = new Fr(4n);
export const NULLIFIER_PEDERSEN_CONSTANT = new Fr(5n);

const OUTER_NULLIFIER_GENERATOR_INDEX = 7;

export class AcirSimulator {
  constructor(private db: DBOracle) {}

  public run(
    request: TxRequest,
    entryPointABI: FunctionAbi,
    contractAddress: AztecAddress,
    portalContractAddress: EthAddress,
    historicRoots: PrivateHistoricTreeRoots,
  ): Promise<ExecutionResult> {
    const callContext = new CallContext(
      request.from,
      contractAddress,
      portalContractAddress,
      false,
      false,
      request.functionData.isConstructor,
    );

    const execution = new Execution(
      this.db,
      request,
      historicRoots,
      entryPointABI,
      contractAddress,
      request.functionData,
      request.args,
      callContext,
    );

    return execution.run();
  }

  // TODO Should be run as unconstrained function
  public computeNoteHash(notePreimage: Fr[], bbWasm: BarretenbergWasm) {
    return pedersenCompressInputs(bbWasm, [NOTE_PEDERSEN_CONSTANT.toBuffer(), ...notePreimage.map(x => x.toBuffer())]);
  }

  // TODO Should be run as unconstrained function
  public computeNullifier(notePreimage: Fr[], privateKey: Buffer, bbWasm: BarretenbergWasm) {
    const noteHash = this.computeNoteHash(notePreimage, bbWasm);
    return pedersenCompressInputs(bbWasm, [NULLIFIER_PEDERSEN_CONSTANT.toBuffer(), noteHash, privateKey]);
  }

  // TODO Should be run as unconstrained function
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
