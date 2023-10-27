// Shim module to force the use of the CJS build of source-resolver & noir_wasm
/**
 * Source resolver module
 */
type SourceResolver = {
  /** Sets up a function to provide file contents */
  initializeResolver: (resolver: (source_id: string) => string) => void;
};

// eslint-disable-next-line @typescript-eslint/no-var-requires
const sourceResolver: SourceResolver = require('@noir-lang/source-resolver');

export const initializeResolver = sourceResolver.initializeResolver;
