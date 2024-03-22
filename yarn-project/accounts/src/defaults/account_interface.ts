import { AccountInterface, AuthWitnessProvider } from '@aztec/aztec.js/account';
import { EntrypointInterface, FeeOptions } from '@aztec/aztec.js/entrypoint';
import { AuthWitness, FunctionCall, TxExecutionRequest } from '@aztec/circuit-types';
import { AztecAddress, CompleteAddress, Fr } from '@aztec/circuits.js';
import { DefaultAccountEntrypoint } from '@aztec/entrypoints/account';
import { NodeInfo } from '@aztec/types/interfaces';

/**
 * Default implementation for an account interface. Requires that the account uses the default
 * entrypoint signature, which accept an AppPayload and a FeePayload as defined in noir-libs/aztec-noir/src/entrypoint module
 */
export class DefaultAccountInterface implements AccountInterface {
  private entrypoint: EntrypointInterface;
  private chainId: Fr;
  private version: Fr;

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
    this.chainId = new Fr(nodeInfo.chainId);
    this.version = new Fr(nodeInfo.protocolVersion);
  }

  createTxExecutionRequest(executions: FunctionCall[], fee?: FeeOptions): Promise<TxExecutionRequest> {
    return this.entrypoint.createTxExecutionRequest(executions, fee);
  }

  createAuthWit(messageHash: Fr): Promise<AuthWitness> {
    return this.authWitnessProvider.createAuthWit(messageHash);
  }

  getCompleteAddress(): CompleteAddress {
    return this.address;
  }

  getAddress(): AztecAddress {
    return this.address.address;
  }

  getChainId(): Fr {
    return this.chainId;
  }

  getVersion(): Fr {
    return this.version;
  }
}
