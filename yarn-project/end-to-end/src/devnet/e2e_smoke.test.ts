import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import {
  type EthAddress,
  FeeJuicePaymentMethodWithClaim,
  Fr,
  type PXE,
  SignerlessWallet,
  TxStatus,
  type WaitOpts,
  createAztecNodeClient,
  createPXEClient,
  fileURLToPath,
  retryUntil,
} from '@aztec/aztec.js';
import { DefaultMultiCallEntrypoint } from '@aztec/aztec.js/entrypoint';
import { GasSettings, deriveSigningKey } from '@aztec/circuits.js';
import { startHttpRpcServer } from '@aztec/foundation/json-rpc/server';
import { type DebugLogger } from '@aztec/foundation/log';
import { promiseWithResolvers } from '@aztec/foundation/promise';
import { FeeJuiceContract, TestContract } from '@aztec/noir-contracts.js';
import { createPXERpcServer } from '@aztec/pxe';

import getPort from 'get-port';
import { exec } from 'node:child_process';
import { lookup } from 'node:dns/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';

import { getACVMConfig } from '../fixtures/get_acvm_config.js';
import { getBBConfig } from '../fixtures/get_bb_config.js';
import { getLogger, setupPXEService } from '../fixtures/utils.js';

const {
  AZTEC_NODE_URL,
  PXE_URL,
  FAUCET_URL,
  AZTEC_CLI = `node ${resolve(fileURLToPath(import.meta.url), '../../../../aztec/dest/bin/index.js')}`,
  ETHEREUM_HOST,
  PXE_PROVER_ENABLED = '0',
  USE_EMPTY_BLOCKS = '0',
} = process.env;

const waitOpts: WaitOpts = { timeout: 3600, interval: 1 };

const MIN_BLOCKS_FOR_BRIDGING = 2;

/**
 * If we can successfully resolve 'host.docker.internal', then we are running in a container, and we should treat
 * localhost as being host.docker.internal.
 */
export const getLocalhost = () =>
  lookup('host.docker.internal')
    .then(() => 'host.docker.internal')
    .catch(() => 'localhost');

