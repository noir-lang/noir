import { MemoryFifo } from './memory_fifo.js';
/**
 * A more specialised fifo queue that enqueues functions to execute. Enqueued functions are executed in serial.
 */
export class SerialQueue {
    constructor() {
        this.queue = new MemoryFifo();
    }
    start() {
        this.runningPromise = this.queue.process(fn => fn());
    }
    length() {
        return this.queue.length();
    }
    cancel() {
        this.queue.cancel();
        return this.runningPromise;
    }
    end() {
        this.queue.end();
        return this.runningPromise;
    }
    /**
     * Enqueues fn for execution on the serial queue.
     * Returns the result of the function after execution.
     */
    put(fn) {
        return new Promise((resolve, reject) => {
            this.queue.put(async () => {
                try {
                    const res = await fn();
                    resolve(res);
                }
                catch (e) {
                    reject(e);
                }
            });
        });
    }
    // Awaiting this ensures the queue is empty before resuming.
    async syncPoint() {
        await this.put(async () => { });
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoic2VyaWFsX3F1ZXVlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2ZpZm8vc2VyaWFsX3F1ZXVlLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBRSxVQUFVLEVBQUUsTUFBTSxrQkFBa0IsQ0FBQztBQUU5Qzs7R0FFRztBQUNILE1BQU0sT0FBTyxXQUFXO0lBQXhCO1FBQ21CLFVBQUssR0FBRyxJQUFJLFVBQVUsRUFBdUIsQ0FBQztJQTBDakUsQ0FBQztJQXZDUSxLQUFLO1FBQ1YsSUFBSSxDQUFDLGNBQWMsR0FBRyxJQUFJLENBQUMsS0FBSyxDQUFDLE9BQU8sQ0FBQyxFQUFFLENBQUMsRUFBRSxDQUFDLEVBQUUsRUFBRSxDQUFDLENBQUM7SUFDdkQsQ0FBQztJQUVNLE1BQU07UUFDWCxPQUFPLElBQUksQ0FBQyxLQUFLLENBQUMsTUFBTSxFQUFFLENBQUM7SUFDN0IsQ0FBQztJQUVNLE1BQU07UUFDWCxJQUFJLENBQUMsS0FBSyxDQUFDLE1BQU0sRUFBRSxDQUFDO1FBQ3BCLE9BQU8sSUFBSSxDQUFDLGNBQWMsQ0FBQztJQUM3QixDQUFDO0lBRU0sR0FBRztRQUNSLElBQUksQ0FBQyxLQUFLLENBQUMsR0FBRyxFQUFFLENBQUM7UUFDakIsT0FBTyxJQUFJLENBQUMsY0FBYyxDQUFDO0lBQzdCLENBQUM7SUFFRDs7O09BR0c7SUFDSSxHQUFHLENBQUksRUFBb0I7UUFDaEMsT0FBTyxJQUFJLE9BQU8sQ0FBQyxDQUFDLE9BQU8sRUFBRSxNQUFNLEVBQUUsRUFBRTtZQUNyQyxJQUFJLENBQUMsS0FBSyxDQUFDLEdBQUcsQ0FBQyxLQUFLLElBQUksRUFBRTtnQkFDeEIsSUFBSTtvQkFDRixNQUFNLEdBQUcsR0FBRyxNQUFNLEVBQUUsRUFBRSxDQUFDO29CQUN2QixPQUFPLENBQUMsR0FBRyxDQUFDLENBQUM7aUJBQ2Q7Z0JBQUMsT0FBTyxDQUFDLEVBQUU7b0JBQ1YsTUFBTSxDQUFDLENBQUMsQ0FBQyxDQUFDO2lCQUNYO1lBQ0gsQ0FBQyxDQUFDLENBQUM7UUFDTCxDQUFDLENBQUMsQ0FBQztJQUNMLENBQUM7SUFFRCw0REFBNEQ7SUFDckQsS0FBSyxDQUFDLFNBQVM7UUFDcEIsTUFBTSxJQUFJLENBQUMsR0FBRyxDQUFDLEtBQUssSUFBSSxFQUFFLEdBQUUsQ0FBQyxDQUFDLENBQUM7SUFDakMsQ0FBQztDQUNGIn0=