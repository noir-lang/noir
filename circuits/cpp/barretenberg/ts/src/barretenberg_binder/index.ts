import { BarretenbergWasm, BarretenbergWasmWorker } from '../barretenberg_wasm/barretenberg_wasm.js';
import { HeapAllocator } from './heap_allocator.js';
import { Bufferable, OutputType } from '../serialize/index.js';
import { asyncMap } from '../async_map/index.js';
import { HeapAllocatorSync } from './heap_allocator_sync.js';

/**
 * Calls a WASM export function, handles allocating/freeing of memory, and serializing/deserializing to types.
 *
 * Notes on function binding ABI:
 * All functions can have an arbitrary number of input and output args.
 * All arguments must be pointers.
 * Input args are determined by being const or pointer to const.
 * Output args must come after input args.
 * All input data is big-endian.
 * All output data is big-endian, except output heap alloc pointers.
 * As integer types are converted to/from big-endian form, we shouldn't have to worry about memory alignment. (SURE?)
 * All functions should return void.
 * This binding function is responsible for allocating argument memory (including output memory).
 * Variable length output args are allocated on the heap, and the resulting pointer is written to the output arg ptr,
 * hence the above statement remains true.
 * Binding will free any variable length output args that were allocated on the heap.
 */
export class BarretenbergBinder {
  constructor(public wasm: BarretenbergWasm | BarretenbergWasmWorker) {}

  async callWasmExport(funcName: string, inArgs: Bufferable[], outTypes: OutputType[]) {
    const alloc = new HeapAllocator(this.wasm);
    const inPtrs = await alloc.copyToMemory(inArgs);
    const outPtrs = await alloc.getOutputPtrs(outTypes);
    await this.wasm.call(funcName, ...inPtrs, ...outPtrs);
    const outArgs = this.deserializeOutputArgs(outTypes, outPtrs, alloc);
    await alloc.freeAll();
    return outArgs;
  }

  private deserializeOutputArgs(outTypes: OutputType[], outPtrs: number[], alloc: HeapAllocator) {
    return asyncMap(outTypes, async (t, i) => {
      if (t.SIZE_IN_BYTES) {
        const slice = await this.wasm.getMemorySlice(outPtrs[i], outPtrs[i] + t.SIZE_IN_BYTES);
        return t.fromBuffer(slice);
      }
      const slice = await this.wasm.getMemorySlice(outPtrs[i], outPtrs[i] + 4);
      const ptr = new DataView(slice.buffer, slice.byteOffset, slice.byteLength).getUint32(0, true);
      alloc.addOutputPtr(ptr);
      return t.fromBuffer(await this.wasm.getMemorySlice(ptr));
    });
  }
}

export class BarretenbergBinderSync {
  constructor(public wasm: BarretenbergWasm) {}

  callWasmExport(funcName: string, inArgs: Bufferable[], outTypes: OutputType[]) {
    const alloc = new HeapAllocatorSync(this.wasm);
    const inPtrs = alloc.copyToMemory(inArgs);
    const outPtrs = alloc.getOutputPtrs(outTypes);
    this.wasm.call(funcName, ...inPtrs, ...outPtrs);
    const outArgs = this.deserializeOutputArgs(outTypes, outPtrs, alloc);
    alloc.freeAll();
    return outArgs;
  }

  private deserializeOutputArgs(outTypes: OutputType[], outPtrs: number[], alloc: HeapAllocatorSync) {
    return outTypes.map((t, i) => {
      if (t.SIZE_IN_BYTES) {
        const slice = this.wasm.getMemorySlice(outPtrs[i], outPtrs[i] + t.SIZE_IN_BYTES);
        return t.fromBuffer(slice);
      }
      const slice = this.wasm.getMemorySlice(outPtrs[i], outPtrs[i] + 4);
      const ptr = new DataView(slice.buffer, slice.byteOffset, slice.byteLength).getUint32(0, true);
      alloc.addOutputPtr(ptr);
      return t.fromBuffer(this.wasm.getMemorySlice(ptr));
    });
  }
}
