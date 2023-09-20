import { AztecAddress } from '../aztec-address/index.js';
import { Fr } from '../fields/fields.js';
import { ABIParameterVisibility, FunctionAbi, FunctionAbiHeader, FunctionType } from './abi.js';
import { encodeArguments } from './encoder.js';

describe('abi/encoder', () => {
  it('serializes fields as fields', () => {
    const abi: FunctionAbiHeader = {
      name: 'constructor',
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

    const field = Fr.random();
    expect(encodeArguments(abi, [field])).toEqual([field]);
  });

  it.each(['AztecAddress', 'EthereumAddress'])('accepts address instance for %s structs', (structType: string) => {
    const abi: FunctionAbiHeader = {
      name: 'constructor',
      functionType: FunctionType.SECRET,
      isInternal: false,
      parameters: [
        {
          name: 'owner',
          type: {
            kind: 'struct',
            path: `aztec::types::address::${structType}`,
            fields: [
              {
                name: 'address',
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
  });

  it('throws when passing string argument as field', () => {
    const testFunctionAbi: FunctionAbi = {
      name: 'constructor',
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
      bytecode: '',
      verificationKey: '',
    };
    const args = ['garbage'];

    expect(() => encodeArguments(testFunctionAbi, args)).toThrowError('Invalid argument "garbage" of type field');
  });

  it('throws when passing string argument as integer', () => {
    const testFunctionAbi: FunctionAbi = {
      name: 'constructor',
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
      bytecode: '',
      verificationKey: '',
    };
    const args = ['garbage'];
    expect(() => encodeArguments(testFunctionAbi, args)).toThrowError('Cannot convert garbage to a BigInt');
  });

  it('throws when passing object argument as field', () => {
    const testFunctionAbi: FunctionAbi = {
      name: 'constructor',
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
      bytecode: '',
      verificationKey: '',
    };
    const args = [
      {
        value: 'garbage',
      },
    ];

    expect(() => encodeArguments(testFunctionAbi, args)).toThrowError(
      'Argument for owner cannot be serialised to a field',
    );
  });
});
