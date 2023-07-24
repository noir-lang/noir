import { Bufferable, serializeBufferable, OutputType } from '../serialize/index.js';
import { BarretenbergWasm, BarretenbergWasmWorker } from '../barretenberg_wasm/barretenberg_wasm.js';
import { asyncMap } from '../async_map/index.js';

/**
 * Keeps track of heap allocations so they can be easily freed.
 * The WASM memory layout has 1024 bytes of unused "scratch" space at the start (addresses 0-1023).
 * We can leverage this for IO rather than making expensive bb_malloc bb_free calls.
 * Heap allocations will be created for input/output args that don't fit into the scratch space.
 * Input and output args can use the same scratch space as it's assume all input reads will be performed before any
 * output writes are performed.
 */
export class HeapAllocator {
  private allocs: number[] = [];
  private inScratchRemaining = 1024;
  private outScratchRemaining = 1024;

  constructor(private wasm: BarretenbergWasm | BarretenbergWasmWorker) {}

  async copyToMemory(bufferable: Bufferable[]) {
    return await asyncMap(bufferable.map(serializeBufferable), async buf => {
      if (buf.length <= this.inScratchRemaining) {
        const ptr = (this.inScratchRemaining -= buf.length);
        await this.wasm.writeMemory(ptr, buf);
        return ptr;
      } else {
        const ptr = await this.wasm.call('bbmalloc', buf.length);
        await this.wasm.writeMemory(ptr, buf);
        this.allocs.push(ptr);
        return ptr;
      }
    });
  }

  async getOutputPtrs(objs: OutputType[]) {
    return await asyncMap(objs, async obj => {
      // If the obj is variable length, we need a 4 byte ptr to write the serialized data address to.
      // WARNING: 4 only works with WASM as it has 32 bit memory.
      const size = obj.SIZE_IN_BYTES || 4;

      if (size <= this.outScratchRemaining) {
        return (this.outScratchRemaining -= size);
      } else {
        const ptr = await this.wasm.call('bbmalloc', size);
        this.allocs.push(ptr);
        return ptr;
      }
    });
  }

  addOutputPtr(ptr: number) {
    if (ptr >= 1024) {
      this.allocs.push(ptr);
    }
  }

  async freeAll() {
    for (const ptr of this.allocs) {
      await this.wasm.call('bbfree', ptr);
    }
  }
}
