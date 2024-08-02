import { FunctionSelector } from '@aztec/foundation/abi';
import { type LogFn } from '@aztec/foundation/log';

export function computeSelector(functionSignature: string, log: LogFn) {
  const selector = FunctionSelector.fromSignature(functionSignature);
  log(`${selector}`);
}
