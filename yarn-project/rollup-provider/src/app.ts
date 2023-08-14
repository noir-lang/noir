import { Fr, HistoricBlockData } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { createDebugLogger } from '@aztec/foundation/log';
import { AztecNode, MerkleTreeId, Tx, TxHash } from '@aztec/types';

import Koa, { Context, DefaultState } from 'koa';
import Router from 'koa-router';
import { PromiseReadable } from 'promise-readable';

const logger = createDebugLogger('aztec:http_router');

/**
 * Factory method for constructing the http service.
 * @param node - An instance of Aztec Node into which calls are forwarded.
 * @param prefix - A prefix for the http service's api routes
 * @returns The constructed http service.
 */
export function appFactory(node: AztecNode, prefix: string) {
  const router = new Router<DefaultState, Context>({ prefix });

  const checkReady = async (ctx: Context, next: () => Promise<void>) => {
    if (!(await node.isReady())) {
      ctx.status = 503;
      ctx.body = { error: 'Server not ready. Try again later.' };
    } else {
      await next();
    }
  };

  const exceptionHandler = async (ctx: Koa.Context, next: () => Promise<void>) => {
    try {
      await next();
    } catch (err: any) {
      logger(err);
      ctx.status = 400;
      ctx.body = { error: err.message };
    }
  };

  router.get('/', async (ctx: Koa.Context) => {
    ctx.body = {
      serviceName: 'aztec rollup',
      isReady: await node.isReady(),
    };
    ctx.set('content-type', 'application/json');
    ctx.status = 200;
  });

  router.get('/get-block', async (ctx: Koa.Context) => {
    const number = +ctx.query.number!;
    const block = await node.getBlock(number);
    const str = block?.encode().toString('hex');
    ctx.set('content-type', 'application/json');
    ctx.body = {
      block: str,
    };
    ctx.status = 200;
  });

  router.get('/get-blocks', async (ctx: Koa.Context) => {
    const from = +ctx.query.from!;
    const limit = +ctx.query.limit!;
    const blocks = await node.getBlocks(from, limit);
    const strs = blocks.map(x => x.encode().toString('hex'));
    ctx.set('content-type', 'application/json');
    ctx.body = {
      blocks: strs,
    };
    ctx.status = 200;
  });

  router.get('/get-block-height', async (ctx: Koa.Context) => {
    ctx.set('content-type', 'application/json');
    ctx.body = {
      blockHeight: await node.getBlockHeight(),
    };
    ctx.status = 200;
  });

  router.get('/get-version', async (ctx: Koa.Context) => {
    ctx.set('content-type', 'application/json');
    ctx.body = {
      version: await node.getVersion(),
    };
    ctx.status = 200;
  });

  router.get('/get-chain-id', async (ctx: Koa.Context) => {
    ctx.set('content-type', 'application/json');
    ctx.body = {
      chainId: await node.getChainId(),
    };
    ctx.status = 200;
  });

  router.get('/get-rollup-address', async (ctx: Koa.Context) => {
    ctx.set('content-type', 'application/json');
    ctx.body = {
      rollupAddress: (await node.getRollupAddress()).toString(),
    };
    ctx.status = 200;
  });

  router.get('/contract-data-and-bytecode', async (ctx: Koa.Context) => {
    const address = ctx.query.address;
    ctx.set('content-type', 'application/json');
    ctx.body = {
      contractData: await node.getContractDataAndBytecode(AztecAddress.fromString(address as string)),
    };
    ctx.status = 200;
  });

  router.get('/contract-data', async (ctx: Koa.Context) => {
    const address = ctx.query.address;
    ctx.set('content-type', 'application/json');
    ctx.body = {
      contractData: await node.getContractData(AztecAddress.fromString(address as string)),
    };
    ctx.status = 200;
  });

  router.get('/tree-roots', async (ctx: Koa.Context) => {
    const roots: Record<MerkleTreeId, Fr> = await node.getTreeRoots();
    const output: { [key: string]: string } = {};
    for (const [key, value] of Object.entries(roots)) {
      output[key] = value.toString();
    }
    ctx.body = {
      roots: output,
    };
    ctx.status = 200;
  });

  router.get('/historic-block-data', async (ctx: Koa.Context) => {
    const blockData: HistoricBlockData = await node.getHistoricBlockData();
    const output: { [key: string]: string } = {};
    for (const [key, value] of Object.entries(blockData)) {
      output[key] = value.toString();
    }
    ctx.body = {
      blockData: output,
    };
    ctx.status = 200;
  });

  router.get('/get-logs', async (ctx: Koa.Context) => {
    const from = +ctx.query.from!;
    const limit = +ctx.query.limit!;
    const logType = Number(ctx.query.logType);
    if (logType !== 0 && logType !== 1) {
      throw new Error('Invalid log type: ' + ctx.query.logType);
    }

    const logs = await node.getLogs(from, limit, logType);
    const strs = logs.map(x => x.toBuffer().toString('hex'));
    ctx.set('content-type', 'application/json');
    ctx.body = {
      logs: strs,
    };
    ctx.status = 200;
  });

  router.get('/get-pending-tx', async (ctx: Koa.Context) => {
    const hash = ctx.query.hash!;
    const txHash = new TxHash(Buffer.from(hash as string, 'hex'));
    const tx = await node.getPendingTxByHash(txHash);
    ctx.set('content-type', 'application/octet-stream');
    if (tx == undefined) {
      ctx.status = 404;
    } else {
      ctx.status = 200;
      ctx.body = tx.toBuffer();
    }
  });

  router.get('/contract-index', async (ctx: Koa.Context) => {
    const leaf = ctx.query.leaf!;
    const index = await node.findContractIndex(Buffer.from(leaf as string, 'hex'));
    ctx.set('content-type', 'application/json');
    ctx.body = {
      index,
    };
    ctx.status = 200;
  });

  router.get('/contract-path', async (ctx: Koa.Context) => {
    const leaf = ctx.query.leaf!;
    const path = await node.getContractPath(BigInt(leaf as string));
    ctx.set('content-type', 'application/json');
    ctx.body = {
      path: path.toString(),
    };
    ctx.status = 200;
  });

  router.get('/commitment-index', async (ctx: Koa.Context) => {
    const leaf = ctx.query.leaf!;
    const index = await node.findCommitmentIndex(Buffer.from(leaf as string, 'hex'));
    ctx.set('content-type', 'application/json');
    ctx.body = {
      index,
    };
    ctx.status = 200;
  });

  router.get('/data-path', async (ctx: Koa.Context) => {
    const leaf = ctx.query.leaf!;
    const index = BigInt(leaf as string);
    const path = await node.getDataTreePath(index);
    ctx.set('content-type', 'application/json');
    const pathAsString = path.toString();
    ctx.body = {
      path: pathAsString,
    };
    ctx.status = 200;
  });

  router.get('/l1-l2-message', async (ctx: Koa.Context) => {
    const key = ctx.query.messageKey!;
    const messageAndindex = await node.getL1ToL2MessageAndIndex(Fr.fromString(key as string));
    ctx.set('content-type', 'application/json');
    ctx.body = {
      message: messageAndindex.message.toBuffer().toString('hex'),
      index: messageAndindex.index,
    };
    ctx.status = 200;
  });

  router.get('/l1-l2-path', async (ctx: Koa.Context) => {
    const leaf = ctx.query.leaf!;
    const path = await node.getL1ToL2MessagesTreePath(BigInt(leaf as string));
    ctx.set('content-type', 'application/json');
    ctx.body = {
      path: path.toString(),
    };
    ctx.status = 200;
  });

  router.get('/public-storage-at', async (ctx: Koa.Context) => {
    logger('public-storage-at');
    const address = ctx.query.address!;
    const slot = ctx.query.slot!;
    const value = await node.getPublicStorageAt(AztecAddress.fromString(address as string), BigInt(slot as string));
    ctx.set('content-type', 'application/json');
    ctx.body = {
      value: value?.toString('hex'),
    };
    ctx.status = 200;
  });

  router.post('/tx', checkReady, async (ctx: Koa.Context) => {
    const stream = new PromiseReadable(ctx.req);
    const postData = (await stream.readAll()) as Buffer;
    const tx = Tx.fromBuffer(postData);
    await node.sendTx(tx);
    ctx.status = 200;
  });

  const app = new Koa();
  app.on('error', error => {
    logger(`KOA app-level error. ${JSON.stringify({ error })}`);
  });
  app.proxy = true;
  app.use(exceptionHandler);
  app.use(router.routes());
  app.use(router.allowedMethods());

  return app;
}
