import * as buffer from 'buffer-esm';
const Buffer = buffer.BufferShim;

// Fix slice method - Uint8Array.slice uses Symbol.species which breaks with BufferShim's constructor
// We need to override slice to return a proper BufferShim instance
Buffer.prototype.slice = function (start, end) {
  // Use Uint8Array.prototype.slice to get the sliced data, then create a new BufferShim from it
  const sliced = Uint8Array.prototype.slice.call(this, start, end);
  return new Buffer(sliced.buffer);
};

// Override Symbol.species to return Uint8Array for slice operations
// This prevents the constructor issue when slice tries to create a new BufferShim
Object.defineProperty(Buffer, Symbol.species, {
  get: function () {
    return Uint8Array;
  },
});

// bb.js requires `allocUnsafeSlow` which is not present in buffer-esm
if (!Buffer.allocUnsafeSlow) {
  Buffer.allocUnsafeSlow = Buffer.allocUnsafe;
}

// bb.js requires writeBigUInt64BE and readBigUInt64BE which are not present in buffer-esm
// so we are adding custom implementations

if (!Buffer.prototype.writeBigUInt64BE) {
  Buffer.prototype.writeBigUInt64BE = function (value, offset = 0) {
    if (typeof value !== 'bigint') {
      throw new TypeError('The "value" argument must be of type bigint');
    }
    if (offset < 0 || offset + 8 > this.length) {
      throw new RangeError('Index out of range');
    }

    // Split the bigint into high and low 32-bit parts
    const high = Number(value >> 32n);
    const low = Number(value & 0xffffffffn);

    this[offset] = (high >>> 24) & 0xff;
    this[offset + 1] = (high >>> 16) & 0xff;
    this[offset + 2] = (high >>> 8) & 0xff;
    this[offset + 3] = high & 0xff;

    this[offset + 4] = (low >>> 24) & 0xff;
    this[offset + 5] = (low >>> 16) & 0xff;
    this[offset + 6] = (low >>> 8) & 0xff;
    this[offset + 7] = low & 0xff;

    return offset + 8;
  };
}

if (!Buffer.prototype.readBigUInt64BE) {
  Buffer.prototype.readBigUInt64BE = function (offset = 0) {
    if (offset < 0 || offset + 8 > this.length) {
      throw new RangeError('Index out of range');
    }

    const high =
      (this[offset] * 0x1000000 + // << 24 but safe in JS
        (this[offset + 1] << 16)) |
      (this[offset + 2] << 8) |
      this[offset + 3];

    const low = (this[offset + 4] * 0x1000000 + (this[offset + 5] << 16)) | (this[offset + 6] << 8) | this[offset + 7];

    return (BigInt(high >>> 0) << 32n) + BigInt(low >>> 0);
  };
}

// Set Buffer as globalThis (window)
if (!globalThis.Buffer) globalThis.Buffer = buffer.BufferShim;

export { Buffer };
export default Buffer;
