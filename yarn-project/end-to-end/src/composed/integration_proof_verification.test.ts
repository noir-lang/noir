import { L2Block, deployL1Contract, fileURLToPath } from '@aztec/aztec.js';
import { BBCircuitVerifier } from '@aztec/bb-prover';
import { AGGREGATION_OBJECT_LENGTH, Fr, HEADER_LENGTH, Proof } from '@aztec/circuits.js';
import { type L1ContractAddresses } from '@aztec/ethereum';
import { type Logger } from '@aztec/foundation/log';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { AvailabilityOracleAbi, RollupAbi } from '@aztec/l1-artifacts';

import { type Anvil } from '@viem/anvil';
import { readFile } from 'fs/promises';
import { join } from 'path';
// @ts-expect-error solc-js doesn't publish its types https://github.com/ethereum/solc-js/issues/689
import solc from 'solc';
import {
  type Account,
  type Chain,
  type GetContractReturnType,
  type HttpTransport,
  type PublicClient,
  type WalletClient,
  getContract,
} from 'viem';
import { mnemonicToAccount } from 'viem/accounts';

import { MNEMONIC } from '../fixtures/fixtures.js';
import { getACVMConfig } from '../fixtures/get_acvm_config.js';
import { getBBConfig } from '../fixtures/get_bb_config.js';
import { getLogger, setupL1Contracts, startAnvil } from '../fixtures/utils.js';

/**
 * Regenerate this test's fixture with
 * AZTEC_GENERATE_TEST_DATA=1 yarn workspace @aztec/end-to-end test e2e_prover
 */
describe('proof_verification', () => {
  let proof: Proof;
  let block: L2Block;
  let aggregationObject: Fr[];
  let anvil: Anvil | undefined;
  let walletClient: WalletClient<HttpTransport, Chain, Account>;
  let publicClient: PublicClient<HttpTransport, Chain>;
  // eslint-disable-next-line
  let l1ContractAddresses: L1ContractAddresses;
  let logger: Logger;
  let circuitVerifier: BBCircuitVerifier;
  let bbTeardown: () => Promise<void>;
  let acvmTeardown: () => Promise<void>;
  let verifierContract: GetContractReturnType<any, typeof walletClient>;

  beforeAll(async () => {
    logger = getLogger();
    let rpcUrl = process.env.ETHEREUM_HOST;
    if (!rpcUrl) {
      ({ anvil, rpcUrl } = await startAnvil());
    }

    ({ l1ContractAddresses, publicClient, walletClient } = await setupL1Contracts(
      rpcUrl,
      mnemonicToAccount(MNEMONIC),
      logger,
    ));

    const bb = await getBBConfig(logger);
    const acvm = await getACVMConfig(logger);

    circuitVerifier = await BBCircuitVerifier.new({
      bbBinaryPath: bb!.bbBinaryPath,
      bbWorkingDirectory: bb!.bbWorkingDirectory,
    });

    bbTeardown = bb!.cleanup;
    acvmTeardown = acvm!.cleanup;

    const input = {
      language: 'Solidity',
      sources: {
        'UltraVerifier.sol': {
          content: await circuitVerifier.generateSolidityContract('RootRollupArtifact', 'UltraVerifier.sol'),
        },
      },
      settings: {
        // we require the optimizer
        optimizer: {
          enabled: true,
          runs: 200,
        },
        evmVersion: 'paris',
        outputSelection: {
          '*': {
            '*': ['evm.bytecode.object', 'abi'],
          },
        },
      },
    };

    const output = JSON.parse(solc.compile(JSON.stringify(input)));

    const abi = output.contracts['UltraVerifier.sol']['UltraVerifier'].abi;
    const bytecode: string = output.contracts['UltraVerifier.sol']['UltraVerifier'].evm.bytecode.object;

    const verifierAddress = await deployL1Contract(walletClient, publicClient, abi, `0x${bytecode}`);
    verifierContract = getContract({
      address: verifierAddress.toString(),
      client: publicClient,
      abi,
    }) as any;
  });

  afterAll(async () => {
    // await ctx.teardown();
    await anvil?.stop();
    await bbTeardown();
    await acvmTeardown();
  });

  beforeAll(async () => {
    // regenerate with
    // AZTEC_GENERATE_TEST_DATA=1 yarn workspace @aztec/end-to-end test e2e_prover
    const blockResult = JSON.parse(
      await readFile(join(fileURLToPath(import.meta.url), '../../fixtures/dumps/block_result.json'), 'utf-8'),
    );

    block = L2Block.fromString(blockResult.block);
    proof = Proof.fromString(blockResult.proof);
    aggregationObject = blockResult.aggregationObject.map((x: string) => Fr.fromString(x));
  });

  describe('bb', () => {
    it('verifies proof', async () => {
      await expect(circuitVerifier.verifyProofForCircuit('RootRollupArtifact', proof)).resolves.toBeUndefined();
    });
  });

  describe('UltraVerifier', () => {
    it('verifies full proof', async () => {
      const reader = BufferReader.asReader(proof.buffer);
      // +2 fields for archive
      const archive = reader.readArray(2, Fr);
      const header = reader.readArray(HEADER_LENGTH, Fr);
      const aggObject = reader.readArray(AGGREGATION_OBJECT_LENGTH, Fr);

      const publicInputs = [...archive, ...header, ...aggObject].map(x => x.toString());

      const proofStr = `0x${proof.buffer
        .subarray((HEADER_LENGTH + 2 + AGGREGATION_OBJECT_LENGTH) * Fr.SIZE_IN_BYTES)
        .toString('hex')}` as const;

      await expect(verifierContract.read.verify([proofStr, publicInputs])).resolves.toBeTruthy();
    });

    it('verifies proof taking public inputs from block', async () => {
      const proofStr = `0x${proof.withoutPublicInputs().toString('hex')}`;
      const publicInputs = [...block.archive.toFields(), ...block.header.toFields(), ...aggregationObject].map(x =>
        x.toString(),
      );

      await expect(verifierContract.read.verify([proofStr, publicInputs])).resolves.toBeTruthy();
    });
  });

  describe('Rollup', () => {
    let availabilityContract: GetContractReturnType<typeof AvailabilityOracleAbi, typeof walletClient>;
    let rollupContract: GetContractReturnType<typeof RollupAbi, typeof walletClient>;

    beforeAll(async () => {
      rollupContract = getContract({
        address: l1ContractAddresses.rollupAddress.toString(),
        abi: RollupAbi,
        client: walletClient,
      });

      availabilityContract = getContract({
        address: l1ContractAddresses.availabilityOracleAddress.toString(),
        abi: AvailabilityOracleAbi,
        client: walletClient,
      });

      await rollupContract.write.setVerifier([verifierContract.address]);
      logger.info('Rollup only accepts valid proofs now');
      await availabilityContract.write.publish([`0x${block.body.toBuffer().toString('hex')}`]);
    });

    it('verifies proof', async () => {
      const args = [
        `0x${block.header.toBuffer().toString('hex')}`,
        `0x${block.archive.root.toBuffer().toString('hex')}`,
        `0x${serializeToBuffer(aggregationObject).toString('hex')}`,
        `0x${proof.withoutPublicInputs().toString('hex')}`,
      ] as const;

      await expect(rollupContract.write.submitProof(args)).resolves.toBeDefined();
    });
  });
});
