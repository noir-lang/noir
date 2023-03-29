import { EthAddress, OldTreeRoots, TxRequest, AztecAddress } from '@aztec/circuits.js';
import { DBOracle } from './db_oracle.js';
import { Execution, ExecutionResult } from './execution.js';

export class AcirSimulator {
  constructor(private db: DBOracle) {}

  run(
    request: TxRequest,
    entryPointACIR: Buffer,
    contractAddress: AztecAddress,
    portalContractAddress: EthAddress,
    oldRoots: OldTreeRoots,
  ): Promise<ExecutionResult> {
    const execution = new Execution(this.db, request, entryPointACIR, contractAddress, portalContractAddress, oldRoots);

    return execution.run();
  }
}
