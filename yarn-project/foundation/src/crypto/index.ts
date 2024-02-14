import { BarretenbergSync } from '@aztec/bb.js';

export * from './keccak/index.js';
export * from './random/index.js';
export * from './sha256/index.js';
export * from './pedersen/index.js';
export * from './poseidon/index.js';

/**
 * Init the bb singleton. This constructs (if not already) the barretenberg sync api within bb.js itself.
 * It takes about 100-200ms to initialize. It may not seem like much, but when in conjunction with many other things
 * initializing, developers may want to pick precisely when to incur this cost.
 * If in a test environment, we'll just do it on module load.
 */
export async function init() {
  await BarretenbergSync.initSingleton();
}
