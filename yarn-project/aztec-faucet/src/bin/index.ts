#!/usr/bin/env -S node --no-warnings
import { NULL_KEY, createEthereumChain } from '@aztec/ethereum';
import { EthAddress } from '@aztec/foundation/eth-address';
import { createDebugLogger } from '@aztec/foundation/log';

import http from 'http';
import Koa from 'koa';
import cors from 'koa-cors';
import Router from 'koa-router';
import { type Hex, http as ViemHttp, createPublicClient, createWalletClient, parseEther } from 'viem';
import { privateKeyToAccount } from 'viem/accounts';

const {
  FAUCET_PORT = 8082,
  API_PREFIX = '',
  RPC_URL = '',
  L1_CHAIN_ID = '',
  PRIVATE_KEY = '',
  INTERVAL = '',
  ETH_AMOUNT = '',
} = process.env;

const logger = createDebugLogger('aztec:faucet');

const rpcUrl = RPC_URL;
const l1ChainId = +L1_CHAIN_ID;
const privateKey: Hex = PRIVATE_KEY ? createHex(PRIVATE_KEY) : NULL_KEY;
const interval = +INTERVAL;
const mapping: { [key: Hex]: Date } = {};

/**
 * Helper function to convert a string to a Hex value
 * @param hex - The string to convert
 * @returns The converted value
 */
function createHex(hex: string) {
  return `0x${hex.replace('0x', '')}` as Hex;
}

/**
 * Function to throttle drips on a per address basis
 * @param address - Address requesting some ETH
 */
function checkThrottle(address: Hex) {
  if (mapping[address] === undefined) {
    return;
  }
  const last = mapping[address];
  const current = new Date();
  const diff = (current.getTime() - last.getTime()) / 1000;
  if (diff < interval) {
    throw new Error(`Not funding address ${address}, please try again later`);
  }
}

/**
 * Helper function to send some ETH to the given address
 * @param address - Address to receive some ETH
 */
async function transferEth(address: string) {
  const chain = createEthereumChain(rpcUrl, l1ChainId);

  const account = privateKeyToAccount(privateKey);
  const walletClient = createWalletClient({
    account: account,
    chain: chain.chainInfo,
    transport: ViemHttp(chain.rpcUrl),
  });
  const publicClient = createPublicClient({
    chain: chain.chainInfo,
    transport: ViemHttp(chain.rpcUrl),
  });
  const hexAddress = createHex(address);
  checkThrottle(hexAddress);
  try {
    const hash = await walletClient.sendTransaction({
      account,
      to: hexAddress,
      value: parseEther(ETH_AMOUNT),
    });
    await publicClient.waitForTransactionReceipt({ hash });
    mapping[hexAddress] = new Date();
    logger.info(`Sent ${ETH_AMOUNT} ETH to ${hexAddress} in tx ${hash}`);
  } catch (error) {
    logger.error(`Failed to send eth to ${hexAddress}`);
    throw error;
  }
}

/**
 * Creates a router for the faucet.
 * @param apiPrefix - The prefix to use for all api requests
 * @returns - The router for handling status requests.
 */
function createRouter(apiPrefix: string) {
  logger.info(`Creating router with prefix ${apiPrefix}`);
  const router = new Router({ prefix: `${apiPrefix}` });
  router.get('/status', (ctx: Koa.Context) => {
    ctx.status = 200;
  });
  router.get('/drip/:address', async (ctx: Koa.Context) => {
    const { address } = ctx.params;
    await transferEth(EthAddress.fromString(address).toChecksumString());
    ctx.status = 200;
  });
  return router;
}

/**
 * Create and start a new Aztec Node HTTP Server
 */
async function main() {
  logger.info(`Setting up Aztec Faucet...`);

  const chain = createEthereumChain(rpcUrl, l1ChainId);
  if (chain.chainInfo.id !== l1ChainId) {
    throw new Error(`Incorrect chain id, expected ${chain.chainInfo.id}`);
  }

  const shutdown = () => {
    logger.info('Shutting down...');
    process.exit(0);
  };

  process.once('SIGINT', shutdown);
  process.once('SIGTERM', shutdown);

  const app = new Koa();
  app.on('error', error => {
    logger.error(`Error on API handler: ${error}`);
  });
  const exceptionHandler = async (ctx: Koa.Context, next: () => Promise<void>) => {
    try {
      await next();
    } catch (err: any) {
      logger.error(err);
      ctx.status = 400;
      ctx.body = { error: err.message };
    }
  };
  app.use(exceptionHandler);
  app.use(cors());
  const apiRouter = createRouter(API_PREFIX);
  app.use(apiRouter.routes());
  app.use(apiRouter.allowedMethods());

  const httpServer = http.createServer(app.callback());
  httpServer.listen(+FAUCET_PORT);
  logger.info(`Aztec Faucet listening on port ${FAUCET_PORT}`);
  await Promise.resolve();
}

main().catch(err => {
  logger.error(err);
  process.exit(1);
});
