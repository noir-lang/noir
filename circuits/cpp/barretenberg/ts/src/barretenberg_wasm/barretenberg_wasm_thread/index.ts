import { Remote } from 'comlink';
import { killSelf, threadLogger } from '../helpers/index.js';
import { BarretenbergWasmBase } from '../barretenberg_wasm_base/index.js';

export class BarretenbergWasmThread extends BarretenbergWasmBase {
  /**
   * Init as worker thread.
   */
  public async initThread(module: WebAssembly.Module, memory: WebAssembly.Memory) {
    this.logger = threadLogger() || this.logger;
    this.memory = memory;
    this.instance = await WebAssembly.instantiate(module, this.getImportObj(this.memory));
  }

  public destroy() {
    killSelf();
  }

  protected getImportObj(memory: WebAssembly.Memory) {
    const baseImports = super.getImportObj(memory);

    /* eslint-disable camelcase */
    return {
      ...baseImports,
      wasi: {
        'thread-spawn': () => {
          this.logger('PANIC: threads cannot spawn threads!');
          this.logger(new Error().stack!);
          killSelf();
        },
      },

      // These are functions implementations for imports we've defined are needed.
      // The native C++ build defines these in a module called "env". We must implement TypeScript versions here.
      env: {
        ...baseImports.env,
        env_hardware_concurrency: () => {
          // We return 1, which should cause any algos using threading to just not create a thread.
          return 1;
        },
      },
    };
    /* eslint-enable camelcase */
  }
}

export type BarretenbergWasmThreadWorker = Remote<BarretenbergWasmThread>;
