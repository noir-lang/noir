import { TypedData } from './typed_data.js';
import { keccak256 } from '../crypto/index.js';
import { abiCoder } from '../contract/index.js';

const EIP_191_PREFIX = Buffer.from('1901', 'hex');

/**
 * Get the dependencies of a struct type. If a struct has the same dependency multiple times, it's only included once
 * in the resulting array.
 */
export const getDependencies = (typedData: TypedData, type: string, dependencies: string[] = []): string[] => {
  const TYPE_REGEX = /^\w+/;
  const match = type.match(TYPE_REGEX)!;
  const actualType = match[0];
  if (dependencies.includes(actualType)) {
    return dependencies;
  }

  if (!typedData.types[actualType]) {
    return dependencies;
  }

  return [
    actualType,
    ...typedData.types[actualType].reduce<string[]>(
      (previous, type) => [
        ...previous,
        ...getDependencies(typedData, type.type, previous).filter(dependency => !previous.includes(dependency)),
      ],
      [],
    ),
  ];
};

/**
 * Encode a type to a string. All dependant types are alphabetically sorted.
 *
 * @param typedData -.
 * @param type -.
 */
export const encodeType = (typedData: TypedData, type: string): string => {
  const [primary, ...dependencies] = getDependencies(typedData, type);
  const types = [primary, ...dependencies.sort()];

  return types
    .map(dependency => {
      return `${dependency}(${typedData.types[dependency].map(type => `${type.type} ${type.name}`)})`;
    })
    .join('');
};

/**
 * Get a type string as hash.
 */
export const getTypeHash = (typedData: TypedData, type: string) => {
  return keccak256(Buffer.from(encodeType(typedData, type)));
};

/**
 * Encodes a single value to an ABI serialisable string, number or Buffer. Returns the data as tuple, which consists of
 * an array of ABI compatible types, and an array of corresponding values.
 */
const encodeValue = (typedData: TypedData, type: string, data: unknown): [string, Buffer | string] => {
  // Checks for array types
  const ARRAY_REGEX = /^(.*)\[([0-9]*?)]$/;
  const match = type.match(ARRAY_REGEX);
  if (match) {
    const arrayType = match[1];
    const length = Number(match[2]) || undefined;

    if (!Array.isArray(data)) {
      throw new Error('Cannot encode data: value is not of array type');
    }

    if (length && data.length !== length) {
      throw new Error(`Cannot encode data: expected length of ${length}, but got ${data.length}`);
    }

    const encodedData = data.map(item => encodeValue(typedData, arrayType, item));
    const types = encodedData.map(item => item[0]);
    const values = encodedData.map(item => item[1]);

    return ['bytes32', keccak256(abiCoder.encodeParameters(types, values))];
  }

  if (typedData.types[type]) {
    return ['bytes32', getStructHash(typedData, type, data as Record<string, unknown>)];
  }

  // Strings and arbitrary byte arrays are hashed to bytes32
  if (type === 'string') {
    return ['bytes32', keccak256(Buffer.from(data as string))];
  }

  if (type === 'bytes') {
    return ['bytes32', keccak256(Buffer.isBuffer(data) ? data : Buffer.from(data as string))];
  }

  return [type, data as string];
};

/**
 * Encode the data to an ABI encoded Buffer. The data should be a key -\> value object with all the required values. All
 * dependant types are automatically encoded.
 */
export const encodeData = (typedData: TypedData, type: string, data: Record<string, unknown>): Buffer => {
  const [types, values] = typedData.types[type].reduce<[string[], unknown[]]>(
    ([types, values], field) => {
      if (data[field.name] === undefined || data[field.name] === null) {
        throw new Error(`Cannot encode data: missing data for '${field.name}'`);
      }

      const value = data[field.name];
      const [type, encodedValue] = encodeValue(typedData, field.type, value);

      return [
        [...types, type],
        [...values, encodedValue],
      ];
    },
    [['bytes32'], [getTypeHash(typedData, type)]],
  );

  return abiCoder.encodeParameters(types, values);
};

/**
 * Get encoded data as a hash. The data should be a key -\> value object with all the required values. All dependant
 * types are automatically encoded.
 */
export const getStructHash = (typedData: TypedData, type: string, data: Record<string, unknown>) => {
  return keccak256(encodeData(typedData, type, data));
};

/**
 * Get the EIP-191 encoded message to sign, from the typedData object. If `hash` is enabled, the message will be hashed
 * with Keccak256.
 */
export const getMessage = (typedData: TypedData, hash?: boolean, domain = 'EIP712Domain') => {
  const message = Buffer.concat([
    EIP_191_PREFIX,
    getStructHash(typedData, domain, typedData.domain as Record<string, unknown>),
    getStructHash(typedData, typedData.primaryType, typedData.message),
  ]);

  if (hash) {
    return keccak256(message);
  }

  return message;
};

/**
 * Get the typed data as array. This can be useful for encoding the typed data with the contract ABI.
 */
export const asArray = (
  typedData: TypedData,
  type: string = typedData.primaryType,
  data: Record<string, unknown> = typedData.message,
): unknown[] => {
  if (!typedData.types[type]) {
    throw new Error('Cannot get data as array: type does not exist');
  }

  return typedData.types[type].reduce<unknown[]>((array, { name, type }) => {
    if (typedData.types[type]) {
      if (!data[name]) {
        throw new Error(`Cannot get data as array: missing data for '${name}'`);
      }

      return [...array, asArray(typedData, type, data[name] as Record<string, unknown>)];
    }

    const value = data[name];
    return [...array, value];
  }, []);
};
