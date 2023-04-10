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

  static instance: Promise<BarretenbergWasm>;

  /**
   * Get a singleton instance of the module.
   * @returns The singleton.
   */
  public static get(): Promise<BarretenbergWasm> {
    if (!this.instance) this.instance = new BarretenbergWasm().init();
    return this.instance;
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
