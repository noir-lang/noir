import {
  FunctionSelector,
  decodeFunctionSignature,
  decodeFunctionSignatureWithParameterNames,
} from '@aztec/foundation/abi';
import { DebugLogger, LogFn } from '@aztec/foundation/log';

import { getContractArtifact } from '../utils.js';

export async function inspectContract(contractArtifactFile: string, debugLogger: DebugLogger, log: LogFn) {
  const contractArtifact = await getContractArtifact(contractArtifactFile, debugLogger);
  const contractFns = contractArtifact.functions.filter(
    f => !f.isInternal && f.name !== 'compute_note_hash_and_nullifier',
  );
  if (contractFns.length === 0) {
    log(`No external functions found for contract ${contractArtifact.name}`);
  }
  for (const fn of contractFns) {
    const signatureWithParameterNames = decodeFunctionSignatureWithParameterNames(fn.name, fn.parameters);
    const signature = decodeFunctionSignature(fn.name, fn.parameters);
    const selector = FunctionSelector.fromSignature(signature);
    log(
      `${fn.functionType} ${signatureWithParameterNames} \n\tfunction signature: ${signature}\n\tselector: ${selector}`,
    );
  }
}
