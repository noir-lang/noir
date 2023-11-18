import { type BarretenbergWasmMain } from './index.js';

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

  constructor(private wasm: BarretenbergWasmMain) {}

  copyToMemory(buffers: Uint8Array[]) {
    return buffers.map(buf => {
      if (buf.length <= this.inScratchRemaining) {
        const ptr = (this.inScratchRemaining -= buf.length);
        this.wasm.writeMemory(ptr, buf);
        return ptr;
      } else {
        const ptr = this.wasm.call('bbmalloc', buf.length);
        this.wasm.writeMemory(ptr, buf);
        this.allocs.push(ptr);
        return ptr;
      }
    });
  }

  getOutputPtrs(outLens: (number | undefined)[]) {
    return outLens.map(len => {
      // If the obj is variable length, we need a 4 byte ptr to write the serialized data address to.
      // WARNING: 4 only works with WASM as it has 32 bit memory.
      const size = len || 4;

      if (size <= this.outScratchRemaining) {
        return (this.outScratchRemaining -= size);
      } else {
        const ptr = this.wasm.call('bbmalloc', size);
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

  freeAll() {
    for (const ptr of this.allocs) {
      this.wasm.call('bbfree', ptr);
    }
  }
}
