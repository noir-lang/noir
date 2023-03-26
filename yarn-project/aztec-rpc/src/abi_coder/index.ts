import { keccak256 } from '../foundation.js';
import { ABIParameter } from '../noir.js';

export function generateFunctionSignature(name: string, parameters: ABIParameter[]) {
  return name === 'constructor' ? name : `${name}(${parameters.map(p => p.type.kind).join(',')})`;
}

export function generateFunctionSelector(name: string, parameters: ABIParameter[]) {
  const signature = generateFunctionSignature(name, parameters);
  return keccak256(Buffer.from(signature)).slice(0, 4);
}
