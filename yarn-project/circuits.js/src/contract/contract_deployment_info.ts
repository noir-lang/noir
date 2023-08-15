import {
  computeContractAddress,
  computeFunctionTreeRoot,
  computePartialAddress,
  computeVarArgsHash,
  hashConstructor,
} from '@aztec/circuits.js/abis';
import { ContractAbi, encodeArguments, generateFunctionSelector } from '@aztec/foundation/abi';

import { CircuitsWasm, CompleteAddress, DeploymentInfo, Fr, FunctionData, PublicKey } from '../index.js';
import { generateFunctionLeaves, hashVKStr, isConstructor } from './contract_tree/contract_tree.js';

/**
 * Generates the deployment info for a contract
 * @param abi - The account contract abi
 * @param args - The args to the account contract constructor
 * @param contractAddressSalt - The salt to be used in the contract address derivation
 * @param publicKey - The account public key
 * @returns - The contract deployment info
 */
export async function getContractDeploymentInfo(
  abi: ContractAbi,
  args: any[],
  contractAddressSalt: Fr,
  publicKey: PublicKey,
): Promise<DeploymentInfo> {
  const constructorAbi = abi.functions.find(isConstructor);
  if (!constructorAbi) {
    throw new Error('Cannot find constructor in the ABI.');
  }
  if (!constructorAbi.verificationKey) {
    throw new Error('Missing verification key for the constructor.');
  }

  const wasm = await CircuitsWasm.get();
  const vkHash = hashVKStr(constructorAbi.verificationKey, wasm);
  const constructorVkHash = Fr.fromBuffer(vkHash);
  const functions = abi.functions.map(f => ({
    ...f,
    selector: generateFunctionSelector(f.name, f.parameters),
  }));
  const leaves = generateFunctionLeaves(functions, wasm);
  const functionTreeRoot = computeFunctionTreeRoot(wasm, leaves);
  const functionData = FunctionData.fromAbi(constructorAbi);
  const flatArgs = encodeArguments(constructorAbi, args);
  const argsHash = await computeVarArgsHash(wasm, flatArgs);
  const constructorHash = hashConstructor(wasm, functionData, argsHash, constructorVkHash.toBuffer());

  const partialAddress = computePartialAddress(wasm, contractAddressSalt, functionTreeRoot, constructorHash);
  const contractAddress = computeContractAddress(
    wasm,
    publicKey,
    contractAddressSalt,
    functionTreeRoot,
    constructorHash,
  );

  const completeAddress = await CompleteAddress.create(contractAddress, publicKey, partialAddress);

  return {
    completeAddress,
    constructorHash: constructorVkHash,
    functionTreeRoot,
  };
}
