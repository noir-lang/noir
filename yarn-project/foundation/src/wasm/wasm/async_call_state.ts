import { WasmModule } from './wasm_module.js';

/**
 * The state of an asynchronous WASM function.
 */
export interface AsyncFnState {
  /**
   * Is this a contination?
   */
  continuation: boolean;
  /**
   * A result, if one exists.
   */
  result?: any;
}

/**
 * To enable asynchronous callbacks from wasm to js, we leverage asyncify.
 * Https://kripken.github.io/blog/wasm/2019/07/16/asyncify.html.
 *
 * This class holds state and logic specific to handling async calls from wasm to js.
 * A single instance of this class is instantiated as part of BarretenbergWasm.
 * It allocates some memory for the asyncify stack data and initialises it.
 *
 * To make an async call into the wasm, just call `call` the same as in BarretenbergWasm, only it returns a promise.
 *
 * To make an async import that will be called from the wasm, wrap a function with the signature:
 *   my_func(state: AsyncFnState, ...args)
 * with a call to `wrapImportFn`.
 * The arguments are whatever the original call arguments were. The addition of AsyncFnState as the first argument
 * allows for the detection of wether the function is continuing after the the async call has completed.
 * If `state.continuation` is false, the function should start its async operation and return the promise.
 * If `state.continuation` is true, the function can get the result from `state.result` perform any finalisation,
 * and return an (optional) value to the wasm.
 */
export class AsyncCallState {
  private ASYNCIFY_DATA_SIZE = 16 * 1024;
  private asyncifyDataAddr!: number;
  private asyncPromise?: Promise<any>;
  // For debug purposes, the last running function
  private debugLastFuncName?: string;
  private wasm!: WasmModule;
  private state?: AsyncFnState;
  private callExport!: (...args: any[]) => number;

  /**
   * Initialize the call hooks with a WasmModule.
   * @param wasm - The module.
   */
  public init(wasm: WasmModule) {
    this.wasm = wasm;
    this.callExport = (name: string, ...args: any[]) => wasm.call(name, ...args);
    // Allocate memory for asyncify stack data.
    this.asyncifyDataAddr = this.callExport('bbmalloc', this.ASYNCIFY_DATA_SIZE);
    // TODO: is this view construction problematic like in WasmModule?
    const view = new Uint32Array(wasm.getRawMemory().buffer);
    // First two integers of asyncify data, are the start and end of the stack region.
    view[this.asyncifyDataAddr >> 2] = this.asyncifyDataAddr + 8;
    view[(this.asyncifyDataAddr + 4) >> 2] = this.asyncifyDataAddr + this.ASYNCIFY_DATA_SIZE;
  }

  /**
   * Log a message.
   * @param args - The message arguments.
   */
  private debug(...args: any[]) {
    this.wasm.getLogger()(...args);
  }

  /**
   * Free the data associated with async call states.
   */
  public destroy() {
    // Free call stack data.
    this.callExport('bbfree', this.asyncifyDataAddr);
  }

  /**
   * We call the wasm function, that will in turn call back into js via callImport and set this.asyncPromise and
   * enable the instrumented "record stack unwinding" code path.
   * Once the stack has unwound out of the wasm call, we enter into a loop of resolving the promise set in the call
   * to callImport, and calling back into the wasm to rewind the stack and continue execution.
   * @param name - The function name.
   * @param args - The function args.
   * @returns The function result.
   */
  public async call(name: string, ...args: any) {
    if (this.state) {
      throw new Error(
        `Can only handle one async call at a time: Can't run ${name}(${args}) because we are still running ${this.debugLastFuncName}`,
      );
    }
    this.state = { continuation: false };
    this.debugLastFuncName = name;
    try {
      let result = this.callExport(name, ...args);

      while (this.asyncPromise) {
        // Disable the instrumented "record stack unwinding" code path.
        this.callExport('asyncify_stop_unwind');
        // Wait for the async work to complete.
        this.state.result = await this.asyncPromise;
        this.state.continuation = true;
        // Enable "stack rewinding" code path.
        this.callExport('asyncify_start_rewind', this.asyncifyDataAddr);
        // Call function again to rebuild the stack, and continue where we left off.
        result = this.callExport(name, ...args);
      }
      return result;
    } finally {
      // Always cleanup - prevent interfering tests
      this.state = undefined;
    }
  }

  /**
   * Wrap a WASM import function.
   * @param fn - The function.
   * @returns A wrapped version with asyncify calls.
   */
  public wrapImportFn(fn: (state: AsyncFnState, ...args: any[]) => any) {
    return (...args: any[]) => {
      if (!this.asyncPromise) {
        // We are in the normal code path. Start the async fetch of data.
        this.asyncPromise = fn(this.state!, ...args);
        // Enable "record stack unwinding" code path and return.
        this.callExport('asyncify_start_unwind', this.asyncifyDataAddr);
      } else {
        // We are in the stack rewind code path, called once the promise is resolved.
        // Save the result data back to the wasm, disable stack rewind code paths, and return.
        this.callExport('asyncify_stop_rewind');
        const result = fn(this.state!, ...args);
        // Cleanup.
        this.asyncPromise = undefined;
        this.state = { continuation: false };
        return result;
      }
    };
  }
}
