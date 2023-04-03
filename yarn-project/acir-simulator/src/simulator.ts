import { AztecAddress, EthAddress } from '@aztec/foundation';
import { OldTreeRoots, TxRequest } from '@aztec/circuits.js';
import { FunctionAbi } from '@aztec/noir-contracts';
import { DBOracle } from './db_oracle.js';
import { Execution, ExecutionResult } from './execution.js';

export class AcirSimulator {
  constructor(private db: DBOracle) {}

  run(
    request: TxRequest,
    entryPointABI: FunctionAbi,
    contractAddress: AztecAddress,
    portalContractAddress: EthAddress,
    oldRoots: OldTreeRoots,
  ): Promise<ExecutionResult> {
    const execution = new Execution(this.db, request, entryPointABI, contractAddress, portalContractAddress, oldRoots);

    return execution.run();
  }
}
