const $transferable = Symbol('thread.transferable');
/**
 *
 */
function isTransferable(thing) {
    if (!thing || typeof thing !== 'object')
        return false;
    // Don't check too thoroughly, since the list of transferable things in JS might grow over time
    return true;
}
/**
 *
 */
export function isTransferDescriptor(thing) {
    return thing && typeof thing === 'object' && thing[$transferable];
}
/**
 * Create a transfer descriptor, marking these as transferable.
 * @param payload - The payload.
 * @param transferables - The transferable objects.
 * @returns The descriptor.
 */
export function Transfer(payload, transferables) {
    if (!transferables) {
        if (!isTransferable(payload))
            throw Error();
        transferables = [payload];
    }
    return {
        [$transferable]: true,
        send: payload,
        transferables,
    };
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidHJhbnNmZXJhYmxlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vLi4vc3JjL3RyYW5zcG9ydC9pbnRlcmZhY2UvdHJhbnNmZXJhYmxlLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE1BQU0sYUFBYSxHQUFHLE1BQU0sQ0FBQyxxQkFBcUIsQ0FBQyxDQUFDO0FBcUJwRDs7R0FFRztBQUNILFNBQVMsY0FBYyxDQUFDLEtBQVU7SUFDaEMsSUFBSSxDQUFDLEtBQUssSUFBSSxPQUFPLEtBQUssS0FBSyxRQUFRO1FBQUUsT0FBTyxLQUFLLENBQUM7SUFDdEQsK0ZBQStGO0lBQy9GLE9BQU8sSUFBSSxDQUFDO0FBQ2QsQ0FBQztBQUVEOztHQUVHO0FBQ0gsTUFBTSxVQUFVLG9CQUFvQixDQUFDLEtBQVU7SUFDN0MsT0FBTyxLQUFLLElBQUksT0FBTyxLQUFLLEtBQUssUUFBUSxJQUFJLEtBQUssQ0FBQyxhQUFhLENBQUMsQ0FBQztBQUNwRSxDQUFDO0FBdUNEOzs7OztHQUtHO0FBQ0gsTUFBTSxVQUFVLFFBQVEsQ0FBSSxPQUFVLEVBQUUsYUFBOEI7SUFDcEUsSUFBSSxDQUFDLGFBQWEsRUFBRTtRQUNsQixJQUFJLENBQUMsY0FBYyxDQUFDLE9BQU8sQ0FBQztZQUFFLE1BQU0sS0FBSyxFQUFFLENBQUM7UUFDNUMsYUFBYSxHQUFHLENBQUMsT0FBTyxDQUFDLENBQUM7S0FDM0I7SUFFRCxPQUFPO1FBQ0wsQ0FBQyxhQUFhLENBQUMsRUFBRSxJQUFJO1FBQ3JCLElBQUksRUFBRSxPQUFPO1FBQ2IsYUFBYTtLQUNkLENBQUM7QUFDSixDQUFDIn0=