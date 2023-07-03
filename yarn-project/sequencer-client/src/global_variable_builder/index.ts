import { GlobalReaderConfig } from './config.js';
import { GlobalVariableBuilder, SimpleGlobalVariableBuilder } from './global_builder.js';
import { ViemReader } from './viem-reader.js';

export { SimpleGlobalVariableBuilder } from './global_builder.js';
export { GlobalReaderConfig } from './config.js';

/**
 * Returns a new instance of the global variable builder.
 * @param config - Configuration to initialize the builder.
 * @returns A new instance of the global variable builder.
 */
export function getGlobalVariableBuilder(config: GlobalReaderConfig): GlobalVariableBuilder {
  return new SimpleGlobalVariableBuilder(new ViemReader(config));
}
