import { ABIParameterVisibility, FunctionAbi } from './abi.js';
import { decodeFunctionSignature, decodeFunctionSignatureWithParameterNames } from './decoder.js';

describe('abi/decoder', () => {
  // Copied from yarn-project/noir-contracts/src/contracts/test_contract/target/Test.json
  const abi = {
    name: 'testCodeGen',
    parameters: [
      { name: 'aField', type: { kind: 'field' }, visibility: 'private' },
      { name: 'aBool', type: { kind: 'boolean' }, visibility: 'private' },
      { name: 'aNumber', type: { kind: 'integer', sign: 'unsigned', width: 32 }, visibility: 'private' },
      { name: 'anArray', type: { kind: 'array', length: 2, type: { kind: 'field' } }, visibility: 'private' },
      {
        name: 'aStruct',
        type: {
          kind: 'struct',
          path: 'Test::DummyNote',
          fields: [
            { name: 'amount', type: { kind: 'field' } },
            { name: 'secretHash', type: { kind: 'field' } },
          ],
        },
        visibility: 'private' as ABIParameterVisibility,
      },
      {
        name: 'aDeepStruct',
        type: {
          kind: 'struct',
          path: 'Test::DeepStruct',
          fields: [
            { name: 'aField', type: { kind: 'field' } },
            { name: 'aBool', type: { kind: 'boolean' } },
            {
              name: 'aNote',
              type: {
                kind: 'struct',
                path: 'Test::DummyNote',
                fields: [
                  { name: 'amount', type: { kind: 'field' } },
                  { name: 'secretHash', type: { kind: 'field' } },
                ],
              },
            },
            {
              name: 'manyNotes',
              type: {
                kind: 'array',
                length: 3,
                type: {
                  kind: 'struct',
                  path: 'Test::DummyNote',
                  fields: [
                    { name: 'amount', type: { kind: 'field' } },
                    { name: 'secretHash', type: { kind: 'field' } },
                  ],
                },
              },
            },
          ],
        },
        visibility: 'private' as ABIParameterVisibility,
      },
    ],
  } as Pick<FunctionAbi, 'name' | 'parameters'>;

  it('decodes function signature', () => {
    expect(decodeFunctionSignature(abi.name, abi.parameters)).toMatchInlineSnapshot(
      `"testCodeGen(Field,bool,u32,[Field;2],(Field,Field),(Field,bool,(Field,Field),[(Field,Field);3]))"`,
    );
  });

  it('decodes function signature with parameter names', () => {
    expect(decodeFunctionSignatureWithParameterNames(abi.name, abi.parameters)).toMatchInlineSnapshot(
      `"testCodeGen(aField: Field, aBool: bool, aNumber: u32, anArray: [Field;2], aStruct: (amount: Field, secretHash: Field), aDeepStruct: (aField: Field, aBool: bool, aNote: (amount: Field, secretHash: Field), manyNotes: [(amount: Field, secretHash: Field);3]))"`,
    );
  });
});
