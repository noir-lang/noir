import { sleep } from '../sleep/index.js';
import { Timer } from '../timer/index.js';
export function* backoffGenerator() {
    const v = [1, 1, 1, 2, 4, 8, 16, 32, 64];
    let i = 0;
    while (true) {
        yield v[Math.min(i++, v.length - 1)];
    }
}
export async function retry(fn, name = 'Operation', backoff = backoffGenerator()) {
    while (true) {
        try {
            return await fn();
        }
        catch (err) {
            const s = backoff.next().value;
            if (s === undefined) {
                throw err;
            }
            console.log(`${name} failed. Will retry in ${s}s...`);
            console.log(err);
            await sleep(s * 1000);
            continue;
        }
    }
}
// Call `fn` repeatedly until it returns true or timeout.
// Both `interval` and `timeout` are seconds.
// Will never timeout if the value is 0.
export async function retryUntil(fn, name = '', timeout = 0, interval = 1) {
    const timer = new Timer();
    while (true) {
        if (await fn()) {
            return;
        }
        await sleep(interval * 1000);
        if (timeout && timer.s() > timeout) {
            throw new Error(name ? `Timeout awaiting ${name}` : 'Timeout');
        }
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5kZXguanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvcmV0cnkvaW5kZXgudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUFFLEtBQUssRUFBRSxNQUFNLG1CQUFtQixDQUFDO0FBQzFDLE9BQU8sRUFBRSxLQUFLLEVBQUUsTUFBTSxtQkFBbUIsQ0FBQztBQUUxQyxNQUFNLFNBQVMsQ0FBQyxDQUFDLGdCQUFnQjtJQUMvQixNQUFNLENBQUMsR0FBRyxDQUFDLENBQUMsRUFBRSxDQUFDLEVBQUUsQ0FBQyxFQUFFLENBQUMsRUFBRSxDQUFDLEVBQUUsQ0FBQyxFQUFFLEVBQUUsRUFBRSxFQUFFLEVBQUUsRUFBRSxDQUFDLENBQUM7SUFDekMsSUFBSSxDQUFDLEdBQUcsQ0FBQyxDQUFDO0lBQ1YsT0FBTyxJQUFJLEVBQUU7UUFDWCxNQUFNLENBQUMsQ0FBQyxJQUFJLENBQUMsR0FBRyxDQUFDLENBQUMsRUFBRSxFQUFFLENBQUMsQ0FBQyxNQUFNLEdBQUcsQ0FBQyxDQUFDLENBQUMsQ0FBQztLQUN0QztBQUNILENBQUM7QUFFRCxNQUFNLENBQUMsS0FBSyxVQUFVLEtBQUssQ0FBUyxFQUF5QixFQUFFLElBQUksR0FBRyxXQUFXLEVBQUUsT0FBTyxHQUFHLGdCQUFnQixFQUFFO0lBQzdHLE9BQU8sSUFBSSxFQUFFO1FBQ1gsSUFBSTtZQUNGLE9BQU8sTUFBTSxFQUFFLEVBQUUsQ0FBQztTQUNuQjtRQUFDLE9BQU8sR0FBUSxFQUFFO1lBQ2pCLE1BQU0sQ0FBQyxHQUFHLE9BQU8sQ0FBQyxJQUFJLEVBQUUsQ0FBQyxLQUFLLENBQUM7WUFDL0IsSUFBSSxDQUFDLEtBQUssU0FBUyxFQUFFO2dCQUNuQixNQUFNLEdBQUcsQ0FBQzthQUNYO1lBQ0QsT0FBTyxDQUFDLEdBQUcsQ0FBQyxHQUFHLElBQUksMEJBQTBCLENBQUMsTUFBTSxDQUFDLENBQUM7WUFDdEQsT0FBTyxDQUFDLEdBQUcsQ0FBQyxHQUFHLENBQUMsQ0FBQztZQUNqQixNQUFNLEtBQUssQ0FBQyxDQUFDLEdBQUcsSUFBSSxDQUFDLENBQUM7WUFDdEIsU0FBUztTQUNWO0tBQ0Y7QUFDSCxDQUFDO0FBRUQseURBQXlEO0FBQ3pELDZDQUE2QztBQUM3Qyx3Q0FBd0M7QUFDeEMsTUFBTSxDQUFDLEtBQUssVUFBVSxVQUFVLENBQUMsRUFBb0MsRUFBRSxJQUFJLEdBQUcsRUFBRSxFQUFFLE9BQU8sR0FBRyxDQUFDLEVBQUUsUUFBUSxHQUFHLENBQUM7SUFDekcsTUFBTSxLQUFLLEdBQUcsSUFBSSxLQUFLLEVBQUUsQ0FBQztJQUMxQixPQUFPLElBQUksRUFBRTtRQUNYLElBQUksTUFBTSxFQUFFLEVBQUUsRUFBRTtZQUNkLE9BQU87U0FDUjtRQUVELE1BQU0sS0FBSyxDQUFDLFFBQVEsR0FBRyxJQUFJLENBQUMsQ0FBQztRQUU3QixJQUFJLE9BQU8sSUFBSSxLQUFLLENBQUMsQ0FBQyxFQUFFLEdBQUcsT0FBTyxFQUFFO1lBQ2xDLE1BQU0sSUFBSSxLQUFLLENBQUMsSUFBSSxDQUFDLENBQUMsQ0FBQyxvQkFBb0IsSUFBSSxFQUFFLENBQUMsQ0FBQyxDQUFDLFNBQVMsQ0FBQyxDQUFDO1NBQ2hFO0tBQ0Y7QUFDSCxDQUFDIn0=