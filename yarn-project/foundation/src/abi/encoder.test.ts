import { AztecAddress } from '../aztec-address/index.js';
import { Fr } from '../fields/fields.js';
import { Point } from '../fields/point.js';
import { ABIParameterVisibility, type FunctionAbi, FunctionType } from './abi.js';
import { encodeArguments } from './encoder.js';

describe('abi/encoder', () => {
  it('serializes fields as fields', () => {
    const abi: FunctionAbi = {
      name: 'constructor',
      functionType: FunctionType.SECRET,
      isInternal: false,
      isInitializer: true,
      parameters: [
        {
          name: 'owner',
          type: {
            kind: 'field',
          },
          visibility: ABIParameterVisibility.SECRET,
        },
      ],
      returnTypes: [],
    };

    const field = Fr.random();
    expect(encodeArguments(abi, [field])).toEqual([field]);
  });

  it('serializes arrays of fields', () => {
    const abi: FunctionAbi = {
      name: 'constructor',
      isInitializer: true,
      functionType: FunctionType.SECRET,
      isInternal: false,
      parameters: [
        {
          name: 'owner',
          type: {
            kind: 'array',
            length: 2,
            type: { kind: 'field' },
          },
          visibility: ABIParameterVisibility.SECRET,
        },
      ],
      returnTypes: [],
    };

    const arr = [Fr.random(), Fr.random()];
    expect(encodeArguments(abi, [arr])).toEqual(arr);
  });

  it('serializes string', () => {
    const abi: FunctionAbi = {
      name: 'constructor',
      isInitializer: true,
      functionType: FunctionType.SECRET,
      isInternal: false,
      parameters: [
        {
          name: 'owner',
          type: {
            kind: 'string',
            length: 4,
          },
          visibility: ABIParameterVisibility.SECRET,
        },
      ],
      returnTypes: [],
    };

    const str = 'abc';
    // As bigints padded with 0 for length 4. ("a" = 97, "b" = 98, "c" = 99, 0)
    const expected = [new Fr(97), new Fr(98), new Fr(99), new Fr(0)];
    expect(encodeArguments(abi, [str])).toEqual(expected);
  });

  it.each(['AztecAddress', 'EthAddress'])('accepts address instance for %s structs', (structType: string) => {
    const abi: FunctionAbi = {
      name: 'constructor',
      isInitializer: true,
      functionType: FunctionType.SECRET,
      isInternal: false,
      parameters: [
        {
          name: 'owner',
          type: {
            kind: 'struct',
            path: `types::address::${structType}`,
            fields: [
              {
                name: 'inner',
                type: { kind: 'field' },
              },
            ],
          },
          visibility: ABIParameterVisibility.SECRET,
        },
      ],
      returnTypes: [],
    };

    const address = AztecAddress.random();

    expect(encodeArguments(abi, [address])).toEqual([address.toField()]);
    expect(encodeArguments(abi, [{ address }])).toEqual([address.toField()]);
    expect(encodeArguments(abi, [{ address: address.toField() }])).toEqual([address.toField()]);

    const completeAddressLike = { address, publicKey: Point.random(), partialAddress: Fr.random() };
    expect(encodeArguments(abi, [completeAddressLike])).toEqual([address.toField()]);
  });

  it('accepts a field for a wrapped field', () => {
    const abi: FunctionAbi = {
      name: 'constructor',
      isInitializer: true,
      functionType: FunctionType.SECRET,
      isInternal: false,
      parameters: [
        {
          name: 'contract_class',
          type: {
            kind: 'struct',
            path: `types::contract_class_id::ContractClassId`,
            fields: [
              {
                name: 'inner',
                type: { kind: 'field' },
              },
            ],
          },
          visibility: ABIParameterVisibility.SECRET,
        },
      ],
      returnTypes: [],
    };

    const value = Fr.random();

    expect(encodeArguments(abi, [value])).toEqual([value]);
    expect(encodeArguments(abi, [{ inner: value }])).toEqual([value]);
  });

  it('throws when passing string argument as field', () => {
    const testFunctionAbi: FunctionAbi = {
      name: 'constructor',
      isInitializer: true,
      functionType: FunctionType.SECRET,
      isInternal: false,
      parameters: [
        {
          name: 'owner',
          type: {
            kind: 'field',
          },
          visibility: ABIParameterVisibility.SECRET,
        },
      ],
      returnTypes: [],
    };
    const args = ['garbage'];

    expect(() => encodeArguments(testFunctionAbi, args)).toThrow('Invalid argument "garbage" of type field');
  });

  it('throws when passing string argument as integer', () => {
    const testFunctionAbi: FunctionAbi = {
      name: 'constructor',
      isInitializer: true,
      functionType: FunctionType.SECRET,
      isInternal: false,
      parameters: [
        {
          name: 'isOwner',
          type: {
            sign: 'value',
            width: 5,
            kind: 'integer',
          },
          visibility: ABIParameterVisibility.SECRET,
        },
      ],
      returnTypes: [],
    };
    const args = ['garbage'];
    expect(() => encodeArguments(testFunctionAbi, args)).toThrow(
      `Type 'string' with value 'garbage' passed to BaseField ctor.`,
    );
  });

  it('throws when passing object argument as field', () => {
    const testFunctionAbi: FunctionAbi = {
      name: 'constructor',
      isInitializer: true,
      functionType: FunctionType.SECRET,
      isInternal: false,
      parameters: [
        {
          name: 'owner',
          type: {
            kind: 'field',
          },
          visibility: ABIParameterVisibility.SECRET,
        },
      ],
      returnTypes: [],
    };
    const args = [
      {
        value: 'garbage',
      },
    ];

    expect(() => encodeArguments(testFunctionAbi, args)).toThrow('Argument for owner cannot be serialized to a field');
  });
});
