import { type AccountWalletWithSecretKey, type AuthWitness, type AztecAddress } from '@aztec/aztec.js';
import { type LogFn } from '@aztec/foundation/log';

export async function addAuthwit(
  wallet: AccountWalletWithSecretKey,
  authwit: AuthWitness,
  authorizer: AztecAddress,
  log: LogFn,
) {
  await wallet.addAuthWitness(authwit);

  log(`Added authorization witness from ${authorizer}`);
}
