const $transferable = Symbol('thread.transferable');

/**
 * Represents a descriptor for transferable objects in multi-threaded environments.
 * Provides a structure for marking certain objects as transferable and managing the ownership transfer
 * between threads, particularly useful when working with Web Workers.
 */
export interface TransferDescriptor<T = any> {
  /**
   * A unique symbol indicating that an object is a TransferDescriptor.
   */
  [$transferable]: true;
  /**
   * The transferable data to be sent between threads.
   */
  send: T;
  /**
   * An array of objects that can be transferred between threads without serialization and deserialization.
   */
  transferables: Transferable[];
}

/**
 * Determines if the provided object is transferable.
 * Transferable objects are instances of a certain set of classes,
 * such as ArrayBuffer or MessagePort, which can be transferred between
 * different execution contexts (e.g., workers) without incurring the
 * overhead of serialization and deserialization.
 *
 * This function checks for the basic transferable criteria, but does not
 * perform an exhaustive check for all possible transferable types. As new
 * transferable types are added to JavaScript, they may be supported without
 * needing to modify this function.
 *
 * @param thing - The object to check for transferability.
 * @returns A boolean indicating whether the object is transferable.
 */
function isTransferable(thing: any): thing is Transferable {
  if (!thing || typeof thing !== 'object') {
    return false;
  }
  // Don't check too thoroughly, since the list of transferable things in JS might grow over time
  return true;
}

/**
 * Determines whether a given object is a TransferDescriptor.
 * A TransferDescriptor is an object with a [$transferable] property set to true and used for
 * transferring ownership of transferable objects between threads.
 * This function checks if the input object has the required properties to be considered
 * a valid TransferDescriptor.
 *
 * @param thing - The object to be checked for being a TransferDescriptor.
 * @returns True if the object is a TransferDescriptor, false otherwise.
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
 * of them will be transferred to the receiving thread.
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
 * Create a TransferDescriptor for transferable objects within an arbitrary object or array, allowing
 * them to be transferred between threads instead of being serialized and deserialized.
 * This method is particularly useful when working with Web Workers and other multi-threaded environments.
 * Transferable objects include ArrayBuffers, MessagePorts, and a few other special types.
 * Note that after transferring, the original thread will lose access to the transferred object unless
 * it's transferred back again.
 *
 * @param payload - The transferable object or an object containing transferable properties.
 * @param transferables - Optional array of Transferable objects found in the payload. If not provided,
 *                        the payload itself should be a Transferable object.
 * @returns A TransferDescriptor<T> containing the payload and transferables, marked as transferable.
 * @throws Error if payload is not transferable and transferables array is not provided.
 * @see https://developers.google.com/web/updates/2011/12/Transferable-Objects-Lightning-Fast
 */
export function Transfer<T>(payload: T, transferables?: Transferable[]): TransferDescriptor<T> {
  if (!transferables) {
    if (!isTransferable(payload)) {
      throw Error();
    }
    transferables = [payload];
  }

  return {
    [$transferable]: true,
    send: payload,
    transferables,
  };
}
