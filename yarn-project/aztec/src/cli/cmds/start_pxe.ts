import {
  type ContractArtifact,
  type ContractInstanceWithAddress,
  Fr,
  getContractClassFromArtifact,
} from '@aztec/aztec.js';
import { type AztecNode, createAztecNodeClient } from '@aztec/circuit-types';
import { getContractArtifact } from '@aztec/cli/cli-utils';
import { type ServerList } from '@aztec/foundation/json-rpc/server';
import { type LogFn } from '@aztec/foundation/log';
import { AztecAddress, type CliPXEOptions, createPXERpcServer, createPXEService, getCliPXEOptions } from '@aztec/pxe';
import { L2BasicContractsMap, Network } from '@aztec/types/network';

import { mergeEnvVarsAndCliOptions, parseModuleOptions } from '../util.js';

const contractAddressesUrl = 'http://static.aztec.network';

export async function startPXE(options: any, signalHandlers: (() => Promise<void>)[], userLog: LogFn) {
  const services: ServerList = [];
  await addPXE(options, services, signalHandlers, userLog, {});
  return services;
}

function isValidNetwork(value: any): value is Network {
  return Object.values(Network).includes(value);
}

async function fetchBasicContractAddresses(url: string) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch basic contract addresses from ${url}`);
  }
  return response.json();
}

export async function addPXE(
  options: any,
  services: ServerList,
  signalHandlers: (() => Promise<void>)[],
  userLog: LogFn,
  deps: { node?: AztecNode } = {},
) {
  const pxeCliOptions = parseModuleOptions(options.pxe);
  const pxeConfig = mergeEnvVarsAndCliOptions<CliPXEOptions>(getCliPXEOptions(), pxeCliOptions);
  let nodeUrl;
  if (pxeCliOptions.network) {
    if (isValidNetwork(pxeCliOptions.network)) {
      if (!pxeCliOptions.apiKey) {
        userLog(`API Key is required to connect to ${pxeCliOptions.network}`);
        process.exit(1);
      }
      nodeUrl = `https://api.aztec.network/${pxeCliOptions.network}/aztec-node-1/${pxeCliOptions.apiKey}`;
    } else {
      userLog(`Network ${pxeCliOptions.network} is not supported`);
      process.exit(1);
    }
  } else {
    nodeUrl = pxeCliOptions.nodeUrl ?? process.env.AZTEC_NODE_URL;
  }
  if (!nodeUrl && !deps.node && !pxeCliOptions.network) {
    userLog('Aztec Node URL (nodeUrl | AZTEC_NODE_URL) option is required to start PXE without --node option');
    process.exit(1);
  }

  const node = deps.node ?? createAztecNodeClient(nodeUrl);
  const pxe = await createPXEService(node, pxeConfig);
  const pxeServer = createPXERpcServer(pxe);

  // register basic contracts
  if (pxeCliOptions.network) {
    userLog(`Registering basic contracts for ${pxeCliOptions.network}`);
    const basicContractsInfo = await fetchBasicContractAddresses(
      `${contractAddressesUrl}/${pxeCliOptions.network}/basic_contracts.json`,
    );
    const l2Contracts: Record<
      string,
      { name: string; address: AztecAddress; initHash: Fr; salt: Fr; artifact: ContractArtifact }
    > = {};
    for (const [key, artifactName] of Object.entries(L2BasicContractsMap[pxeCliOptions.network as Network])) {
      l2Contracts[key] = {
        name: key,
        address: AztecAddress.fromString(basicContractsInfo[key].address),
        initHash: Fr.fromString(basicContractsInfo[key].initHash),
        salt: Fr.fromString(basicContractsInfo[key].salt),
        artifact: await getContractArtifact(artifactName, userLog),
      };
    }

    Object.values(l2Contracts).forEach(async ({ name, address, artifact, initHash, salt }) => {
      const instance: ContractInstanceWithAddress = {
        version: 1,
        salt,
        initializationHash: initHash,
        address,
        deployer: AztecAddress.ZERO,
        contractClassId: getContractClassFromArtifact(artifact!).id,
        publicKeysHash: Fr.ZERO,
      };
      userLog(`Registering ${name} at ${address.toString()}`);
      await pxe.registerContract({ artifact, instance });
    });
  }

  // Add PXE to services list
  services.push({ pxe: pxeServer });

  // Add PXE stop function to signal handlers
  signalHandlers.push(pxe.stop);

  return pxe;
}
