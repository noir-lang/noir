import { MemoryFifo } from './memory_fifo.js';
/**
 * Allows the acquiring of up to `size` tokens before calls to acquire block, waiting for a call to release().
 */
export class Semaphore {
    constructor(size) {
        this.queue = new MemoryFifo();
        new Array(size).fill(true).map(() => this.queue.put(true));
    }
    async acquire() {
        await this.queue.get();
    }
    release() {
        this.queue.put(true);
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoic2VtYXBob3JlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2ZpZm8vc2VtYXBob3JlLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBRSxVQUFVLEVBQUUsTUFBTSxrQkFBa0IsQ0FBQztBQUU5Qzs7R0FFRztBQUNILE1BQU0sT0FBTyxTQUFTO0lBR3BCLFlBQVksSUFBWTtRQUZQLFVBQUssR0FBRyxJQUFJLFVBQVUsRUFBVyxDQUFDO1FBR2pELElBQUksS0FBSyxDQUFDLElBQUksQ0FBQyxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsQ0FBQyxHQUFHLENBQUMsR0FBRyxFQUFFLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDLENBQUMsQ0FBQztJQUM3RCxDQUFDO0lBRU0sS0FBSyxDQUFDLE9BQU87UUFDbEIsTUFBTSxJQUFJLENBQUMsS0FBSyxDQUFDLEdBQUcsRUFBRSxDQUFDO0lBQ3pCLENBQUM7SUFFTSxPQUFPO1FBQ1osSUFBSSxDQUFDLEtBQUssQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDLENBQUM7SUFDdkIsQ0FBQztDQUNGIn0=