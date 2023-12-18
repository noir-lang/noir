import { init } from '@aztec/foundation/crypto';

/**
 * This should only be needed to be called in CJS environments that don't have top level await.
 * Initializes any asynchronous subsystems required to use the library.
 * At time of writing, this is just our foundation crypto lib.
 */
export async function initAztecJs() {
  await init();
}
