import { type L1ReaderConfig } from '@aztec/ethereum';

import { type GlobalVariableBuilder, SimpleTestGlobalVariableBuilder } from './global_builder.js';
import { ViemReader } from './viem-reader.js';

export { SimpleTestGlobalVariableBuilder as SimpleGlobalVariableBuilder } from './global_builder.js';
export { GlobalVariableBuilder } from './global_builder.js';

/**
 * Returns a new instance of the global variable builder.
 * @param config - Configuration to initialize the builder.
 * @returns A new instance of the global variable builder.
 */
export function getGlobalVariableBuilder(config: L1ReaderConfig): GlobalVariableBuilder {
  return new SimpleTestGlobalVariableBuilder(new ViemReader(config));
}
