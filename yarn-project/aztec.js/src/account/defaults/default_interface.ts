import { CompleteAddress, Fr } from '@aztec/circuits.js';
import { AuthWitness, FunctionCall, NodeInfo, TxExecutionRequest } from '@aztec/types';

import { AccountInterface, AuthWitnessProvider, EntrypointInterface } from '../interface.js';
import { DefaultAccountEntrypoint } from './default_entrypoint.js';

/**
 * Default implementation for an account interface. Requires that the account uses the default
 * entrypoint signature, which accepts an EntrypointPayload as defined in noir-libs/aztec-noir/src/entrypoint.nr.
 */
export class DefaultAccountInterface implements AccountInterface {
  private entrypoint: EntrypointInterface;

  constructor(
    private authWitnessProvider: AuthWitnessProvider,
    private address: CompleteAddress,
    nodeInfo: Pick<NodeInfo, 'chainId' | 'version'>,
  ) {
    this.entrypoint = new DefaultAccountEntrypoint(
      address.address,
      authWitnessProvider,
      nodeInfo.chainId,
      nodeInfo.version,
    );
  }

  createTxExecutionRequest(executions: FunctionCall[]): Promise<TxExecutionRequest> {
    return this.entrypoint.createTxExecutionRequest(executions);
  }
  createAuthWitness(message: Fr): Promise<AuthWitness> {
    return this.authWitnessProvider.createAuthWitness(message);
  }
  getCompleteAddress(): CompleteAddress {
    return this.address;
  }
}