describe('End-to-end tests for devnet', () => {
  // eslint-disable-next-line
  let pxe: PXE;
  let pxeUrl: string; // needed for the CLI
  let logger: DebugLogger;
  let l1ChainId: number;
  let feeJuiceL1: EthAddress;
  let teardown: () => void | Promise<void>;

  beforeAll(async () => {
    logger = getLogger();

    if (!ETHEREUM_HOST) {
      throw new Error('ETHEREUM_HOST must be set');
    }

    if (!AZTEC_CLI) {
      throw new Error('AZTEC_CLI must be set');
    }

    if (!FAUCET_URL) {
      throw new Error('FAUCET_URL must be set');
    }

    logger.info(`Using AZTEC_CLI: ${AZTEC_CLI}`);

    if (AZTEC_NODE_URL) {
      logger.info(`Using AZTEC_NODE_URL: ${AZTEC_NODE_URL}`);
      const node = createAztecNodeClient(AZTEC_NODE_URL);
      const bbConfig = await getBBConfig(logger);
      const acvmConfig = await getACVMConfig(logger);
      const svc = await setupPXEService(node, {
        ...bbConfig,
        ...acvmConfig,
        proverEnabled: ['1', 'true'].includes(PXE_PROVER_ENABLED!),
      });
      pxe = svc.pxe;

      const nodeInfo = await pxe.getNodeInfo();
      const pxeInfo = await pxe.getPXEInfo();

      expect(nodeInfo.protocolContractAddresses.classRegisterer).toEqual(
        pxeInfo.protocolContractAddresses.classRegisterer,
      );
      expect(nodeInfo.protocolContractAddresses.instanceDeployer).toEqual(
        pxeInfo.protocolContractAddresses.instanceDeployer,
      );
      expect(nodeInfo.protocolContractAddresses.feeJuice).toEqual(pxeInfo.protocolContractAddresses.feeJuice);
      expect(nodeInfo.protocolContractAddresses.keyRegistry).toEqual(pxeInfo.protocolContractAddresses.keyRegistry);
      expect(nodeInfo.protocolContractAddresses.multiCallEntrypoint).toEqual(
        pxeInfo.protocolContractAddresses.multiCallEntrypoint,
      );

      const port = await getPort();
      const localhost = await getLocalhost();
      pxeUrl = `http://${localhost}:${port}`;
      // start a server for the CLI to talk to
      const server = startHttpRpcServer('pxe', pxe, createPXERpcServer, port);

      teardown = async () => {
        const { promise, resolve, reject } = promiseWithResolvers<void>();
        server.close(e => (e ? reject(e) : resolve()));
        await promise;

        await svc.teardown();
        await bbConfig?.cleanup();
        await acvmConfig?.cleanup();
      };
    } else if (PXE_URL) {
      logger.info(`Using PXE_URL: ${PXE_URL}`);
      pxe = createPXEClient(PXE_URL);
      pxeUrl = PXE_URL;
      teardown = () => {};
    } else {
      throw new Error('AZTEC_NODE_URL or PXE_URL must be set');
    }

    ({
      l1ChainId,
      l1ContractAddresses: { feeJuiceAddress: feeJuiceL1 },
    } = await pxe.getNodeInfo());
    logger.info(`PXE instance started`);
  });

  afterAll(async () => {
    await teardown();
  });

  it('deploys an account while paying with FeeJuice', async () => {
    const privateKey = Fr.random();
    const l1Account = await cli<{ privateKey: string; address: string }>('create-l1-account');
    const l2Account = getSchnorrAccount(pxe, privateKey, deriveSigningKey(privateKey), Fr.ZERO);

    await expect(getL1Balance(l1Account.address)).resolves.toEqual(0n);
    await expect(getL1Balance(l1Account.address, feeJuiceL1)).resolves.toEqual(0n);

    await cli('drip-faucet', { 'faucet-url': FAUCET_URL!, token: 'eth', address: l1Account.address });
    await expect(getL1Balance(l1Account.address)).resolves.toBeGreaterThan(0n);

    await cli('drip-faucet', { 'faucet-url': FAUCET_URL!, token: 'fee_juice', address: l1Account.address });
    await expect(getL1Balance(l1Account.address, feeJuiceL1)).resolves.toBeGreaterThan(0n);

    const amount = 1_000_000_000_000n;
    const { claimAmount, claimSecret } = await cli<{ claimAmount: string; claimSecret: { value: string } }>(
      'bridge-fee-juice',
      [amount, l2Account.getAddress()],
      {
        'l1-rpc-url': ETHEREUM_HOST!,
        'l1-chain-id': l1ChainId.toString(),
        'l1-private-key': l1Account.privateKey,
        'rpc-url': pxeUrl,
        mint: true,
      },
    );

    if (['1', 'true', 'yes'].includes(USE_EMPTY_BLOCKS)) {
      await advanceChainWithEmptyBlocks(pxe);
    } else {
      await waitForL1MessageToArrive();
    }

    const txReceipt = await l2Account
      .deploy({
        fee: {
          gasSettings: GasSettings.default(),
          paymentMethod: new FeeJuicePaymentMethodWithClaim(
            l2Account.getAddress(),
            BigInt(claimAmount),
            Fr.fromString(claimSecret.value),
          ),
        },
      })
      .wait(waitOpts);

    // disabled because the CLI process doesn't exit
    // const { txHash, address } = await cli<{ txHash: string; address: { value: string } }>('create-account', {
    //   'private-key': privateKey,
    //   payment: `method=fee_juice,claimSecret=${claimSecret.value},claimAmount=${claimAmount}`,
    //   wait: false,
    // });
    // expect(address).toEqual(l2Account.getAddress().toString());
    // const txReceipt = await retryUntil(
    //   async () => {
    //     const receipt = await pxe.getTxReceipt(txHash);
    //     if (receipt.status === TxStatus.PENDING) {
    //       return undefined;
    //     }
    //     return receipt;
    //   },
    //   'wait_for_l2_account',
    //   waitOpts.timeout,
    //   waitOpts.interval,
    // );

    expect(txReceipt.status).toBe(TxStatus.SUCCESS);
    const feeJuice = await FeeJuiceContract.at(
      (
        await pxe.getNodeInfo()
      ).protocolContractAddresses.feeJuice,
      await l2Account.getWallet(),
    );
    const balance = await feeJuice.methods.balance_of_public(l2Account.getAddress()).simulate();
    expect(balance).toEqual(amount - txReceipt.transactionFee!);
  });

  type OptionValue = null | undefined | boolean | { toString(): string };
  type ArgumentValue = { toString(): string };

  function cli<T>(cliCommand: string): Promise<T>;
  function cli<T>(cliCommand: string, args: ArgumentValue[]): Promise<T>;
  function cli<T>(cliCommand: string, opts: Record<string, OptionValue>): Promise<T>;
  function cli<T>(cliCommand: string, args: ArgumentValue[], opts: Record<string, OptionValue>): Promise<T>;
  function cli<T>(
    cliCommand: string,
    _args?: ArgumentValue[] | Record<string, OptionValue>,
    _opts?: Record<string, OptionValue>,
  ): Promise<T> {
    const { promise, resolve, reject } = promiseWithResolvers<T>();
    const args = Array.isArray(_args) ? _args : [];
    const opts = _args && !Array.isArray(_args) ? _args : typeof _opts !== 'undefined' ? _opts : {};

    const cmdArguments = args.map(x => x.toString());

    opts.json = true;
    const cmdOptions = Object.entries(opts)
      .filter((entry): entry is [string, { toString(): string }] => entry[1] !== undefined && entry[1] !== null)
      .map(([key, value]) =>
        typeof value === 'boolean' ? (value ? `--${key}` : `--no-${key}`) : `--${key} ${value.toString()}`,
      );

    const cmd = `${AZTEC_CLI} ${cliCommand} ${cmdArguments.join(' ')} ${cmdOptions.join(' ')}`;
    logger.info(`Running aztec-cli: ${cmd}`);
    const child = exec(cmd, {
      cwd: tmpdir(),
      env: {
        NODE_OPTIONS: '--no-warnings',
      },
    });

    let err = '';
    child.stderr?.on('data', data => {
      logger.error(data.toString());
      err += data.toString();
    });

    let out = '';
    child.stdout?.on('data', data => {
      out += data.toString();
    });

    child.on('error', reject);
    child.on('close', (code, signal) => {
      if (code === 0) {
        const res = JSON.parse(out);
        logger.info(`aztec-cli JSON output: ${JSON.stringify(res)}`);
        resolve(res);
      } else {
        reject(new Error(`aztec-cli ${cliCommand} non-zero exit: code=${code} signal=${signal} ${err}`));
      }
    });

    return promise;
  }

  async function getL1Balance(address: string, token?: EthAddress): Promise<bigint> {
    const { balance } = await cli<{ balance: string }>('get-l1-balance', [address], {
      'l1-rpc-url': ETHEREUM_HOST!,
      'l1-chain-id': l1ChainId.toString(),
      token,
    });

    return BigInt(balance);
  }

  async function waitForL1MessageToArrive() {
    const targetBlockNumber = (await pxe.getBlockNumber()) + MIN_BLOCKS_FOR_BRIDGING;
    await retryUntil(async () => (await pxe.getBlockNumber()) >= targetBlockNumber, 'wait_for_l1_message', 0, 10);
  }

  async function advanceChainWithEmptyBlocks(pxe: PXE) {
    const { l1ChainId, protocolVersion } = await pxe.getNodeInfo();
    const test = await TestContract.deploy(
      new SignerlessWallet(pxe, new DefaultMultiCallEntrypoint(l1ChainId, protocolVersion)),
    )
      .send({ universalDeploy: true, skipClassRegistration: true })
      .deployed();

    // start at 1 because deploying the contract has already mined a block
    for (let i = 1; i < MIN_BLOCKS_FOR_BRIDGING; i++) {
      await test.methods.get_this_address().send().wait(waitOpts);
    }
  }
});
