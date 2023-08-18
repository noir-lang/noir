import { numToUInt32BE } from '@aztec/foundation/serialize';
import { IWasmModule, WasmModule } from '@aztec/foundation/wasm';

import isNode from 'detect-node';
import { readFile } from 'fs/promises';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

import { Crs } from '../crs/index.js';

const NAME = '/aztec3-circuits';
const CODE_PATH = isNode
  ? join(dirname(fileURLToPath(import.meta.url)), `../../resources/${NAME}.wasm`)
  : `${NAME}.wasm`;
const MAX_SRS_POINTS = 50000;

/**
 * Get the text of a binary file, either locally or on the web.
 * @param path - Path to the WASM binary.
 * @returns The binary buffer.
 */
async function fetchBinary(path: string) {
  if (isNode) {
    return await readFile(path);
  } else {
    const res = await fetch(path);
    return Buffer.from(await res.arrayBuffer());
  }
}
const BB_JS_NYI_ERROR =
  'NOT YET IMPLEMENTED - needed for proofs, plan is to use barretenberg.js from NPM for proofs. See https://github.com/AztecProtocol/aztec-packages/issues/781';
/**
 * A low-level wrapper for an instance of Aztec3 Circuits WASM.
 */
export class CircuitsWasm implements IWasmModule {
  static instance: Promise<CircuitsWasm>;

  /**
   * Creates a Circuits module from a properly configured WasmModule.
   * Not meant for public use.
   *
   * @param wasm - The module configured in CircuitsWasm.new().
   */
  private constructor(private wasm: WasmModule) {}

  /**
   * Get a singleton instance of the module.
   * @returns The singleton.
   */
  public static get(): Promise<CircuitsWasm> {
    if (!this.instance) this.instance = CircuitsWasm.new();
    return this.instance;
  }

  /**
   * Create and initialize a Circuits module. Not meant for public use.
   *
   * Has 30 pages by default. 30*2**16 \> 1mb stack size plus other overheads.
   * 8192 maximum by default. 512mb.
   * @param initial - Initial memory pages.
   * @param maximum - Max memory pages.
   * @param loggerName - The logger name.
   * @returns The wrapper.
   */
  private static async new(initial = 30, maximum = 8192, loggerName = 'wasm'): Promise<CircuitsWasm> {
    const wasm = new WasmModule(
      await fetchBinary(CODE_PATH),
      module => ({
        /**
         * Log a string from wasm.
         * @param addr - The string address to log.
         */
        logstr(addr: number) {
          const rawStr = wasm.getMemoryAsString(addr);
          const m = wasm.getMemory();
          const str = `${rawStr} (mem: ${(m.length / (1024 * 1024)).toFixed(2)}MB)`;
          if (str.startsWith('abort: ') || str.startsWith('important: ')) {
            // we explicitly want to route to console.log for every abort message to not miss them:
            // eslint-disable-next-line no-console
            console.log(str);
          }
          wasm.getLogger()(str);
        },
        memory: module.getRawMemory(),
        // eslint-disable-next-line camelcase
        set_data: () => {
          wasm.getLogger()('set_data: NYI');
        },
        // eslint-disable-next-line camelcase
        get_data: () => {
          throw new Error(BB_JS_NYI_ERROR);
        },
      }),
      loggerName,
    );
    await wasm.init(initial, maximum);
    await CircuitsWasm.initializeSrs(wasm);
    return new CircuitsWasm(wasm);
  }

  /**
   * Ensures the global SRS is initialized.
   * Currently used in VK serialization and will be used in proofs.
   * TODO(AD): proof should use external bb.js
   * TODO(AD): revisit when SRS should be initialized
   * @param wasm - The WASM module.
   */
  private static async initializeSrs(wasm: WasmModule) {
    const crs = new Crs(MAX_SRS_POINTS);
    await crs.init();
    const g1Buf = wasm.call('bbmalloc', crs.getG1Data().length);
    wasm.writeMemory(g1Buf, crs.getG1Data());
    const g1SizeBuf = wasm.call('bbmalloc', 4);
    wasm.writeMemory(g1SizeBuf, numToUInt32BE(crs.numPoints));
    const g2Buf = wasm.call('bbmalloc', crs.getG2Data().length);
    wasm.writeMemory(g2Buf, crs.getG2Data());
    wasm.call('srs_init_srs', g1Buf, g1SizeBuf, g2Buf);
    wasm.call('bbfree', g1Buf);
    wasm.call('bbfree', g1SizeBuf);
    wasm.call('bbfree', g2Buf);
  }

  /**
   * Retrieve the exports object of the CircuitsWasm module.
   *
   * @returns An object containing exported functions and properties.
   */
  public exports(): any {
    return this.wasm.exports();
  }

  /**
   * Get a slice of memory between two addresses.
   * @param start - The start address.
   * @param end - The end address.
   * @returns A Uint8Array view of memory.
   */
  public getMemorySlice(start: number, end: number) {
    return this.wasm.getMemorySlice(start, end);
  }

  /**
   * Write data into the heap.
   * @param arr - The data to write.
   * @param offset - The address to write data at.
   */
  public writeMemory(offset: number, arr: Uint8Array) {
    this.wasm.writeMemory(offset, arr);
  }

  /**
   * Get memory as string.
   * @param offset - The address to get null-terminated string data from.
   * @returns JS string.
   */
  public getMemoryAsString(offset: number) {
    return this.wasm.getMemoryAsString(offset);
  }

  /**
   * Calls into the WebAssembly.
   * @param name - The method name.
   * @param args - The arguments to the method.
   * @returns The numeric integer or address result.
   */
  public call(name: string, ...args: any): number {
    return this.wasm.call(name, ...args);
  }
}
