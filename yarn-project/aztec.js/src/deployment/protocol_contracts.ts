import { getCanonicalClassRegisterer } from '@aztec/protocol-contracts/class-registerer';

import { UnsafeContract } from '../contract/unsafe_contract.js';
import { Wallet } from '../wallet/index.js';

/** Returns a Contract wrapper for the class registerer. */
export function getRegistererContract(wallet: Wallet) {
  const { artifact, instance } = getCanonicalClassRegisterer();
  return new UnsafeContract(instance, artifact, wallet);
}
