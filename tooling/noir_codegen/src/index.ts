import { CompiledCircuit } from '@noir-lang/types';

const codegenImports = `import { InputMap, InputValue } from "@noir-lang/noirc_abi"
import { Noir } from "@noir-lang/noir_js"`;

const codegenFunction = (
  name: string,
  compiled_program: CompiledCircuit,
) => `export async function ${name}(args: InputMap): Promise<InputValue> {
  const program = new Noir(${JSON.stringify(compiled_program)});
  const { returnValue } = await program.execute(args);
  return returnValue;
}`;

export const codegen = (programs: [string, CompiledCircuit][]): string => {
  const results = [codegenImports];
  for (const [name, program] of programs) {
    results.push(codegenFunction(name, stripUnwantedFields(program)));
  }

  return results.join('\n\n');
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function stripUnwantedFields(value: any): CompiledCircuit {
  const { abi, bytecode } = value;
  return { abi, bytecode };
}
