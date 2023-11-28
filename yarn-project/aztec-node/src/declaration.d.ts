// eslint-disable-next-line @typescript-eslint/no-unused-vars
import { RootDatabaseOptionsWithPath } from 'lmdb';

// The problem is this snippet `nodeDb = open({});` in src/aztec-node/db.ts
// tsc compiles this code fine, but ts-jest can't.
// This is a mixture for two bugs:
// - the first in ts-jest, it gets confused by packages with mixed CJS and ESM type exports - https://github.com/kulshekhar/ts-jest/issues/4221
// - the second in lmdb, it outputs different CJS and ESM types - https://github.com/kriszyp/lmdb-js/issues/243#issuecomment-1823585586

declare module 'lmdb' {
  /* eslint-disable jsdoc/require-jsdoc */
  interface RootDatabaseOptionsWithPath {
    path?: string;
  }
  /* eslint-enable jsdoc/require-jsdoc */
}
