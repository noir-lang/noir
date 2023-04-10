import { AsyncWasmWrapper, WasmModule } from '@aztec/foundation/wasm';

import isNode from 'detect-node';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';
import { loadProverCrs, loadVerifierCrs } from './load_crs.js';

const NAME = 'barretenberg';

/**
 * A low-level wrapper for an instance of Barretenberg WASM.
 */
export class BarretenbergWasm extends AsyncWasmWrapper {
  codePath = isNode ? join(dirname(fileURLToPath(import.meta.url)), `${NAME}.wasm`) : `${NAME}.wasm`;

  /**
   * Create and initialize a BarretenbergWasm module.
   * @param initial - Initial memory pages.
   * @returns The module.
   */
  public static async new(initial?: number) {
    const barretenberg = new BarretenbergWasm();
    await barretenberg.init(initial);
    return barretenberg;
  }

  constructor(loggerName?: string) {
    super(loggerName);
  }

  protected getImportFns(wasm: WasmModule) {
    return {
      ...super.getImportFns(wasm),

      // eslint-disable-next-line camelcase
      env_load_verifier_crs: this.wrapAsyncImportFn(async () => {
        return await loadVerifierCrs(wasm);
      }),
      // eslint-disable-next-line camelcase
      env_load_prover_crs: this.wrapAsyncImportFn(async (numPoints: number) => {
        return await loadProverCrs(wasm, numPoints);
      }),
    };
  }
}
