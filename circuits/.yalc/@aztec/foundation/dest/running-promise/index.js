export class RunningPromise {
    constructor(fn, pollingInterval = 10000) {
        this.fn = fn;
        this.pollingInterval = pollingInterval;
        this.running = false;
        this.runningPromise = Promise.resolve();
        this.interruptPromise = Promise.resolve();
        this.interruptResolve = () => { };
    }
    /**
     * Starts the running promise
     */
    start() {
        this.running = true;
        this.interruptPromise = new Promise(resolve => (this.interruptResolve = resolve));
        const poll = async () => {
            while (this.running) {
                await this.fn();
                await this.interruptableSleep(this.pollingInterval);
            }
        };
        this.runningPromise = poll();
    }
    async stop() {
        this.running = false;
        this.interruptResolve();
        await this.runningPromise;
    }
    async interruptableSleep(timeInMs) {
        let timeout;
        const sleepPromise = new Promise(resolve => {
            timeout = setTimeout(resolve, timeInMs);
        });
        await Promise.race([sleepPromise, this.interruptPromise]);
        clearTimeout(timeout);
    }
    isRunning() {
        return this.running;
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5kZXguanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvcnVubmluZy1wcm9taXNlL2luZGV4LnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE1BQU0sT0FBTyxjQUFjO0lBS3pCLFlBQW9CLEVBQXVCLEVBQVUsa0JBQWtCLEtBQUs7UUFBeEQsT0FBRSxHQUFGLEVBQUUsQ0FBcUI7UUFBVSxvQkFBZSxHQUFmLGVBQWUsQ0FBUTtRQUpwRSxZQUFPLEdBQUcsS0FBSyxDQUFDO1FBQ2hCLG1CQUFjLEdBQUcsT0FBTyxDQUFDLE9BQU8sRUFBRSxDQUFDO1FBQ25DLHFCQUFnQixHQUFHLE9BQU8sQ0FBQyxPQUFPLEVBQUUsQ0FBQztRQUNyQyxxQkFBZ0IsR0FBRyxHQUFHLEVBQUUsR0FBRSxDQUFDLENBQUM7SUFDMkMsQ0FBQztJQUVoRjs7T0FFRztJQUNJLEtBQUs7UUFDVixJQUFJLENBQUMsT0FBTyxHQUFHLElBQUksQ0FBQztRQUNwQixJQUFJLENBQUMsZ0JBQWdCLEdBQUcsSUFBSSxPQUFPLENBQUMsT0FBTyxDQUFDLEVBQUUsQ0FBQyxDQUFDLElBQUksQ0FBQyxnQkFBZ0IsR0FBRyxPQUFPLENBQUMsQ0FBQyxDQUFDO1FBRWxGLE1BQU0sSUFBSSxHQUFHLEtBQUssSUFBSSxFQUFFO1lBQ3RCLE9BQU8sSUFBSSxDQUFDLE9BQU8sRUFBRTtnQkFDbkIsTUFBTSxJQUFJLENBQUMsRUFBRSxFQUFFLENBQUM7Z0JBQ2hCLE1BQU0sSUFBSSxDQUFDLGtCQUFrQixDQUFDLElBQUksQ0FBQyxlQUFlLENBQUMsQ0FBQzthQUNyRDtRQUNILENBQUMsQ0FBQztRQUNGLElBQUksQ0FBQyxjQUFjLEdBQUcsSUFBSSxFQUFFLENBQUM7SUFDL0IsQ0FBQztJQUVELEtBQUssQ0FBQyxJQUFJO1FBQ1IsSUFBSSxDQUFDLE9BQU8sR0FBRyxLQUFLLENBQUM7UUFDckIsSUFBSSxDQUFDLGdCQUFnQixFQUFFLENBQUM7UUFDeEIsTUFBTSxJQUFJLENBQUMsY0FBYyxDQUFDO0lBQzVCLENBQUM7SUFFTyxLQUFLLENBQUMsa0JBQWtCLENBQUMsUUFBZ0I7UUFDL0MsSUFBSSxPQUF3QixDQUFDO1FBQzdCLE1BQU0sWUFBWSxHQUFHLElBQUksT0FBTyxDQUFDLE9BQU8sQ0FBQyxFQUFFO1lBQ3pDLE9BQU8sR0FBRyxVQUFVLENBQUMsT0FBTyxFQUFFLFFBQVEsQ0FBQyxDQUFDO1FBQzFDLENBQUMsQ0FBQyxDQUFDO1FBQ0gsTUFBTSxPQUFPLENBQUMsSUFBSSxDQUFDLENBQUMsWUFBWSxFQUFFLElBQUksQ0FBQyxnQkFBZ0IsQ0FBQyxDQUFDLENBQUM7UUFDMUQsWUFBWSxDQUFDLE9BQU8sQ0FBQyxDQUFDO0lBQ3hCLENBQUM7SUFFTSxTQUFTO1FBQ2QsT0FBTyxJQUFJLENBQUMsT0FBTyxDQUFDO0lBQ3RCLENBQUM7Q0FDRiJ9