import { GrumpkinScalar, generatePublicKey } from '@aztec/aztec.js';
import { type LogFn } from '@aztec/foundation/log';

import { mnemonicToAccount } from 'viem/accounts';

export function generatePrivateKey(mnemonic: string | undefined, log: LogFn) {
  let privKey;
  let publicKey;
  if (mnemonic) {
    const acc = mnemonicToAccount(mnemonic);
    // TODO(#2052): This reduction is not secure enough. TACKLE THIS ISSUE BEFORE MAINNET.
    const key = GrumpkinScalar.fromBufferReduce(Buffer.from(acc.getHdKey().privateKey!));
    publicKey = generatePublicKey(key);
  } else {
    const key = GrumpkinScalar.random();
    privKey = key.toString();
    publicKey = generatePublicKey(key);
  }
  log(`\nPrivate Key: ${privKey}\nPublic Key: ${publicKey.toString()}\n`);
}
