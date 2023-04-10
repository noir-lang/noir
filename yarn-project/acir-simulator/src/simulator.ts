import { AztecAddress, EthAddress, Fr } from '@aztec/foundation';
import { OldTreeRoots, TxRequest } from '@aztec/circuits.js';
import { FunctionAbi } from '@aztec/noir-contracts';
import { DBOracle } from './db_oracle.js';
import { Execution, ExecutionResult } from './execution.js';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { pedersenCompressInputs } from '@aztec/barretenberg.js/crypto';

export const NOTE_PEDERSEN_CONSTANT = new Fr(2n);
export const MAPPING_SLOT_PEDERSEN_CONSTANT = new Fr(4n);
export const NULLIFIER_PEDERSEN_CONSTANT = new Fr(5n);

export class AcirSimulator {
  constructor(private db: DBOracle) {}

  public run(
    request: TxRequest,
    entryPointABI: FunctionAbi,
    contractAddress: AztecAddress,
    portalContractAddress: EthAddress,
    oldRoots: OldTreeRoots,
  ): Promise<ExecutionResult> {
    const execution = new Execution(this.db, request, entryPointABI, contractAddress, portalContractAddress, oldRoots);

    return execution.run();
  }

  public computeNoteHash(notePreimage: Fr[], bbWasm: BarretenbergWasm) {
    return pedersenCompressInputs(bbWasm, [NOTE_PEDERSEN_CONSTANT.toBuffer(), ...notePreimage.map(x => x.toBuffer())]);
  }

  public computeNullifier(notePreimage: Fr[], privateKey: Buffer, bbWasm: BarretenbergWasm) {
    const noteHash = this.computeNoteHash(notePreimage, bbWasm);
    return pedersenCompressInputs(bbWasm, [NULLIFIER_PEDERSEN_CONSTANT.toBuffer(), noteHash, privateKey]);
  }
}
