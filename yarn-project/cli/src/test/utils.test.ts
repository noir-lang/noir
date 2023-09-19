import { AztecAddress, Fr } from '@aztec/aztec.js';
import { AztecRPC, CompleteAddress } from '@aztec/types';

import { MockProxy, mock } from 'jest-mock-extended';

import { encodeArgs } from '../encoding.js';
import { getTxSender } from '../utils.js';
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
    expect(() => encodeArgs(args1, mockContractAbi.functions[1].parameters)).toThrow(
      'Invalid number of args provided. Expected: 5, received: 2',
    );

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
});
