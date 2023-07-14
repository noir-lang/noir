import { jest } from '@jest/globals';

import { randomBytes } from '../crypto/index.js';
import { Fq, Fr } from '../fields/fields.js';
import { BufferReader } from './buffer_reader.js';
import { serializeBufferArrayToVector } from './free_funcs.js';

const ARRAY = Array.from(Array(32)).map((_, idx) => (idx % 2 === 0 ? 0 : 1));
const BUFFER = Buffer.from(ARRAY);
const NUMBER = 65537;
const sizes = [16, 48, 32];

describe('buffer reader', () => {
  let bufferReader: BufferReader;

  beforeEach(() => {
    bufferReader = new BufferReader(BUFFER);
  });

  describe('readNumber', () => {
    it('should return number', () => {
      expect(bufferReader.readNumber()).toBe(NUMBER);
    });
  });

  describe('readBoolean', () => {
    it('should read true when 1 and false when 0', () => {
      ARRAY.forEach(element => {
        if (element !== 0) {
          expect(bufferReader.readBoolean()).toBe(true);
        } else {
          expect(bufferReader.readBoolean()).toBe(false);
        }
      });
    });
  });

  describe('readBytes', () => {
    it('should read buffer by slices', () => {
      expect(bufferReader.readBytes(2)).toEqual(Buffer.from(ARRAY.slice(0, 2)));
      expect(bufferReader.readBytes(3)).toEqual(Buffer.from(ARRAY.slice(2, 5)));
    });
  });

  describe('readFr', () => {
    it('should get Fr from buffer', () => {
      expect(bufferReader.readFr()).toEqual(Fr.fromBuffer(BUFFER));
    });
  });

  describe('readFq', () => {
    it('should get Fq from buffer', () => {
      expect(bufferReader.readFq()).toEqual(Fq.fromBuffer(BUFFER));
    });
  });

  describe('readNumberVector', () => {
    let vectorBufferReader: BufferReader;

    beforeEach(() => {
      const uintArr = [7, 13, 16];
      const uintBufArr = uintArr.map(num => {
        const uintBuf = Buffer.alloc(4);
        uintBuf.writeUInt32BE(num, 0);
        return uintBuf;
      });
      const uintArrVec = serializeBufferArrayToVector(uintBufArr);
      vectorBufferReader = new BufferReader(uintArrVec);
    });

    it('should read number vector', () => {
      expect(vectorBufferReader.readNumberVector()).toEqual([7, 13, 16]);
    });
  });

  describe('readVector', () => {
    it('should read vector and generate result array', () => {
      const fn = jest.fn();
      let i = -1;
      const result = bufferReader.readVector({
        fromBuffer: () => {
          fn();
          i++;
          return i;
        },
      });
      expect(result.length).toBe(NUMBER);
      expect(result).toEqual(Array.from(Array(NUMBER).keys()));
      expect(fn).toHaveBeenCalledTimes(NUMBER);
    });
  });

  describe('readArray', () => {
    it('should read array from buffer', () => {
      const fn = jest.fn();
      let i = -1;
      expect(
        bufferReader.readArray(10, {
          fromBuffer: () => {
            fn();
            i++;
            return i;
          },
        }),
      ).toEqual([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    });
  });

  describe('readBufferArray', () => {
    it('should read variable length array from buffer', () => {
      // Testing `readBufferArray` with a buffer that ONLY contains the data that will be read.
      // No `size` variable is passed in this case.
      const bufferArray: Buffer[] = [];
      let buf = Buffer.alloc(0);
      for (const size of sizes) {
        const sizeBuf = Buffer.alloc(4);
        sizeBuf.writeUInt32BE(size);
        const bytes = randomBytes(size);
        const ranBuf = Buffer.concat([sizeBuf, bytes]);
        bufferArray.push(bytes);
        buf = Buffer.concat([buf, ranBuf]);
      }
      const reader = BufferReader.asReader(buf);
      const res = reader.readBufferArray();
      expect(res).toEqual(bufferArray);
    });

    it('should read variable length array from buffer with other contents', () => {
      // testing `readBufferArray` with a buffer that includes some other data before and after the data that will be read.
      // The `size` variable needs to be passed in this case.
      const bufferArray: Buffer[] = [];
      const prefixBytes = randomBytes(32);
      const postfixBytes = randomBytes(16);
      let bufLen = 0;
      let buf = Buffer.alloc(32, prefixBytes);
      for (const size of sizes) {
        const sizeBuf = Buffer.alloc(4);
        sizeBuf.writeUInt32BE(size);

        const bytes = randomBytes(size);
        const ranBuf = Buffer.concat([sizeBuf, bytes]);
        buf = Buffer.concat([buf, ranBuf]);

        bufferArray.push(bytes);
        bufLen += ranBuf.length;
      }
      buf = Buffer.concat([buf, postfixBytes]);
      const reader = BufferReader.asReader(buf);
      const preRes = reader.readBytes(prefixBytes.length);
      expect(preRes).toEqual(prefixBytes);
      expect(reader.readBufferArray(bufLen)).toEqual(bufferArray);
      expect(reader.readBytes(postfixBytes.length)).toEqual(postfixBytes);
    });
  });

  describe('readObject', () => {
    it('should read object from buffer', () => {
      const fn = jest.fn();
      const object = bufferReader.readObject({
        fromBuffer: (reader: BufferReader) => {
          fn();
          return { value: 'test-string', buffer: reader };
        },
      });
      expect(object.value).toEqual('test-string');
      expect(object.buffer).toEqual(bufferReader);
      expect(fn).toHaveBeenCalledTimes(1);
    });
  });

  describe('peekBytes', () => {
    it('should return bytes from buffer', () => {
      expect(bufferReader.peekBytes(10)).toEqual(Buffer.from(ARRAY.slice(0, 10)));
    });
  });
});
