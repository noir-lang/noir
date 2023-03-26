/**
 * A simple fifo queue. It can grow unbounded. It can have multiple producers and consumers.
 * Putting an item onto the queue always succeeds, unless either end() or cancel() has been called in which case
 * the item being pushed is simply discarded.
 */
export class MemoryFifo {
    constructor() {
        this.waiting = [];
        this.items = [];
        this.flushing = false;
    }
    length() {
        return this.items.length;
    }
    /**
     * Returns next item within the queue, or blocks until and item has been put into the queue.
     * If given a timeout, the promise will reject if no item is received after `timeout` seconds.
     * If the queue is flushing, `null` is returned.
     */
    get(timeout) {
        if (this.items.length) {
            return Promise.resolve(this.items.shift());
        }
        if (this.items.length === 0 && this.flushing) {
            return Promise.resolve(null);
        }
        return new Promise((resolve, reject) => {
            this.waiting.push(resolve);
            if (timeout) {
                setTimeout(() => {
                    const index = this.waiting.findIndex(r => r === resolve);
                    if (index > -1) {
                        this.waiting.splice(index, 1);
                        const err = new Error('Timeout getting item from queue.');
                        reject(err);
                    }
                }, timeout * 1000);
            }
        });
    }
    /**
     * Put an item onto back of the queue.
     */
    put(item) {
        if (this.flushing) {
            return;
        }
        else if (this.waiting.length) {
            this.waiting.shift()(item);
        }
        else {
            this.items.push(item);
        }
    }
    /**
     * Once ended, no further items are added to queue. Consumers will consume remaining items within the queue.
     * The queue is not reusable after calling `end()`.
     * Any consumers waiting for an item receive null.
     */
    end() {
        this.flushing = true;
        this.waiting.forEach(resolve => resolve(null));
    }
    /**
     * Once cancelled, all items are discarded from the queue, and no further items are added to the queue.
     * The queue is not reusable after calling `cancel()`.
     * Any consumers waiting for an item receive null.
     */
    cancel() {
        this.flushing = true;
        this.items = [];
        this.waiting.forEach(resolve => resolve(null));
    }
    /**
     * Helper method that can be used to continously consume and process items on the queue.
     */
    async process(handler) {
        try {
            while (true) {
                const item = await this.get();
                if (item === null) {
                    break;
                }
                await handler(item);
            }
        }
        catch (err) {
            console.error('Queue handler exception:', err);
        }
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibWVtb3J5X2ZpZm8uanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvZmlmby9tZW1vcnlfZmlmby50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQTs7OztHQUlHO0FBQ0gsTUFBTSxPQUFPLFVBQVU7SUFBdkI7UUFDVSxZQUFPLEdBQWlDLEVBQUUsQ0FBQztRQUMzQyxVQUFLLEdBQVEsRUFBRSxDQUFDO1FBQ2hCLGFBQVEsR0FBRyxLQUFLLENBQUM7SUFzRjNCLENBQUM7SUFwRlEsTUFBTTtRQUNYLE9BQU8sSUFBSSxDQUFDLEtBQUssQ0FBQyxNQUFNLENBQUM7SUFDM0IsQ0FBQztJQUVEOzs7O09BSUc7SUFDSSxHQUFHLENBQUMsT0FBZ0I7UUFDekIsSUFBSSxJQUFJLENBQUMsS0FBSyxDQUFDLE1BQU0sRUFBRTtZQUNyQixPQUFPLE9BQU8sQ0FBQyxPQUFPLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxLQUFLLEVBQUcsQ0FBQyxDQUFDO1NBQzdDO1FBRUQsSUFBSSxJQUFJLENBQUMsS0FBSyxDQUFDLE1BQU0sS0FBSyxDQUFDLElBQUksSUFBSSxDQUFDLFFBQVEsRUFBRTtZQUM1QyxPQUFPLE9BQU8sQ0FBQyxPQUFPLENBQUMsSUFBSSxDQUFDLENBQUM7U0FDOUI7UUFFRCxPQUFPLElBQUksT0FBTyxDQUFXLENBQUMsT0FBTyxFQUFFLE1BQU0sRUFBRSxFQUFFO1lBQy9DLElBQUksQ0FBQyxPQUFPLENBQUMsSUFBSSxDQUFDLE9BQU8sQ0FBQyxDQUFDO1lBRTNCLElBQUksT0FBTyxFQUFFO2dCQUNYLFVBQVUsQ0FBQyxHQUFHLEVBQUU7b0JBQ2QsTUFBTSxLQUFLLEdBQUcsSUFBSSxDQUFDLE9BQU8sQ0FBQyxTQUFTLENBQUMsQ0FBQyxDQUFDLEVBQUUsQ0FBQyxDQUFDLEtBQUssT0FBTyxDQUFDLENBQUM7b0JBQ3pELElBQUksS0FBSyxHQUFHLENBQUMsQ0FBQyxFQUFFO3dCQUNkLElBQUksQ0FBQyxPQUFPLENBQUMsTUFBTSxDQUFDLEtBQUssRUFBRSxDQUFDLENBQUMsQ0FBQzt3QkFDOUIsTUFBTSxHQUFHLEdBQUcsSUFBSSxLQUFLLENBQUMsa0NBQWtDLENBQUMsQ0FBQzt3QkFDMUQsTUFBTSxDQUFDLEdBQUcsQ0FBQyxDQUFDO3FCQUNiO2dCQUNILENBQUMsRUFBRSxPQUFPLEdBQUcsSUFBSSxDQUFDLENBQUM7YUFDcEI7UUFDSCxDQUFDLENBQUMsQ0FBQztJQUNMLENBQUM7SUFFRDs7T0FFRztJQUNJLEdBQUcsQ0FBQyxJQUFPO1FBQ2hCLElBQUksSUFBSSxDQUFDLFFBQVEsRUFBRTtZQUNqQixPQUFPO1NBQ1I7YUFBTSxJQUFJLElBQUksQ0FBQyxPQUFPLENBQUMsTUFBTSxFQUFFO1lBQzlCLElBQUksQ0FBQyxPQUFPLENBQUMsS0FBSyxFQUFHLENBQUMsSUFBSSxDQUFDLENBQUM7U0FDN0I7YUFBTTtZQUNMLElBQUksQ0FBQyxLQUFLLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxDQUFDO1NBQ3ZCO0lBQ0gsQ0FBQztJQUVEOzs7O09BSUc7SUFDSSxHQUFHO1FBQ1IsSUFBSSxDQUFDLFFBQVEsR0FBRyxJQUFJLENBQUM7UUFDckIsSUFBSSxDQUFDLE9BQU8sQ0FBQyxPQUFPLENBQUMsT0FBTyxDQUFDLEVBQUUsQ0FBQyxPQUFPLENBQUMsSUFBSSxDQUFDLENBQUMsQ0FBQztJQUNqRCxDQUFDO0lBRUQ7Ozs7T0FJRztJQUNJLE1BQU07UUFDWCxJQUFJLENBQUMsUUFBUSxHQUFHLElBQUksQ0FBQztRQUNyQixJQUFJLENBQUMsS0FBSyxHQUFHLEVBQUUsQ0FBQztRQUNoQixJQUFJLENBQUMsT0FBTyxDQUFDLE9BQU8sQ0FBQyxPQUFPLENBQUMsRUFBRSxDQUFDLE9BQU8sQ0FBQyxJQUFJLENBQUMsQ0FBQyxDQUFDO0lBQ2pELENBQUM7SUFFRDs7T0FFRztJQUNJLEtBQUssQ0FBQyxPQUFPLENBQUMsT0FBbUM7UUFDdEQsSUFBSTtZQUNGLE9BQU8sSUFBSSxFQUFFO2dCQUNYLE1BQU0sSUFBSSxHQUFHLE1BQU0sSUFBSSxDQUFDLEdBQUcsRUFBRSxDQUFDO2dCQUM5QixJQUFJLElBQUksS0FBSyxJQUFJLEVBQUU7b0JBQ2pCLE1BQU07aUJBQ1A7Z0JBQ0QsTUFBTSxPQUFPLENBQUMsSUFBSSxDQUFDLENBQUM7YUFDckI7U0FDRjtRQUFDLE9BQU8sR0FBRyxFQUFFO1lBQ1osT0FBTyxDQUFDLEtBQUssQ0FBQywwQkFBMEIsRUFBRSxHQUFHLENBQUMsQ0FBQztTQUNoRDtJQUNILENBQUM7Q0FDRiJ9