import { AccountInterface, AuthWitnessProvider, EntrypointInterface, FeeOptions } from '@aztec/aztec.js/account';
import { AuthWitness, FunctionCall, TxExecutionRequest } from '@aztec/circuit-types';
import { CompleteAddress, Fr } from '@aztec/circuits.js';
import { DefaultAccountEntrypoint } from '@aztec/entrypoints/account';
import { NodeInfo } from '@aztec/types/interfaces';

/**
 * Default implementation for an account interface. Requires that the account uses the default
 * entrypoint signature, which accept an AppPayload and a FeePayload as defined in noir-libs/aztec-noir/src/entrypoint module
 */
export class DefaultAccountInterface implements AccountInterface {
  private entrypoint: EntrypointInterface;

  constructor(
    private authWitnessProvider: AuthWitnessProvider,
    private address: CompleteAddress,
    nodeInfo: Pick<NodeInfo, 'chainId' | 'protocolVersion'>,
  ) {
    this.entrypoint = new DefaultAccountEntrypoint(
      address.address,
      authWitnessProvider,
      nodeInfo.chainId,
      nodeInfo.protocolVersion,
    );
  }

  createTxExecutionRequest(executions: FunctionCall[], fee?: FeeOptions): Promise<TxExecutionRequest> {
    return this.entrypoint.createTxExecutionRequest(executions, fee);
  }

  createAuthWitness(message: Fr): Promise<AuthWitness> {
    return this.authWitnessProvider.createAuthWitness(message);
  }

  getCompleteAddress(): CompleteAddress {
    return this.address;
  }
}
