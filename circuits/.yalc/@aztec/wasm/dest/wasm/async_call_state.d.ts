import { WasmModule } from "./wasm_module.js";
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
export declare class AsyncCallState {
  private ASYNCIFY_DATA_SIZE;
  private asyncifyDataAddr;
  private asyncPromise?;
  private wasm;
  private state?;
  private callExport;
  /**
   * Initialize the call hooks with a WasmModule.
   * @param wasm - The module.
   */
  init(wasm: WasmModule): void;
  /**
   * Log a message.
   * @param args - The message arguments.
   */
  private debug;
  /**
   * Free the data associated with async call states.
   */
  destroy(): void;
  /**
   * We call the wasm function, that will in turn call back into js via callImport and set this.asyncPromise and
   * enable the instrumented "record stack unwinding" code path.
   * Once the stack has unwound out of the wasm call, we enter into a loop of resolving the promise set in the call
   * to callImport, and calling back into the wasm to rewind the stack and continue execution.
   * @param name - The function name.
   * @param args - The function args.
   * @returns The function result.
   */
  call(name: string, ...args: any): Promise<number>;
  /**
   * Wrap a WASM import function.
   * @param fn - The function.
   * @returns A wrapped version with asyncify calls.
   */
  wrapImportFn(
    fn: (state: AsyncFnState, ...args: any[]) => any
  ): (...args: any[]) => any;
}
//# sourceMappingURL=async_call_state.d.ts.map
