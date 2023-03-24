const $transferable = Symbol('thread.transferable');

/**
 * A descriptor representing a payload with transferable components.
 * These components will have ownership transfered when published on an event bus.
 */
export interface TransferDescriptor<T = any> {
  /**
   * Marked as transferable.
   */
  [$transferable]: true;
  /**
   * The payload with the transferable objects.
   */
  send: T;
  /**
   * The objects to transfer.
   */
  transferables: Transferable[];
}

/**
 *
 */
function isTransferable(thing: any): thing is Transferable {
  if (!thing || typeof thing !== 'object') return false;
  // Don't check too thoroughly, since the list of transferable things in JS might grow over time
  return true;
}

/**
 *
 */
export function isTransferDescriptor(thing: any): thing is TransferDescriptor {
  return thing && typeof thing === 'object' && thing[$transferable];
}

/**
 * Mark a transferable object as such, so it will no be serialized and
 * deserialized on messaging with the main thread, but to transfer
 * ownership of it to the receiving thread.
 *
 * Only works with array buffers, message ports and few more special
 * types of objects, but it's much faster than serializing and
 * deserializing them.
 *
 * Note:
 * The transferable object cannot be accessed by this thread again
 * unless the receiving thread transfers it back again!
 *
 * @param transferable - Array buffer, message port or similar.
 * @see https://developers.google.com/web/updates/2011/12/Transferable-Objects-Lightning-Fast
 */
export function Transfer<T>(transferable: Transferable): TransferDescriptor<T>;

/**
 * Mark transferable objects within an arbitrary object or array as
 * being a transferable object. They will then not be serialized
 * and deserialized on messaging with the main thread, but ownership
 * of them will be tranferred to the receiving thread.
 *
 * Only array buffers, message ports and few more special types of
 * objects can be transferred, but it's much faster than serializing and
 * deserializing them.
 *
 * Note:
 * The transferable object cannot be accessed by this thread again
 * unless the receiving thread transfers it back again!
 *
 * @param transferable - Array buffer, message port or similar.
 * @see https://developers.google.com/web/updates/2011/12/Transferable-Objects-Lightning-Fast
 */
export function Transfer<T>(payload: T, transferables: Transferable[]): TransferDescriptor<T>;

/**
 * Create a transfer descriptor, marking these as transferable.
 * @param payload - The payload.
 * @param transferables - The transferable objects.
 * @returns The descriptor.
 */
export function Transfer<T>(payload: T, transferables?: Transferable[]): TransferDescriptor<T> {
  if (!transferables) {
    if (!isTransferable(payload)) throw Error();
    transferables = [payload];
  }

  return {
    [$transferable]: true,
    send: payload,
    transferables,
  };
}
