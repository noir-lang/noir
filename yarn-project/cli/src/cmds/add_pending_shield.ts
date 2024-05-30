import { type AztecAddress, Fr, computeSecretHash } from '@aztec/aztec.js';
import { ExtendedNote, Note, type TxHash } from '@aztec/circuit-types';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';
import { TokenContract } from '@aztec/noir-contracts.js';

import { createCompatibleClient } from '../client.js';

export async function addPendingShield(
  ownerAddress: AztecAddress,
  tokenAddress: AztecAddress,
  amount: bigint,
  secret: Fr,
  txHash: TxHash,
  rpcUrl: string,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const secretHash = computeSecretHash(secret);
  const note = new Note([new Fr(amount), secretHash]);
  const extendedNote = new ExtendedNote(
    note,
    ownerAddress,
    tokenAddress,
    TokenContract.storage.pending_shields.slot,
    TokenContract.notes.TransparentNote.id,
    txHash,
  );
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  await client.addNote(extendedNote);
  log(`Added pending shield note owned by ${ownerAddress.toString()} for ${amount}`);
}

// await t.addPendingShieldNoteToPXE(bobsAddress, maxFee - actualFee, computeSecretHash(rebateSecret), tx.txHash);
