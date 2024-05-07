import { type AztecNodeService } from '@aztec/aztec-node';
import { type AccountWallet, EthAddress, PublicFeePaymentMethod, TxStatus } from '@aztec/aztec.js';
import { GasSettings } from '@aztec/circuits.js';
import { FPCContract, GasTokenContract, TestContract, TokenContract } from '@aztec/noir-contracts.js';
import { getCanonicalGasTokenAddress } from '@aztec/protocol-contracts/gas-token';
import { ProverPool } from '@aztec/prover-client/prover-pool';

import { jest } from '@jest/globals';

import { getACVMConfig } from '../fixtures/get_acvm_config.js';
import { getBBConfig } from '../fixtures/get_bb_config.js';
import { type EndToEndContext, publicDeployAccounts, setup } from '../fixtures/utils.js';

jest.setTimeout(600_000);

const txTimeoutSec = 600;

describe('benchmarks/proving', () => {
  let ctx: EndToEndContext;
  let wallet: AccountWallet;
  let testContract: TestContract;
  let tokenContract: TokenContract;
  let fpContract: FPCContract;
  let acvmCleanup: () => Promise<void>;
  let bbCleanup: () => Promise<void>;
  let proverPool: ProverPool;

  // setup the environment quickly using fake proofs
  beforeAll(async () => {
    ctx = await setup(
      1,
      {
        // do setup with fake proofs
        realProofs: false,
        proverAgents: 4,
        proverAgentPollInterval: 10,
        minTxsPerBlock: 1,
      },
      {},
      true, // enable gas
    );

    wallet = ctx.wallet;

    await publicDeployAccounts(wallet, ctx.wallets);

    testContract = await TestContract.deploy(wallet).send().deployed();
    tokenContract = await TokenContract.deploy(wallet, wallet.getAddress(), 'test', 't', 18).send().deployed();
    const gas = await GasTokenContract.at(
      getCanonicalGasTokenAddress(ctx.deployL1ContractsValues.l1ContractAddresses.gasPortalAddress),
      wallet,
    );
    fpContract = await FPCContract.deploy(wallet, tokenContract.address, gas.address).send().deployed();

    await Promise.all([
      gas.methods.mint_public(fpContract.address, 1e12).send().wait(),
      tokenContract.methods.mint_public(wallet.getAddress(), 1e12).send().wait(),
    ]);
  });

  // remove the fake prover and setup the real one
  beforeAll(async () => {
    const [acvmConfig, bbConfig] = await Promise.all([getACVMConfig(ctx.logger), getBBConfig(ctx.logger)]);
    if (!acvmConfig || !bbConfig) {
      throw new Error('Missing ACVM or BB config');
    }

    acvmCleanup = acvmConfig.cleanup;
    bbCleanup = bbConfig.cleanup;

    proverPool = ProverPool.nativePool(
      {
        ...acvmConfig,
        ...bbConfig,
      },
      4,
      10,
    );

    ctx.logger.info('Stopping fake provers');
    await ctx.aztecNode.setConfig({
      // stop the fake provers
      proverAgents: 0,
      // 4-tx blocks so that we have at least one merge level
      minTxsPerBlock: 4,
    });

    ctx.logger.info('Starting real provers');
    await proverPool.start((ctx.aztecNode as AztecNodeService).getProver().getProvingJobSource());
  });

  afterAll(async () => {
    await proverPool.stop();
    await ctx.teardown();
    await acvmCleanup();
    await bbCleanup();
  });

  it('builds a full block', async () => {
    const txs = [
      // fully private tx
      testContract.methods.emit_nullifier(42).send(),
      // tx with setup, app, teardown
      testContract.methods.emit_unencrypted(43).send({
        fee: {
          gasSettings: GasSettings.default(),
          paymentMethod: new PublicFeePaymentMethod(tokenContract.address, fpContract.address, wallet),
        },
      }),
      // tx with messages
      testContract.methods.create_l2_to_l1_message_public(45, 46, EthAddress.random()).send(),
      // tx with private and public exec
      testContract.methods.set_tx_max_block_number(100, true).send({
        fee: {
          gasSettings: GasSettings.default(),
          paymentMethod: new PublicFeePaymentMethod(tokenContract.address, fpContract.address, wallet),
        },
      }),
    ];

    const receipts = await Promise.all(txs.map(tx => tx.wait({ timeout: txTimeoutSec })));
    expect(receipts.every(r => r.status === TxStatus.MINED)).toBe(true);
  });
});
