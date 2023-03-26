const $transferable = Symbol('thread.transferable');
function isTransferable(thing) {
    if (!thing || typeof thing !== 'object')
        return false;
    // Don't check too thoroughly, since the list of transferable things in JS might grow over time
    return true;
}
export function isTransferDescriptor(thing) {
    return thing && typeof thing === 'object' && thing[$transferable];
}
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
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidHJhbnNmZXJhYmxlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vLi4vc3JjL3RyYW5zcG9ydC9pbnRlcmZhY2UvdHJhbnNmZXJhYmxlLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE1BQU0sYUFBYSxHQUFHLE1BQU0sQ0FBQyxxQkFBcUIsQ0FBQyxDQUFDO0FBUXBELFNBQVMsY0FBYyxDQUFDLEtBQVU7SUFDaEMsSUFBSSxDQUFDLEtBQUssSUFBSSxPQUFPLEtBQUssS0FBSyxRQUFRO1FBQUUsT0FBTyxLQUFLLENBQUM7SUFDdEQsK0ZBQStGO0lBQy9GLE9BQU8sSUFBSSxDQUFDO0FBQ2QsQ0FBQztBQUVELE1BQU0sVUFBVSxvQkFBb0IsQ0FBQyxLQUFVO0lBQzdDLE9BQU8sS0FBSyxJQUFJLE9BQU8sS0FBSyxLQUFLLFFBQVEsSUFBSSxLQUFLLENBQUMsYUFBYSxDQUFDLENBQUM7QUFDcEUsQ0FBQztBQXVDRCxNQUFNLFVBQVUsUUFBUSxDQUFJLE9BQVUsRUFBRSxhQUE4QjtJQUNwRSxJQUFJLENBQUMsYUFBYSxFQUFFO1FBQ2xCLElBQUksQ0FBQyxjQUFjLENBQUMsT0FBTyxDQUFDO1lBQUUsTUFBTSxLQUFLLEVBQUUsQ0FBQztRQUM1QyxhQUFhLEdBQUcsQ0FBQyxPQUFPLENBQUMsQ0FBQztLQUMzQjtJQUVELE9BQU87UUFDTCxDQUFDLGFBQWEsQ0FBQyxFQUFFLElBQUk7UUFDckIsSUFBSSxFQUFFLE9BQU87UUFDYixhQUFhO0tBQ2QsQ0FBQztBQUNKLENBQUMifQ==