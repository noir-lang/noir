#!/usr/bin/env -S node --no-warnings
import { NULL_KEY, createEthereumChain } from '@aztec/ethereum';
import { EthAddress } from '@aztec/foundation/eth-address';
import { createDebugLogger } from '@aztec/foundation/log';
import { PortalERC20Abi } from '@aztec/l1-artifacts';

import http from 'http';
import Koa from 'koa';
import cors from 'koa-cors';
import Router from 'koa-router';
import {
  type Hex,
  type LocalAccount,
  http as ViemHttp,
  createPublicClient,
  createWalletClient,
  getContract,
  parseEther,
} from 'viem';
import { mnemonicToAccount, privateKeyToAccount } from 'viem/accounts';

const {
  FAUCET_PORT = 8082,
  API_PREFIX = '',
  RPC_URL = '',
  L1_CHAIN_ID = '',
  FORK_MNEMONIC = '',
  FAUCET_ACCOUNT_INDEX = '',
  PRIVATE_KEY = '',
  INTERVAL = '',
  ETH_AMOUNT = '',
  // asset_name:contract_address
  EXTRA_ASSETS = '',
  EXTRA_ASSET_AMOUNT = '',
} = process.env;

const logger = createDebugLogger('aztec:faucet');

const rpcUrl = RPC_URL;
const l1ChainId = +L1_CHAIN_ID;
const interval = +INTERVAL;
type AssetName = string & { __brand: 'AssetName' };
type ThrottleKey = `${'eth' | AssetName}/${Hex}`;
type Assets = Record<AssetName, Hex>;

const mapping: { [key: ThrottleKey]: Date } = {};
const assets: Assets = {};

if (EXTRA_ASSETS) {
  const assetList = EXTRA_ASSETS.split(',');
  assetList.forEach(asset => {
    const [name, address] = asset.split(':');
    if (!name || !address) {
      throw new Error(`Invalid asset: ${asset}`);
    }
    assets[name as AssetName] = createHex(address);
  });
}

class ThrottleError extends Error {
  constructor(address: string) {
    super(`Not funding address ${address}, please try again later`);
  }
}

/**
 * Checks if the requested asset is something the faucet can handle.
 * @param asset - The asset to check
 * @returns True if the asset is known
 */
function isKnownAsset(asset: any): asset is 'eth' | AssetName {
  return asset === 'eth' || asset in assets;
}

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
function checkThrottle(asset: 'eth' | AssetName, address: Hex) {
  const key: ThrottleKey = `${asset}/${address}`;
  if (mapping[key] === undefined) {
    return;
  }
  const last = mapping[key];
  const current = new Date();
  const diff = (current.getTime() - last.getTime()) / 1000;
  if (diff < interval) {
    throw new ThrottleError(address);
  }
}

/**
 * Update the throttle mapping for the given asset and address
 * @param asset - The asset to throttle
 * @param address - The address to throttle
 */
function updateThrottle(asset: 'eth' | AssetName, address: Hex) {
  const key: ThrottleKey = `${asset}/${address}`;
  mapping[key] = new Date();
}

/**
 * Get the account to use for sending ETH
 * @returns The account to use for sending ETH
 */
function getFaucetAccount(): LocalAccount {
  let account: LocalAccount;
  if (FORK_MNEMONIC) {
    const accountIndex = Number.isNaN(+FAUCET_ACCOUNT_INDEX) ? 0 : +FAUCET_ACCOUNT_INDEX;
    account = mnemonicToAccount(FORK_MNEMONIC, {
      accountIndex,
    });
  } else if (PRIVATE_KEY) {
    account = privateKeyToAccount(PRIVATE_KEY as `0x${string}`);
  } else {
    logger.warn('No mnemonic or private key provided, using null key');
    account = privateKeyToAccount(NULL_KEY);
  }

  return account;
}

function createClients() {
  const chain = createEthereumChain(rpcUrl, l1ChainId);

  const account = getFaucetAccount();
  const walletClient = createWalletClient({
    account: account,
    chain: chain.chainInfo,
    transport: ViemHttp(chain.rpcUrl),
  });
  const publicClient = createPublicClient({
    chain: chain.chainInfo,
    transport: ViemHttp(chain.rpcUrl),
  });

  return { account, walletClient, publicClient };
}

/**
 * Helper function to send some ETH to the given address
 * @param address - Address to receive some ETH
 */
async function transferEth(address: string) {
  const { account, walletClient, publicClient } = createClients();
  const hexAddress = createHex(address);
  checkThrottle('eth', hexAddress);
  try {
    const hash = await walletClient.sendTransaction({
      account,
      to: hexAddress,
      value: parseEther(ETH_AMOUNT),
    });
    await publicClient.waitForTransactionReceipt({ hash });
    updateThrottle('eth', hexAddress);
    logger.info(`Sent ${ETH_AMOUNT} ETH to ${hexAddress} in tx ${hash}`);
  } catch (error) {
    logger.error(`Failed to send eth to ${hexAddress}`);
    throw error;
  }
}

/**
 * Mints FeeJuice to the given address
 * @param address - Address to receive some FeeJuice
 */
async function transferAsset(assetName: AssetName, address: string) {
  const { publicClient, walletClient } = createClients();
  const hexAddress = createHex(address);
  checkThrottle(assetName, hexAddress);

  const assetAddress = assets[assetName];

  try {
    const contract = getContract({
      abi: PortalERC20Abi,
      address: assetAddress,
      client: walletClient,
    });

    const amount = BigInt(EXTRA_ASSET_AMOUNT);
    const hash = await contract.write.mint([hexAddress, amount]);
    await publicClient.waitForTransactionReceipt({ hash });
    updateThrottle(assetName, hexAddress);
    logger.info(`Sent ${amount} ${assetName} to ${hexAddress} in tx ${hash}`);
  } catch (err) {
    logger.error(`Failed to send ${assetName} to ${hexAddress}`);
    throw err;
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
    const { asset } = ctx.query;

    if (!asset) {
      throw new Error('No asset specified');
    }

    if (!isKnownAsset(asset)) {
      throw new Error(`Unknown asset: "${asset}"`);
    }

    if (asset === 'eth') {
      await transferEth(EthAddress.fromString(address).toChecksumString());
    } else {
      await transferAsset(asset, EthAddress.fromString(address).toChecksumString());
    }

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
      ctx.status = err instanceof ThrottleError ? 429 : 400;
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
