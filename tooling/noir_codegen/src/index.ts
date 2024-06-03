import { CompiledCircuit } from '@noir-lang/types';
import { TypingsGenerator } from './utils/typings_generator.js';

export const codegen = (
  programs: [string, CompiledCircuit][],
  embedArtifact: boolean,
  useFixedLengthArrays: boolean,
): string => {
  return new TypingsGenerator(
    programs.map((program) => ({
      circuitName: program[0],
      artifact: embedArtifact ? program[1] : undefined,
      abi: structuredClone(program[1].abi), // We'll mutate the ABI types when doing typescript codegen, so we clone it to avoid mutating the artifact.
    })),
    useFixedLengthArrays,
  ).codegen();
};
