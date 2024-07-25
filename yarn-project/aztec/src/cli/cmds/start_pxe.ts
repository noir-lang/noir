import { type AztecNode, createAztecNodeClient } from '@aztec/circuit-types';
import { type ServerList } from '@aztec/foundation/json-rpc/server';
import { type LogFn } from '@aztec/foundation/log';
import { type PXEServiceConfig, createPXERpcServer, createPXEService, getPXEServiceConfig } from '@aztec/pxe';

import { mergeEnvVarsAndCliOptions, parseModuleOptions } from '../util.js';

export async function startPXE(options: any, signalHandlers: (() => Promise<void>)[], userLog: LogFn) {
  const services: ServerList = [];
  await addPXE(options, services, signalHandlers, userLog, {});
  return services;
}

export async function addPXE(
  options: any,
  services: ServerList,
  signalHandlers: (() => Promise<void>)[],
  userLog: LogFn,
  deps: { node?: AztecNode } = {},
) {
  const pxeCliOptions = parseModuleOptions(options.pxe);
  const pxeConfig = mergeEnvVarsAndCliOptions<PXEServiceConfig>(getPXEServiceConfig(), pxeCliOptions);
  const nodeUrl = pxeCliOptions.nodeUrl ?? process.env.AZTEC_NODE_URL;
  if (!nodeUrl && !deps.node) {
    userLog('Aztec Node URL (nodeUrl | AZTEC_NODE_URL) option is required to start PXE without --node option');
    process.exit(1);
  }

  const node = deps.node ?? createAztecNodeClient(nodeUrl);
  const pxe = await createPXEService(node, pxeConfig);
  const pxeServer = createPXERpcServer(pxe);

  // Add PXE to services list
  services.push({ pxe: pxeServer });

  // Add PXE stop function to signal handlers
  signalHandlers.push(pxe.stop);

  return pxe;
}
