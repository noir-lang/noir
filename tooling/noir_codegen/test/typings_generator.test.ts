import type { CompiledCircuit } from '@noir-lang/types';
import { expect } from 'chai';
import { codegen } from '../src/index.js';

it('marks InputMap as a type-only import', () => {
  const program = {
    abi: {
      parameters: [],
      return_type: null,
      error_types: {},
    },
    bytecode: '',
  } as unknown as CompiledCircuit;

  const generated = codegen([['example', program]], false, false);

  expect(generated).to.include('import { Noir, type InputMap, type CompiledCircuit, type ForeignCallHandler }');
});
