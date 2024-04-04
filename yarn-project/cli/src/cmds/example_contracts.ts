import { type LogFn } from '@aztec/foundation/log';

import { getExampleContractArtifacts } from '../utils.js';

export async function exampleContracts(log: LogFn) {
  const abisList = await getExampleContractArtifacts();
  const names = Object.keys(abisList).filter(name => !name.startsWith('Avm'));
  names.forEach(name => log(name));
}
