import { EventEmitter } from 'events';
import { isTransferDescriptor } from '../interface/transferable.js';
export function createDispatchProxyFromFn(class_, requestFn) {
    const proxy = class_.prototype instanceof EventEmitter ? new EventEmitter() : {};
    for (const fn of Object.getOwnPropertyNames(class_.prototype)) {
        if (fn === 'constructor') {
            continue;
        }
        proxy[fn] = requestFn(fn);
    }
    return proxy;
}
export function createDispatchProxy(class_, transportClient) {
    // Create a proxy of class_ that passes along methods over our transportClient
    const proxy = createDispatchProxyFromFn(class_, (fn) => (...args) => {
        // Pass our proxied function name and arguments over our transport client
        const transfer = args.reduce((acc, a) => (isTransferDescriptor(a) ? [...acc, ...a.transferables] : acc), []);
        args = args.map(a => (isTransferDescriptor(a) ? a.send : a));
        return transportClient.request({ fn, args }, transfer);
    });
    if (proxy instanceof EventEmitter) {
        // Handle proxied 'emit' calls if our proxy object is an EventEmitter
        transportClient.on('event_msg', ({ fn, args }) => {
            if (fn === 'emit') {
                const [eventName, ...restArgs] = args;
                proxy.emit(eventName, ...restArgs);
            }
        });
    }
    return proxy;
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiY3JlYXRlX2Rpc3BhdGNoX3Byb3h5LmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vLi4vc3JjL3RyYW5zcG9ydC9kaXNwYXRjaC9jcmVhdGVfZGlzcGF0Y2hfcHJveHkudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBRUEsT0FBTyxFQUFFLFlBQVksRUFBRSxNQUFNLFFBQVEsQ0FBQztBQUN0QyxPQUFPLEVBQUUsb0JBQW9CLEVBQXNCLE1BQU0sOEJBQThCLENBQUM7QUFxQ3hGLE1BQU0sVUFBVSx5QkFBeUIsQ0FDdkMsTUFBbUMsRUFDbkMsU0FBMkQ7SUFFM0QsTUFBTSxLQUFLLEdBQVEsTUFBTSxDQUFDLFNBQVMsWUFBWSxZQUFZLENBQUMsQ0FBQyxDQUFDLElBQUksWUFBWSxFQUFFLENBQUMsQ0FBQyxDQUFDLEVBQUUsQ0FBQztJQUN0RixLQUFLLE1BQU0sRUFBRSxJQUFJLE1BQU0sQ0FBQyxtQkFBbUIsQ0FBQyxNQUFNLENBQUMsU0FBUyxDQUFDLEVBQUU7UUFDN0QsSUFBSSxFQUFFLEtBQUssYUFBYSxFQUFFO1lBQ3hCLFNBQVM7U0FDVjtRQUNELEtBQUssQ0FBQyxFQUFFLENBQUMsR0FBRyxTQUFTLENBQUMsRUFBRSxDQUFDLENBQUM7S0FDM0I7SUFDRCxPQUFPLEtBQUssQ0FBQztBQUNmLENBQUM7QUFFRCxNQUFNLFVBQVUsbUJBQW1CLENBQ2pDLE1BQW1DLEVBQ25DLGVBQTZDO0lBRTdDLDhFQUE4RTtJQUM5RSxNQUFNLEtBQUssR0FBRyx5QkFBeUIsQ0FBQyxNQUFNLEVBQUUsQ0FBQyxFQUFVLEVBQUUsRUFBRSxDQUFDLENBQUMsR0FBRyxJQUFXLEVBQUUsRUFBRTtRQUNqRix5RUFBeUU7UUFDekUsTUFBTSxRQUFRLEdBQW1CLElBQUksQ0FBQyxNQUFNLENBQzFDLENBQUMsR0FBRyxFQUFFLENBQUMsRUFBRSxFQUFFLENBQUMsQ0FBQyxvQkFBb0IsQ0FBQyxDQUFDLENBQUMsQ0FBQyxDQUFDLENBQUMsQ0FBQyxHQUFHLEdBQUcsRUFBRSxHQUFHLENBQUMsQ0FBQyxhQUFhLENBQUMsQ0FBQyxDQUFDLENBQUMsR0FBRyxDQUFDLEVBQzFFLEVBQW9CLENBQ3JCLENBQUM7UUFDRixJQUFJLEdBQUcsSUFBSSxDQUFDLEdBQUcsQ0FBQyxDQUFDLENBQUMsRUFBRSxDQUFDLENBQUMsb0JBQW9CLENBQUMsQ0FBQyxDQUFDLENBQUMsQ0FBQyxDQUFDLENBQUMsQ0FBQyxJQUFJLENBQUMsQ0FBQyxDQUFDLENBQUMsQ0FBQyxDQUFDLENBQUM7UUFDN0QsT0FBTyxlQUFlLENBQUMsT0FBTyxDQUFDLEVBQUUsRUFBRSxFQUFFLElBQUksRUFBRSxFQUFFLFFBQVEsQ0FBQyxDQUFDO0lBQ3pELENBQUMsQ0FBQyxDQUFDO0lBQ0gsSUFBSSxLQUFLLFlBQVksWUFBWSxFQUFFO1FBQ2pDLHFFQUFxRTtRQUNyRSxlQUFlLENBQUMsRUFBRSxDQUFDLFdBQVcsRUFBRSxDQUFDLEVBQUUsRUFBRSxFQUFFLElBQUksRUFBRSxFQUFFLEVBQUU7WUFDL0MsSUFBSSxFQUFFLEtBQUssTUFBTSxFQUFFO2dCQUNqQixNQUFNLENBQUMsU0FBUyxFQUFFLEdBQUcsUUFBUSxDQUFDLEdBQUcsSUFBSSxDQUFDO2dCQUN0QyxLQUFLLENBQUMsSUFBSSxDQUFDLFNBQVMsRUFBRSxHQUFHLFFBQVEsQ0FBQyxDQUFDO2FBQ3BDO1FBQ0gsQ0FBQyxDQUFDLENBQUM7S0FDSjtJQUNELE9BQU8sS0FBSyxDQUFDO0FBQ2YsQ0FBQyJ9