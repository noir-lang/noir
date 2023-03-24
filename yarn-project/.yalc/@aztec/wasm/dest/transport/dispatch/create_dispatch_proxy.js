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
/**
 * Create a proxy object of our class T that uses transportClient
 * @param class_ - Our class T.
 * @param transportClient - The transport infrastructure.
 * @returns A proxy over T.
 */
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
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiY3JlYXRlX2Rpc3BhdGNoX3Byb3h5LmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vLi4vc3JjL3RyYW5zcG9ydC9kaXNwYXRjaC9jcmVhdGVfZGlzcGF0Y2hfcHJveHkudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBRUEsT0FBTyxFQUFFLFlBQVksRUFBRSxNQUFNLFFBQVEsQ0FBQztBQUN0QyxPQUFPLEVBQUUsb0JBQW9CLEVBQXNCLE1BQU0sOEJBQThCLENBQUM7QUF3Q3hGLE1BQU0sVUFBVSx5QkFBeUIsQ0FDdkMsTUFBbUMsRUFDbkMsU0FBMkQ7SUFFM0QsTUFBTSxLQUFLLEdBQVEsTUFBTSxDQUFDLFNBQVMsWUFBWSxZQUFZLENBQUMsQ0FBQyxDQUFDLElBQUksWUFBWSxFQUFFLENBQUMsQ0FBQyxDQUFDLEVBQUUsQ0FBQztJQUN0RixLQUFLLE1BQU0sRUFBRSxJQUFJLE1BQU0sQ0FBQyxtQkFBbUIsQ0FBQyxNQUFNLENBQUMsU0FBUyxDQUFDLEVBQUU7UUFDN0QsSUFBSSxFQUFFLEtBQUssYUFBYSxFQUFFO1lBQ3hCLFNBQVM7U0FDVjtRQUNELEtBQUssQ0FBQyxFQUFFLENBQUMsR0FBRyxTQUFTLENBQUMsRUFBRSxDQUFDLENBQUM7S0FDM0I7SUFDRCxPQUFPLEtBQUssQ0FBQztBQUNmLENBQUM7QUFFRDs7Ozs7R0FLRztBQUNILE1BQU0sVUFBVSxtQkFBbUIsQ0FDakMsTUFBbUMsRUFDbkMsZUFBNkM7SUFFN0MsOEVBQThFO0lBQzlFLE1BQU0sS0FBSyxHQUFHLHlCQUF5QixDQUFDLE1BQU0sRUFBRSxDQUFDLEVBQVUsRUFBRSxFQUFFLENBQUMsQ0FBQyxHQUFHLElBQVcsRUFBRSxFQUFFO1FBQ2pGLHlFQUF5RTtRQUN6RSxNQUFNLFFBQVEsR0FBbUIsSUFBSSxDQUFDLE1BQU0sQ0FDMUMsQ0FBQyxHQUFHLEVBQUUsQ0FBQyxFQUFFLEVBQUUsQ0FBQyxDQUFDLG9CQUFvQixDQUFDLENBQUMsQ0FBQyxDQUFDLENBQUMsQ0FBQyxDQUFDLEdBQUcsR0FBRyxFQUFFLEdBQUcsQ0FBQyxDQUFDLGFBQWEsQ0FBQyxDQUFDLENBQUMsQ0FBQyxHQUFHLENBQUMsRUFDMUUsRUFBb0IsQ0FDckIsQ0FBQztRQUNGLElBQUksR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBQyxFQUFFLENBQUMsQ0FBQyxvQkFBb0IsQ0FBQyxDQUFDLENBQUMsQ0FBQyxDQUFDLENBQUMsQ0FBQyxDQUFDLElBQUksQ0FBQyxDQUFDLENBQUMsQ0FBQyxDQUFDLENBQUMsQ0FBQztRQUM3RCxPQUFPLGVBQWUsQ0FBQyxPQUFPLENBQUMsRUFBRSxFQUFFLEVBQUUsSUFBSSxFQUFFLEVBQUUsUUFBUSxDQUFDLENBQUM7SUFDekQsQ0FBQyxDQUFDLENBQUM7SUFDSCxJQUFJLEtBQUssWUFBWSxZQUFZLEVBQUU7UUFDakMscUVBQXFFO1FBQ3JFLGVBQWUsQ0FBQyxFQUFFLENBQUMsV0FBVyxFQUFFLENBQUMsRUFBRSxFQUFFLEVBQUUsSUFBSSxFQUFFLEVBQUUsRUFBRTtZQUMvQyxJQUFJLEVBQUUsS0FBSyxNQUFNLEVBQUU7Z0JBQ2pCLE1BQU0sQ0FBQyxTQUFTLEVBQUUsR0FBRyxRQUFRLENBQUMsR0FBRyxJQUFJLENBQUM7Z0JBQ3RDLEtBQUssQ0FBQyxJQUFJLENBQUMsU0FBUyxFQUFFLEdBQUcsUUFBUSxDQUFDLENBQUM7YUFDcEM7UUFDSCxDQUFDLENBQUMsQ0FBQztLQUNKO0lBQ0QsT0FBTyxLQUFLLENBQUM7QUFDZixDQUFDIn0=