#!/usr/bin/env node
import { createLogger } from '@aztec/foundation/log';
import { Command } from 'commander';
import { mnemonicToAccount, privateKeyToAccount } from 'viem/accounts';
import { createDebugLogger } from '@aztec/foundation/log';
import { createEthereumChain, deployL1Contracts } from '@aztec/ethereum';
import { deployL2Contract } from './deploy_l2_contract.js';

const logger = createDebugLogger('aztec:cli');

const program = new Command();
const log = createLogger('aztec:aztec-cli');

/**
 * Function to execute the 'deployRollupContracts' command.
 * @param rpcUrl - The RPC URL of the ethereum node.
 * @param apiKey - The api key of the ethereum node endpoint.
 * @param privateKey - The private key to be used in contract deployment.
 * @param mnemonic - The mnemonic to be used in contract deployment.
 */
async function deployRollupContracts(rpcUrl: string, apiKey: string, privateKey: string, mnemonic: string) {
  const account = !privateKey ? mnemonicToAccount(mnemonic!) : privateKeyToAccount(`0x${privateKey}`);
  const chain = createEthereumChain(rpcUrl, apiKey);
  await deployL1Contracts(chain.rpcUrl, account, chain.chainInfo, logger);
}

/**
 * A placeholder for the Aztec-cli.
 */
async function main() {
  program
    .command('run')
    .argument('<cmd>', 'Command')
    .action((cmd: string) => {
      log(`Running '${cmd}'...`);
    });

  program
    .command('deployRollupContracts')
    .argument(
      '[rpcUrl]',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      'http://localhost:8545',
    )
    .option('-a, --apiKey <string>', 'Api key for the ethereum host', undefined)
    .option('-p, --privateKey <string>', 'The private key to use for deployment')
    .option(
      '-m, --mnemonic <string>',
      'The mnemonic to use in deployment',
      'test test test test test test test test test test test junk',
    )
    .action(async (rpcUrl: string, options) => {
      await deployRollupContracts(rpcUrl, options.apiKey ?? '', options.privateKey, options.mnemonic);
    });

  program
    .command('deployL2')
    .argument('[rpcUrl]', 'Url of the rollup provider', 'http://localhost:9000')
    .argument('[interval]', 'Interval between contract deployments (seconds), 0 means only a single deployment', 60)
    .action(async (rpcUrl: string, intervalArg: string) => {
      try {
        const interval = Number(intervalArg);
        await deployL2Contract(rpcUrl, interval * 1000, logger);
      } catch (err) {
        logger(`Error`, err);
      }
    });

  await program.parseAsync(process.argv);
}

main().catch(err => {
  log(`Error thrown: ${err}`);
  process.exit(1);
});
