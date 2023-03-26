export * from './mutex_database.js';
export class Mutex {
    constructor(db, name, timeout = 5000, tryLockInterval = 2000, pingInterval = 2000) {
        this.db = db;
        this.name = name;
        this.timeout = timeout;
        this.tryLockInterval = tryLockInterval;
        this.pingInterval = pingInterval;
        this.id = 0;
    }
    async lock(untilAcquired = true) {
        while (true) {
            if (await this.db.acquireLock(this.name, this.timeout)) {
                const id = this.id;
                this.pingTimeout = setTimeout(() => this.ping(id), this.pingInterval);
                return true;
            }
            if (!untilAcquired) {
                return false;
            }
            await new Promise(resolve => setTimeout(resolve, this.tryLockInterval));
        }
    }
    async unlock() {
        clearTimeout(this.pingTimeout);
        this.id++;
        await this.db.releaseLock(this.name);
    }
    async ping(id) {
        if (id !== this.id) {
            return;
        }
        await this.db.extendLock(this.name, this.timeout);
        this.pingTimeout = setTimeout(() => this.ping(id), this.pingInterval);
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5kZXguanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvbXV0ZXgvaW5kZXgudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBRUEsY0FBYyxxQkFBcUIsQ0FBQztBQUVwQyxNQUFNLE9BQU8sS0FBSztJQUloQixZQUNtQixFQUFpQixFQUNqQixJQUFZLEVBQ1osVUFBVSxJQUFJLEVBQ2Qsa0JBQWtCLElBQUksRUFDdEIsZUFBZSxJQUFJO1FBSm5CLE9BQUUsR0FBRixFQUFFLENBQWU7UUFDakIsU0FBSSxHQUFKLElBQUksQ0FBUTtRQUNaLFlBQU8sR0FBUCxPQUFPLENBQU87UUFDZCxvQkFBZSxHQUFmLGVBQWUsQ0FBTztRQUN0QixpQkFBWSxHQUFaLFlBQVksQ0FBTztRQVI5QixPQUFFLEdBQUcsQ0FBQyxDQUFDO0lBU1osQ0FBQztJQUVHLEtBQUssQ0FBQyxJQUFJLENBQUMsYUFBYSxHQUFHLElBQUk7UUFDcEMsT0FBTyxJQUFJLEVBQUU7WUFDWCxJQUFJLE1BQU0sSUFBSSxDQUFDLEVBQUUsQ0FBQyxXQUFXLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSxJQUFJLENBQUMsT0FBTyxDQUFDLEVBQUU7Z0JBQ3RELE1BQU0sRUFBRSxHQUFHLElBQUksQ0FBQyxFQUFFLENBQUM7Z0JBQ25CLElBQUksQ0FBQyxXQUFXLEdBQUcsVUFBVSxDQUFDLEdBQUcsRUFBRSxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsRUFBRSxDQUFDLEVBQUUsSUFBSSxDQUFDLFlBQVksQ0FBQyxDQUFDO2dCQUN0RSxPQUFPLElBQUksQ0FBQzthQUNiO1lBRUQsSUFBSSxDQUFDLGFBQWEsRUFBRTtnQkFDbEIsT0FBTyxLQUFLLENBQUM7YUFDZDtZQUNELE1BQU0sSUFBSSxPQUFPLENBQUMsT0FBTyxDQUFDLEVBQUUsQ0FBQyxVQUFVLENBQUMsT0FBTyxFQUFFLElBQUksQ0FBQyxlQUFlLENBQUMsQ0FBQyxDQUFDO1NBQ3pFO0lBQ0gsQ0FBQztJQUVNLEtBQUssQ0FBQyxNQUFNO1FBQ2pCLFlBQVksQ0FBQyxJQUFJLENBQUMsV0FBVyxDQUFDLENBQUM7UUFDL0IsSUFBSSxDQUFDLEVBQUUsRUFBRSxDQUFDO1FBQ1YsTUFBTSxJQUFJLENBQUMsRUFBRSxDQUFDLFdBQVcsQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLENBQUM7SUFDdkMsQ0FBQztJQUVPLEtBQUssQ0FBQyxJQUFJLENBQUMsRUFBVTtRQUMzQixJQUFJLEVBQUUsS0FBSyxJQUFJLENBQUMsRUFBRSxFQUFFO1lBQ2xCLE9BQU87U0FDUjtRQUVELE1BQU0sSUFBSSxDQUFDLEVBQUUsQ0FBQyxVQUFVLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSxJQUFJLENBQUMsT0FBTyxDQUFDLENBQUM7UUFDbEQsSUFBSSxDQUFDLFdBQVcsR0FBRyxVQUFVLENBQUMsR0FBRyxFQUFFLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxFQUFFLENBQUMsRUFBRSxJQUFJLENBQUMsWUFBWSxDQUFDLENBQUM7SUFDeEUsQ0FBQztDQUNGIn0=