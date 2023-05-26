import { AztecNode, txFromJson, txToJson } from '@aztec/aztec-node';
import { Fr } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { createDebugLogger } from '@aztec/foundation/log';
import { MerkleTreeId, TxHash } from '@aztec/types';
import Koa, { Context, DefaultState } from 'koa';
import Router from 'koa-router';
import { PromiseReadable } from 'promise-readable';

const logger = createDebugLogger('aztec:http_router');

/**
 * Factory method for constructing the http service.
 * @param node - An instance of Aztec Node into which calls are forwared.
 * @param prefix - A prefix for the http service's api routes
 * @returns The constructed http servce.
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

  router.get('/get-blocks', async (ctx: Koa.Context) => {
    const from = +ctx.query.from!;
    const take = +ctx.query.take!;
    const blocks = await node.getBlocks(from, take);
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

  router.get('/contract-data', async (ctx: Koa.Context) => {
    const address = ctx.query.address;
    ctx.set('content-type', 'application/json');
    ctx.body = {
      contractData: await node.getContractData(AztecAddress.fromString(address as string)),
    };
    ctx.status = 200;
  });

  router.get('/contract-info', async (ctx: Koa.Context) => {
    const address = ctx.query.address;
    ctx.set('content-type', 'application/json');
    ctx.body = {
      contractInfo: await node.getContractData(AztecAddress.fromString(address as string)),
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

  router.get('/get-unverified', async (ctx: Koa.Context) => {
    const from = +ctx.query.from!;
    const take = +ctx.query.take!;
    const blocks = await node.getUnverifiedData(from, take);
    const strs = blocks.map(x => x.toBuffer().toString('hex'));
    ctx.set('content-type', 'application/json');
    ctx.body = {
      unverified: strs,
    };
    ctx.status = 200;
  });

  router.get('/get-pending-tx', async (ctx: Koa.Context) => {
    const hash = ctx.query.hash!;
    const txHash = new TxHash(Buffer.from(hash as string, 'hex'));
    const tx = await node.getPendingTxByHash(txHash);
    ctx.set('content-type', 'application/json');
    ctx.body = {
      tx: tx == undefined ? undefined : txToJson(tx),
    };
    ctx.status = 200;
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

  router.get('/storage-at', async (ctx: Koa.Context) => {
    logger('storage-at');
    const address = ctx.query.address!;
    const slot = ctx.query.slot!;
    const value = await node.getStorageAt(AztecAddress.fromString(address as string), BigInt(slot as string));
    ctx.set('content-type', 'application/json');
    ctx.body = {
      value: value?.toString('hex'),
    };
    ctx.status = 200;
  });

  router.post('/tx', checkReady, async (ctx: Koa.Context) => {
    const stream = new PromiseReadable(ctx.req);
    const postData = JSON.parse((await stream.readAll()) as string);
    const tx = txFromJson(postData);
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
