import { createAztecNodeClient } from '@aztec/circuit-types';
import { ServerList } from '@aztec/foundation/json-rpc/server';
import { LogFn } from '@aztec/foundation/log';
import { PXEServiceConfig, createPXERpcServer, createPXEService, getPXEServiceConfig } from '@aztec/pxe';

import { mergeEnvVarsAndCliOptions, parseModuleOptions } from '../util.js';

const { AZTEC_NODE_URL } = process.env;

export const startPXE = async (options: any, signalHandlers: (() => Promise<void>)[], userLog: LogFn) => {
  // Services that will be started in a single multi-rpc server
  const services: ServerList = [];
  // Starting a PXE with a remote node.
  // get env vars first
  const pxeConfigEnvVars = getPXEServiceConfig();
  // get config from options
  const pxeCliOptions = parseModuleOptions(options.pxe);

  // Determine node url from options or env vars
  const nodeUrl = pxeCliOptions.nodeUrl || AZTEC_NODE_URL;
  // throw if no Aztec Node URL is provided
  if (!nodeUrl) {
    userLog('Aztec Node URL (nodeUrl | AZTEC_NODE_URL) option is required to start PXE without --node option');
    throw new Error('Aztec Node URL (nodeUrl | AZTEC_NODE_URL) option is required to start PXE without --node option');
  }

  // merge env vars and cli options
  const pxeConfig = mergeEnvVarsAndCliOptions<PXEServiceConfig>(pxeConfigEnvVars, pxeCliOptions);

  // create a node client
  const node = createAztecNodeClient(nodeUrl);

  const pxe = await createPXEService(node, pxeConfig);
  const pxeServer = createPXERpcServer(pxe);
  services.push({ pxe: pxeServer });
  signalHandlers.push(pxe.stop);
  return services;
};
