import { AztecRPC } from '@aztec/aztec-rpc';
import { AztecAddress, Point } from '@aztec/circuits.js';
import { SentTx } from '../index.js';
import { createDebugLogger } from '@aztec/foundation/log';

/**
 * Creates an Aztec Account.
 * @returns The account's address & public key.
 */
export async function createAccounts(
  aztecRpcClient: AztecRPC,
  privateKey: Buffer,
  numberOfAccounts = 1,
  logger = createDebugLogger('aztec:aztec.js:accounts'),
): Promise<[AztecAddress, Point][]> {
  const results: [AztecAddress, Point][] = [];
  for (let i = 0; i < numberOfAccounts; ++i) {
    // We use the well-known private key and the validating account contract for the first account,
    // and generate random keypairs with gullible account contracts (ie no sig validation) for the rest.
    // TODO(#662): Let the aztec rpc server generate the keypair rather than hardcoding the private key
    const privKey = i == 0 ? privateKey : undefined;
    const [txHash, newAddress] = await aztecRpcClient.createSmartAccount(privKey);
    // wait for tx to be mined
    await new SentTx(aztecRpcClient, Promise.resolve(txHash)).isMined();
    const address = newAddress;
    const pubKey = await aztecRpcClient.getAccountPublicKey(address);
    logger(`Created account ${address.toString()} with public key ${pubKey.toString()}`);
    results.push([newAddress, pubKey]);
  }
  return results;
}
