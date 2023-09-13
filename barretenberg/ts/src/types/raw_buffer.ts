// Used when the data is to be sent exactly as is. i.e. no length prefix will be added.
// This is useful for sending structured data that can be parsed-as-you-go, as opposed to just an array of bytes.
export class RawBuffer extends Uint8Array {}
