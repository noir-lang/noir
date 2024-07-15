import { getContractClassFromArtifact } from '@aztec/circuits.js';
import {
  type FunctionArtifact,
  FunctionSelector,
  decodeFunctionSignature,
  decodeFunctionSignatureWithParameterNames,
} from '@aztec/foundation/abi';
import { sha256 } from '@aztec/foundation/crypto';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { getContractArtifact } from '../../utils/aztec.js';

export async function inspectContract(contractArtifactFile: string, debugLogger: DebugLogger, log: LogFn) {
  const contractArtifact = await getContractArtifact(contractArtifactFile, log);
  const contractFns = contractArtifact.functions.filter(f => f.name !== 'compute_note_hash_and_optionally_a_nullifier');
  if (contractFns.length === 0) {
    log(`No functions found for contract ${contractArtifact.name}`);
  }
  const contractClass = getContractClassFromArtifact(contractArtifact);
  const bytecodeLengthInFields = 1 + Math.ceil(contractClass.packedBytecode.length / 31);

  log(`Contract class details:`);
  log(`\tidentifier: ${contractClass.id.toString()}`);
  log(`\tartifact hash: ${contractClass.artifactHash.toString()}`);
  log(`\tprivate function tree root: ${contractClass.privateFunctionsRoot.toString()}`);
  log(`\tpublic bytecode commitment: ${contractClass.publicBytecodeCommitment.toString()}`);
  log(`\tpublic bytecode length: ${contractClass.packedBytecode.length} bytes (${bytecodeLengthInFields} fields)`);
  log(`\nExternal functions:`);
  contractFns.filter(f => !f.isInternal).forEach(f => logFunction(f, log));
  log(`\nInternal functions:`);
  contractFns.filter(f => f.isInternal).forEach(f => logFunction(f, log));
}

function logFunction(fn: FunctionArtifact, log: LogFn) {
  const signatureWithParameterNames = decodeFunctionSignatureWithParameterNames(fn.name, fn.parameters);
  const signature = decodeFunctionSignature(fn.name, fn.parameters);
  const selector = FunctionSelector.fromSignature(signature);
  const bytecodeSize = fn.bytecode.length;
  const bytecodeHash = sha256(fn.bytecode).toString('hex');
  log(
    `${fn.functionType} ${signatureWithParameterNames} \n\tfunction signature: ${signature}\n\tselector: ${selector}\n\tbytecode: ${bytecodeSize} bytes (sha256 ${bytecodeHash})`,
  );
}
