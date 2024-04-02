import { L1Actor, L1ToL2Message, L2Actor } from '@aztec/circuit-types';
import { type AztecAddress, EthAddress, type Fr } from '@aztec/circuits.js';
import { computeMessageSecretHash } from '@aztec/circuits.js/hash';
import { sha256ToField } from '@aztec/foundation/crypto';

/**
 * Test utility function to craft an L1 to L2 message.
 * @param selector - The cross chain message selector.
 * @param contentPreimage - The args after the selector.
 * @param targetContract - The contract to consume the message.
 * @param secret - The secret to unlock the message.
 * @returns The L1 to L2 message.
 */
export const buildL1ToL2Message = (
  selector: string,
  contentPreimage: Fr[],
  targetContract: AztecAddress,
  secret: Fr,
) => {
  // Write the selector into a buffer.
  const selectorBuf = Buffer.from(selector, 'hex');

  const content = sha256ToField([selectorBuf, ...contentPreimage]);
  const secretHash = computeMessageSecretHash(secret);

  // Eventually the kernel will need to prove the kernel portal pair exists within the contract tree,
  // EthAddress.random() will need to be replaced when this happens
  return new L1ToL2Message(new L1Actor(EthAddress.random(), 1), new L2Actor(targetContract, 1), content, secretHash);
};
