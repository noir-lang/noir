import { type AuthWitness, type FunctionCall, type PXE, type TxExecutionRequest } from '@aztec/circuit-types';
import { type CompleteAddress, type Fr } from '@aztec/circuits.js';

import { DefaultEntrypoint } from '../entrypoint/default_entrypoint.js';
import { type EntrypointInterface } from '../entrypoint/entrypoint.js';
import { BaseWallet } from './base_wallet.js';

/**
 * Wallet implementation which creates a transaction request directly to the requested contract without any signing.
 */
export class SignerlessWallet extends BaseWallet {
  constructor(pxe: PXE, private entrypoint?: EntrypointInterface) {
    super(pxe);
  }

  async createTxExecutionRequest(executions: FunctionCall[]): Promise<TxExecutionRequest> {
    let entrypoint = this.entrypoint;
    if (!entrypoint) {
      const { chainId, protocolVersion } = await this.pxe.getNodeInfo();
      entrypoint = new DefaultEntrypoint(chainId, protocolVersion);
    }

    return entrypoint.createTxExecutionRequest(executions);
  }

  getChainId(): Fr {
    throw new Error('Method not implemented.');
  }

  getVersion(): Fr {
    throw new Error('Method not implemented.');
  }

  getCompleteAddress(): CompleteAddress {
    throw new Error('Method not implemented.');
  }

  createAuthWit(_messageHash: Fr): Promise<AuthWitness> {
    throw new Error('Method not implemented.');
  }
}
