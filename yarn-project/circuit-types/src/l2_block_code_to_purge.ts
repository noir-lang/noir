import {
  AppendOnlyTreeSnapshot,
  AztecAddress,
  ContentCommitment,
  EthAddress,
  Fr,
  GlobalVariables,
  Header,
  NUM_BYTES_PER_SHA256,
  PartialStateReference,
  StateReference,
} from '@aztec/circuits.js';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';

/**
 * Makes header.
 */
export function makeHeader(
  seed = 0,
  blockNumber: number | undefined = undefined,
  txsEffectsHash: Buffer | undefined = undefined,
  inHash: Buffer | undefined = undefined,
): Header {
  return new Header(
    makeAppendOnlyTreeSnapshot(seed + 0x100),
    makeContentCommitment(seed + 0x200, txsEffectsHash, inHash),
    makeStateReference(seed + 0x600),
    makeGlobalVariables((seed += 0x700), blockNumber),
  );
}

/**
 * Makes arbitrary append only tree snapshot.
 * @param seed - The seed to use for generating the append only tree snapshot.
 * @returns An append only tree snapshot.
 */
export function makeAppendOnlyTreeSnapshot(seed = 1): AppendOnlyTreeSnapshot {
  return new AppendOnlyTreeSnapshot(new Fr(seed), seed);
}

/**
 * Makes content commitment
 */
function makeContentCommitment(
  seed = 0,
  txsEffectsHash: Buffer | undefined = undefined,
  inHash: Buffer | undefined = undefined,
): ContentCommitment {
  return new ContentCommitment(
    new Fr(seed),
    txsEffectsHash ?? toBufferBE(BigInt(seed + 0x100), NUM_BYTES_PER_SHA256),
    inHash ?? toBufferBE(BigInt(seed + 0x200), NUM_BYTES_PER_SHA256),
    toBufferBE(BigInt(seed + 0x300), NUM_BYTES_PER_SHA256),
  );
}

/**
 * Makes arbitrary state reference.
 * @param seed - The seed to use for generating the state reference.
 * @returns A state reference.
 */
function makeStateReference(seed = 0): StateReference {
  return new StateReference(makeAppendOnlyTreeSnapshot(seed), makePartialStateReference(seed + 1));
}

/**
 * Makes arbitrary partial state reference.
 * @param seed - The seed to use for generating the partial state reference.
 * @returns A partial state reference.
 */
function makePartialStateReference(seed = 0): PartialStateReference {
  return new PartialStateReference(
    makeAppendOnlyTreeSnapshot(seed),
    makeAppendOnlyTreeSnapshot(seed + 1),
    makeAppendOnlyTreeSnapshot(seed + 2),
  );
}

/**
 * Makes global variables.
 * @param seed - The seed to use for generating the global variables.
 * @param blockNumber - The block number to use for generating the global variables.
 * If blockNumber is undefined, it will be set to seed + 2.
 * @returns Global variables.
 */
export function makeGlobalVariables(seed = 1, blockNumber: number | undefined = undefined): GlobalVariables {
  if (blockNumber !== undefined) {
    return new GlobalVariables(
      new Fr(seed),
      new Fr(seed + 1),
      new Fr(blockNumber),
      new Fr(seed + 3),
      EthAddress.fromField(new Fr(seed + 4)),
      AztecAddress.fromField(new Fr(seed + 5)),
    );
  }
  return new GlobalVariables(
    new Fr(seed),
    new Fr(seed + 1),
    new Fr(seed + 2),
    new Fr(seed + 3),
    EthAddress.fromField(new Fr(seed + 4)),
    AztecAddress.fromField(new Fr(seed + 5)),
  );
}
