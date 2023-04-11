import { WasmWrapper } from '@aztec/foundation/wasm';

import isNode from 'detect-node';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const NAME = 'primitives';
/**
 * A low-level wrapper for an instance of the barretenberg primitives wasm.
 */
export class PrimitivesWasm extends WasmWrapper {
  codePath = isNode ? join(dirname(fileURLToPath(import.meta.url)), `../../resources/${NAME}.wasm`) : `${NAME}.wasm`;

  static instance: Promise<PrimitivesWasm>;

  /**
   * Get a singleton instance of the module.
   * @returns The singleton.
   */
  public static get(): Promise<PrimitivesWasm> {
    if (!this.instance) this.instance = new PrimitivesWasm().init();
    return this.instance;
  }

  private constructor(loggerName?: string) {
    super(loggerName);
  }
}
