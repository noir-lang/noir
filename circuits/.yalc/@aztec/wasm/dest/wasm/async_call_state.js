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
    constructor() {
        this.ASYNCIFY_DATA_SIZE = 16 * 1024;
    }
    /**
     * Initialize the call hooks with a WasmModule.
     * @param wasm - The module.
     */
    init(wasm) {
        this.wasm = wasm;
        this.callExport = (name, ...args) => wasm.call(name, ...args);
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
    debug(...args) {
        this.wasm.getLogger()(...args);
    }
    /**
     * Free the data associated with async call states.
     */
    destroy() {
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
    async call(name, ...args) {
        if (this.state) {
            throw new Error(`Can only handle one async call at a time: ${name}(${args})`);
        }
        this.state = { continuation: false };
        let result = this.callExport(name, ...args);
        while (this.asyncPromise) {
            // Disable the instrumented "record stack unwinding" code path.
            this.callExport('asyncify_stop_unwind');
            this.debug('stack unwound.');
            // Wait for the async work to complete.
            this.state.result = await this.asyncPromise;
            this.state.continuation = true;
            this.debug('result set starting rewind.');
            // Enable "stack rewinding" code path.
            this.callExport('asyncify_start_rewind', this.asyncifyDataAddr);
            // Call function again to rebuild the stack, and continue where we left off.
            result = this.callExport(name, ...args);
        }
        // Cleanup
        this.state = undefined;
        return result;
    }
    /**
     * Wrap a WASM import function.
     * @param fn - The function.
     * @returns A wrapped version with asyncify calls.
     */
    wrapImportFn(fn) {
        return (...args) => {
            if (!this.asyncPromise) {
                // We are in the normal code path. Start the async fetch of data.
                this.asyncPromise = fn(this.state, ...args);
                // Enable "record stack unwinding" code path and return.
                this.callExport('asyncify_start_unwind', this.asyncifyDataAddr);
            }
            else {
                // We are in the stack rewind code path, called once the promise is resolved.
                // Save the result data back to the wasm, disable stack rewind code paths, and return.
                this.callExport('asyncify_stop_rewind');
                const result = fn(this.state, ...args);
                // Cleanup.
                this.asyncPromise = undefined;
                this.state = { continuation: false };
                return result;
            }
        };
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiYXN5bmNfY2FsbF9zdGF0ZS5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy93YXNtL2FzeW5jX2NhbGxfc3RhdGUudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBZ0JBOzs7Ozs7Ozs7Ozs7Ozs7Ozs7R0FrQkc7QUFDSCxNQUFNLE9BQU8sY0FBYztJQUEzQjtRQUNVLHVCQUFrQixHQUFHLEVBQUUsR0FBRyxJQUFJLENBQUM7SUFtR3pDLENBQUM7SUE1RkM7OztPQUdHO0lBQ0ksSUFBSSxDQUFDLElBQWdCO1FBQzFCLElBQUksQ0FBQyxJQUFJLEdBQUcsSUFBSSxDQUFDO1FBQ2pCLElBQUksQ0FBQyxVQUFVLEdBQUcsQ0FBQyxJQUFZLEVBQUUsR0FBRyxJQUFXLEVBQUUsRUFBRSxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsSUFBSSxFQUFFLEdBQUcsSUFBSSxDQUFDLENBQUM7UUFDN0UsMkNBQTJDO1FBQzNDLElBQUksQ0FBQyxnQkFBZ0IsR0FBRyxJQUFJLENBQUMsVUFBVSxDQUFDLFVBQVUsRUFBRSxJQUFJLENBQUMsa0JBQWtCLENBQUMsQ0FBQztRQUM3RSxrRUFBa0U7UUFDbEUsTUFBTSxJQUFJLEdBQUcsSUFBSSxXQUFXLENBQUMsSUFBSSxDQUFDLFlBQVksRUFBRSxDQUFDLE1BQU0sQ0FBQyxDQUFDO1FBQ3pELGtGQUFrRjtRQUNsRixJQUFJLENBQUMsSUFBSSxDQUFDLGdCQUFnQixJQUFJLENBQUMsQ0FBQyxHQUFHLElBQUksQ0FBQyxnQkFBZ0IsR0FBRyxDQUFDLENBQUM7UUFDN0QsSUFBSSxDQUFDLENBQUMsSUFBSSxDQUFDLGdCQUFnQixHQUFHLENBQUMsQ0FBQyxJQUFJLENBQUMsQ0FBQyxHQUFHLElBQUksQ0FBQyxnQkFBZ0IsR0FBRyxJQUFJLENBQUMsa0JBQWtCLENBQUM7SUFDM0YsQ0FBQztJQUVEOzs7T0FHRztJQUNLLEtBQUssQ0FBQyxHQUFHLElBQVc7UUFDMUIsSUFBSSxDQUFDLElBQUksQ0FBQyxTQUFT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