import { loadProverCrs, loadVerifierCrs } from '@aztec/barretenberg.js/wasm';
import { AsyncWasmWrapper, WasmModule } from '@aztec/foundation/wasm';

import isNode from 'detect-node';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const NAME = '/aztec3-circuits';

/**
 * A low-level wrapper for an instance of Aztec3 Circuits WASM.
 */
export class CircuitsWasm extends AsyncWasmWrapper {
  codePath = isNode ? join(dirname(fileURLToPath(import.meta.url)), `../../resources/${NAME}.wasm`) : `${NAME}.wasm`;

  static instance: Promise<CircuitsWasm>;

  /**
   * Get a singleton instance of the module.
   * @returns The singleton.
   */
  public static get(): Promise<CircuitsWasm> {
    if (!this.instance) this.instance = new CircuitsWasm().init();
    return this.instance;
  }

  /**
   * Create and initialize a Circuits module.
   * @deprecated Use the get method to retrieve a singleton instance.
   * @param initial - Initial memory pages.
   * @returns The module.
   */
  public static async new(initial?: number) {
    const barretenberg = new CircuitsWasm();
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
