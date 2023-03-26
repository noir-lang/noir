import { Semaphore } from './semaphore.js';
import { SerialQueue } from './serial_queue.js';
/**
 * Leverages the unbounded SerialQueue and Semaphore to create a SerialQueue that will block when putting an item
 * if the queue size = maxQueueSize.
 */
export class BoundedSerialQueue {
    constructor(maxQueueSize) {
        this.queue = new SerialQueue();
        this.semaphore = new Semaphore(maxQueueSize);
    }
    start() {
        this.queue.start();
    }
    length() {
        return this.queue.length();
    }
    cancel() {
        return this.queue.cancel();
    }
    end() {
        return this.queue.end();
    }
    /**
     * The caller will block until fn is succesfully enqueued.
     * The fn itself is execute asyncronously and its result discarded.
     */
    async put(fn) {
        await this.semaphore.acquire();
        this.queue
            .put(async () => {
            try {
                await fn();
            }
            finally {
                this.semaphore.release();
            }
        })
            .catch(err => {
            console.error('BoundedSerialQueue handler exception:', err);
        });
    }
    /**
     * The caller will block until fn is successfully executed, and it's result returned.
     */
    async exec(fn) {
        await this.semaphore.acquire();
        return this.queue.put(async () => {
            try {
                return await fn();
            }
            finally {
                this.semaphore.release();
            }
        });
    }
    // Awaiting this ensures the queue is empty before resuming.
    async syncPoint() {
        await this.queue.syncPoint();
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiYm91bmRlZF9zZXJpYWxfcXVldWUuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvZmlmby9ib3VuZGVkX3NlcmlhbF9xdWV1ZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUUsU0FBUyxFQUFFLE1BQU0sZ0JBQWdCLENBQUM7QUFDM0MsT0FBTyxFQUFFLFdBQVcsRUFBRSxNQUFNLG1CQUFtQixDQUFDO0FBRWhEOzs7R0FHRztBQUNILE1BQU0sT0FBTyxrQkFBa0I7SUFJN0IsWUFBWSxZQUFvQjtRQUhmLFVBQUssR0FBRyxJQUFJLFdBQVcsRUFBRSxDQUFDO1FBSXpDLElBQUksQ0FBQyxTQUFTLEdBQUcsSUFBSSxTQUFTLENBQUMsWUFBWSxDQUFDLENBQUM7SUFDL0MsQ0FBQztJQUVNLEtBQUs7UUFDVixJQUFJLENBQUMsS0FBSyxDQUFDLEtBQUssRUFBRSxDQUFDO0lBQ3JCLENBQUM7SUFFTSxNQUFNO1FBQ1gsT0FBTyxJQUFJLENBQUMsS0FBSyxDQUFDLE1BQU0sRUFBRSxDQUFDO0lBQzdCLENBQUM7SUFFTSxNQUFNO1FBQ1gsT0FBTyxJQUFJLENBQUMsS0FBSyxDQUFDLE1BQU0sRUFBRSxDQUFDO0lBQzdCLENBQUM7SUFFTSxHQUFHO1FBQ1IsT0FBTyxJQUFJLENBQUMsS0FBSyxDQUFDLEdBQUcsRUFBRSxDQUFDO0lBQzFCLENBQUM7SUFFRDs7O09BR0c7SUFDSSxLQUFLLENBQUMsR0FBRyxDQUFDLEVBQXVCO1FBQ3RDLE1BQU0sSUFBSSxDQUFDLFNBQVMsQ0FBQyxPQUFPLEVBQUUsQ0FBQztRQUMvQixJQUFJLENBQUMsS0FBSzthQUNQLEdBQUcsQ0FBQyxLQUFLLElBQUksRUFBRTtZQUNkLElBQUk7Z0JBQ0YsTUFBTSxFQUFFLEVBQUUsQ0FBQzthQUNaO29CQUFTO2dCQUNSLElBQUksQ0FBQyxTQUFTLENBQUMsT0FBTyxFQUFFLENBQUM7YUFDMUI7UUFDSCxDQUFDLENBQUM7YUFDRCxLQUFLLENBQUMsR0FBRyxDQUFDLEVBQUU7WUFDWCxPQUFPLENBQUMsS0FBSyxDQUFDLHVDQUF1QyxFQUFFLEdBQUcsQ0FBQyxDQUFDO1FBQzlELENBQUMsQ0FBQyxDQUFDO0lBQ1AsQ0FBQztJQUVEOztPQUVHO0lBQ0ksS0FBSyxDQUFDLElBQUksQ0FBSSxFQUFvQjtRQUN2QyxNQUFNLElBQUksQ0FBQyxTQUFTLENBQUMsT0FBTyxFQUFFLENBQUM7UUFDL0IsT0FBTyxJQUFJLENBQUMsS0FBSyxDQUFDLEdBQUcsQ0FBQyxLQUFLLElBQUksRUFBRTtZQUMvQixJQUFJO2dCQUNGLE9BQU8sTUFBTSxFQUFFLEVBQUUsQ0FBQzthQUNuQjtvQkFBUztnQkFDUixJQUFJLENBQUMsU0FBUyxDQUFDLE9BQU8sRUFBRSxDQUFDO2FBQzFCO1FBQ0gsQ0FBQyxDQUFDLENBQUM7SUFDTCxDQUFDO0lBRUQsNERBQTREO0lBQ3JELEtBQUssQ0FBQyxTQUFTO1FBQ3BCLE1BQU0sSUFBSSxDQUFDLEtBQUssQ0FBQyxTQUFTLEVBQUUsQ0FBQztJQUMvQixDQUFDO0NBQ0YifQ==