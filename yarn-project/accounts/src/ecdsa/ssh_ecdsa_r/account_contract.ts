import { type AuthWitnessProvider } from '@aztec/aztec.js/account';
import { AuthWitness, type CompleteAddress } from '@aztec/circuit-types';
import { EcdsaSignature } from '@aztec/circuits.js/barretenberg';
import { type ContractArtifact } from '@aztec/foundation/abi';
import { type Fr } from '@aztec/foundation/fields';

import { DefaultAccountContract } from '../../defaults/account_contract.js';
import { signWithAgent } from '../../utils/ssh_agent.js';
import { EcdsaRAccountContractArtifact } from './artifact.js';

const secp256r1N = 115792089210356248762697446949407573529996955224135760342422259061068512044369n;
/**
 * Account contract that authenticates transactions using ECDSA signatures
 * verified against a secp256r1 public key stored in an immutable encrypted note.
 * Since this implementation relays signatures to an SSH agent, we provide the
 * public key here not for signature verification, but to identify actual identity
 * that will be used to sign authwitnesses.
 */
export class EcdsaRSSHAccountContract extends DefaultAccountContract {
  constructor(private signingPublicKey: Buffer) {
    super(EcdsaRAccountContractArtifact as ContractArtifact);
  }

  getDeploymentArgs() {
    return [this.signingPublicKey.subarray(0, 32), this.signingPublicKey.subarray(32, 64)];
  }

  getAuthWitnessProvider(_address: CompleteAddress): AuthWitnessProvider {
    return new SSHEcdsaRAuthWitnessProvider(this.signingPublicKey);
  }
}

/** Creates auth witnesses using ECDSA signatures. */
class SSHEcdsaRAuthWitnessProvider implements AuthWitnessProvider {
  constructor(private signingPublicKey: Buffer) {}

  #parseECDSASignature(data: Buffer) {
    // Extract ECDSA signature components
    let offset = 0;
    const sigTypeLen = data.readUInt32BE(offset);
    offset += 4;
    const sigType = data.subarray(offset, offset + sigTypeLen).toString();
    offset += sigTypeLen;

    if (sigType !== 'ecdsa-sha2-nistp256') {
      throw new Error(`Unexpected signature type: ${sigType}`);
    }

    offset += 4;
    const rLen = data.readUInt32BE(offset);
    offset += 4;
    let r = data.subarray(offset, offset + rLen);
    offset += rLen;

    const sLen = data.readUInt32BE(offset);
    offset += 4;
    let s = data.subarray(offset, offset + sLen);

    // R and S are encoded using ASN.1 DER format, which may include a leading zero byte to avoid interpreting the value as negative
    if (r.length > 32) {
      r = Buffer.from(Uint8Array.prototype.slice.call(r, 1));
    }

    if (s.length > 32) {
      s = Buffer.from(Uint8Array.prototype.slice.call(s, 1));
    }

    const maybeHighS = BigInt(`0x${s.toString('hex')}`);

    // ECDSA signatures must have a low S value so they can be used as a nullifier. BB forces a value of 27 for v, so
    // only one PublicKey can verify the signature (and not its negated counterpart) https://ethereum.stackexchange.com/a/55728
    if (maybeHighS > secp256r1N / 2n + 1n) {
      s = Buffer.from((secp256r1N - maybeHighS).toString(16), 'hex');
    }

    return new EcdsaSignature(r, s, Buffer.from([0]));
  }

  async createAuthWit(messageHash: Fr): Promise<AuthWitness> {
    // Key type and curve name
    const keyType = Buffer.from('ecdsa-sha2-nistp256');
    const curveName = Buffer.from('nistp256');
    const data = await signWithAgent(keyType, curveName, this.signingPublicKey, messageHash.toBuffer());
    const signature = this.#parseECDSASignature(data);

    return new AuthWitness(messageHash, [...signature.r, ...signature.s]);
  }
}
