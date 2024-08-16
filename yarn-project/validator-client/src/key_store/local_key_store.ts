import { Signature } from '@aztec/circuit-types';

import { type PrivateKeyAccount, privateKeyToAccount } from 'viem/accounts';

import { type ValidatorKeyStore } from './interface.js';

/**
 * Local Key Store
 *
 * An implementation of the Key store using an in memory private key.
 */
export class LocalKeyStore implements ValidatorKeyStore {
  private signer: PrivateKeyAccount;

  constructor(privateKey: string) {
    this.signer = privateKeyToAccount(privateKey as `0x{string}`);
  }

  /**
   * Sign a message with the keystore private key
   *
   * @param messageBuffer - The message buffer to sign
   * @return signature
   */
  public async sign(digestBuffer: Buffer): Promise<Signature> {
    const digest: `0x${string}` = `0x${digestBuffer.toString('hex')}`;
    const signature = await this.signer.signMessage({ message: { raw: digest } });

    return Signature.from0xString(signature);
  }
}
