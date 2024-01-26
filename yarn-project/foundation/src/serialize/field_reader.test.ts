import { Fq, Fr } from '../fields/fields.js';
import { FieldReader } from './field_reader.js';

const FIELDS = [new Fr(0), new Fr(1), new Fr(23), new Fr(45), new Fr(6789)];

class Something {
  constructor(public id: Fr, public value: number) {}

  static fromFields(reader: FieldReader): Something {
    return new Something(reader.readField(), reader.readU32());
  }
}

describe('field reader', () => {
  let reader: FieldReader;

  beforeEach(() => {
    reader = new FieldReader(FIELDS);
  });

  describe('readFr', () => {
    it('should read Fr', () => {
      FIELDS.forEach(fr => {
        expect(reader.readField()).toEqual(fr);
      });

      expect(() => reader.readField()).toThrow('Not enough fields to be consumed.');
    });
  });

  describe('readFq', () => {
    it('should get Fq from buffer', () => {
      expect(reader.readFq()).toEqual(Fq.fromHighLow(new Fr(0), new Fr(1)));
      expect(reader.readFq()).toEqual(Fq.fromHighLow(new Fr(23), new Fr(45)));

      expect(() => reader.readFq()).toThrow('Not enough fields to be consumed.');
    });
  });

  describe('readBoolean', () => {
    it('should read false when 0 and true when 1, throw otherwise', () => {
      expect(reader.readBoolean()).toBe(false);
      expect(reader.readBoolean()).toBe(true);

      expect(() => reader.readBoolean()).toThrow('Field is not a boolean');
    });
  });

  describe('readU32', () => {
    it('should return number', () => {
      expect(reader.readU32()).toBe(0);
      expect(reader.readU32()).toBe(1);
      expect(reader.readU32()).toBe(23);
      expect(reader.readU32()).toBe(45);
      expect(reader.readU32()).toBe(6789);
    });

    it('should throw if reading a value larger than u32', () => {
      const reader = new FieldReader([new Fr(2n ** 32n)]);
      expect(() => reader.readU32()).toThrow('Field is not a u32.');
    });
  });

  describe('readFieldArray', () => {
    it('should read an array of fields', () => {
      expect(reader.readFieldArray(3)).toEqual([new Fr(0), new Fr(1), new Fr(23)]);
    });

    it('should throw if reading more fields than in the reader', () => {
      expect(() => reader.readFieldArray(FIELDS.length + 1)).toThrow('Not enough fields to be consumed.');
    });
  });

  describe('readArray', () => {
    it('should read array of custom type', () => {
      const things = reader.readArray(2, Something);
      expect(things).toEqual([new Something(new Fr(0), 1), new Something(new Fr(23), 45)]);
    });

    it('should throw if reading more fields than in the reader', () => {
      expect(() => reader.readArray(3, Something)).toThrow('Not enough fields to be consumed.');
    });
  });

  describe('readObject', () => {
    it('should read object from buffer', () => {
      expect(reader.readObject(Something)).toEqual(new Something(new Fr(0), 1));
      expect(reader.readObject(Something)).toEqual(new Something(new Fr(23), 45));

      expect(() => reader.readObject(Something)).toThrow('Not enough fields to be consumed.');
    });
  });
});
