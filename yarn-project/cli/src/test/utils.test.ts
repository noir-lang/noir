import { AztecAddress, Fr } from '@aztec/aztec.js';
import { AztecRPC, CompleteAddress } from '@aztec/types';

import { InvalidArgumentError } from 'commander';
import { MockProxy, mock } from 'jest-mock-extended';

import { encodeArgs } from '../encoding.js';
import { getSaltFromHexString, getTxSender, stripLeadingHex } from '../utils.js';
import { mockContractAbi } from './mocks.js';

describe('CLI Utils', () => {
  let client: MockProxy<AztecRPC>;

  // test values
  const addr1 = AztecAddress.random();
  const addr2 = AztecAddress.random();
  const addr3 = AztecAddress.random();
  const fieldArray = [addr1.toString(), addr2.toString(), addr3.toString()];
  const num = 33;
  const field = Fr.random();
  const struct = {
    subField1: field.toString(),
    subField2: 'true',
  };
  beforeEach(() => {
    client = mock<AztecRPC>();
  });
  it('Gets a txSender correctly or throw error', async () => {
    // returns a parsed Aztec Address
    const aztecAddress = AztecAddress.random();
    const result = await getTxSender(client, aztecAddress.toString());
    expect(client.getRegisteredAccounts).toHaveBeenCalledTimes(0);
    expect(result).toEqual(aztecAddress);

    // returns an address found in the aztec client
    const completeAddress = await CompleteAddress.random();
    client.getRegisteredAccounts.mockResolvedValueOnce([completeAddress]);
    const resultWithoutString = await getTxSender(client);
    expect(client.getRegisteredAccounts).toHaveBeenCalled();
    expect(resultWithoutString).toEqual(completeAddress.address);

    // throws when invalid parameter passed
    const errorAddr = 'foo';
    await expect(
      (async () => {
        await getTxSender(client, errorAddr);
      })(),
    ).rejects.toThrow(`Invalid option 'from' passed: ${errorAddr}`);

    // Throws error when no string is passed & no accounts found in RPC
    client.getRegisteredAccounts.mockResolvedValueOnce([]);
    await expect(
      (async () => {
        await getTxSender(client);
      })(),
    ).rejects.toThrow('No accounts found in Aztec RPC instance.');
  });

  it('Encodes args correctly', () => {
    const args = [addr1.toString(), 'false', num.toString(), `${JSON.stringify(fieldArray)}`, JSON.stringify(struct)];
    const result = encodeArgs(args, mockContractAbi.functions[1].parameters);
    const exp = [
      addr1.toBigInt(),
      false,
      33n,
      [addr1.toBigInt(), addr2.toBigInt(), addr3.toBigInt()],
      {
        subField1: field.toBigInt(),
        subField2: true,
      },
    ];
    expect(result).toEqual(exp);
  });

  it('Errors on invalid inputs', () => {
    // invalid number of args
    const args1 = [field.toString(), 'false'];
    expect(() => encodeArgs(args1, mockContractAbi.functions[1].parameters)).toThrow('Invalid args provided');

    // invalid array length
    const invalidArray = fieldArray.concat([Fr.random().toString()]);
    const args2 = [
      addr1.toString(),
      'false',
      num.toString(),
      `${JSON.stringify(invalidArray)}`,
      JSON.stringify(struct),
    ];
    expect(() => encodeArgs(args2, mockContractAbi.functions[1].parameters)).toThrow(
      'Invalid array length passed for arrayParam. Expected 3, received 4.',
    );

    // invalid struct
    const invalidStruct = {
      subField1: Fr.random().toString(),
    };
    const args3 = [
      addr1.toString(),
      'false',
      num.toString(),
      `${JSON.stringify(fieldArray)}`,
      JSON.stringify(invalidStruct),
    ];
    expect(() => encodeArgs(args3, mockContractAbi.functions[1].parameters)).toThrow(
      'Expected field subField2 not found in struct structParam.',
    );

    // invalid bool
    const args4 = [
      addr1.toString(),
      'foo',
      num.toString(),
      `${JSON.stringify(fieldArray)}`,
      JSON.stringify(invalidStruct),
    ];
    expect(() => encodeArgs(args4, mockContractAbi.functions[1].parameters)).toThrow(
      'Invalid boolean value passed for boolParam: foo.',
    );

    // invalid field
    const args5 = ['foo', 'false', num.toString(), `${JSON.stringify(fieldArray)}`, JSON.stringify(invalidStruct)];
    expect(() => encodeArgs(args5, mockContractAbi.functions[1].parameters)).toThrow(
      'Invalid value passed for fieldParam. Could not parse foo as a field.',
    );

    // invalid int
    const args6 = [addr1.toString(), 'false', 'foo', `${JSON.stringify(fieldArray)}`, JSON.stringify(invalidStruct)];
    expect(() => encodeArgs(args6, mockContractAbi.functions[1].parameters)).toThrow(
      'Invalid value passed for integerParam. Could not parse foo as an integer.',
    );
  });

  describe('stripLeadingHex', () => {
    it.each([
      ['0x1', '1'],
      ['1', '1'],
      ['0x00', '00'],
      ['00', '00'],
    ])('removes optional leading hex', (hex, expected) => {
      expect(stripLeadingHex(hex)).toEqual(expected);
    });
  });

  describe('getSaltFromHex', () => {
    it.each([
      ['0', Fr.ZERO],
      ['0x0', Fr.ZERO],
      ['00', Fr.ZERO],
      ['1', new Fr(1)],
      ['0x1', new Fr(1)],
      ['0x01', new Fr(1)],
      ['0xa', new Fr(0xa)],
      ['fff', new Fr(0xfff)],
    ])('correctly generates salt from a hex string', (hex, expected) => {
      expect(getSaltFromHexString(hex)).toEqual(expected);
    });

    it.each(['foo', '', ' ', ' 0x1', '01foo', 'foo1', '0xfoo'])('throws an error for invalid hex strings', str => {
      expect(() => getSaltFromHexString(str)).toThrow(InvalidArgumentError);
    });
  });
});
