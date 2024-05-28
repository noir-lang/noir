import { type AuthWitness, type PXE, type TxExecutionRequest } from '@aztec/circuit-types';
import { type CompleteAddress, type Fq, type Fr } from '@aztec/circuits.js';

import { DefaultEntrypoint } from '../entrypoint/default_entrypoint.js';
import { type EntrypointInterface, type ExecutionRequestInit } from '../entrypoint/entrypoint.js';
import { BaseWallet } from './base_wallet.js';

/**
 * Wallet implementation which creates a transaction request directly to the requested contract without any signing.
 */
export class SignerlessWallet extends BaseWallet {
  constructor(pxe: PXE, private entrypoint?: EntrypointInterface) {
    super(pxe);
  }

  async createTxExecutionRequest(execution: ExecutionRequestInit): Promise<TxExecutionRequest> {
    let entrypoint = this.entrypoint;
    if (!entrypoint) {
      const { chainId, protocolVersion } = await this.pxe.getNodeInfo();
      entrypoint = new DefaultEntrypoint(chainId, protocolVersion);
    }

    return entrypoint.createTxExecutionRequest(execution);
  }

  getChainId(): Fr {
    throw new Error('SignerlessWallet: Method getChainId not implemented.');
  }

  getVersion(): Fr {
    throw new Error('SignerlessWallet: Method getVersion not implemented.');
  }

  getPublicKeysHash(): Fr {
    throw new Error('SignerlessWallet: Method getPublicKeysHash not implemented.');
  }

  getCompleteAddress(): CompleteAddress {
    throw new Error('SignerlessWallet: Method getCompleteAddress not implemented.');
  }

  createAuthWit(_messageHash: Fr): Promise<AuthWitness> {
    throw new Error('SignerlessWallet: Method createAuthWit not implemented.');
  }

  rotateNullifierKeys(_newNskM: Fq): Promise<void> {
    throw new Error('SignerlessWallet: Method rotateNullifierKeys not implemented.');
  }
}
