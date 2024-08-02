import { Fr } from '@aztec/circuits.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { type Command } from 'commander';

import {
  logJson,
  parseAztecAddress,
  parseEthereumAddress,
  parseField,
  parseFieldFromHexString,
  parseOptionalAztecAddress,
  parseOptionalInteger,
  parseOptionalLogId,
  parseOptionalTxHash,
  parsePartialAddress,
  parsePublicKey,
  parseTxHash,
  pxeOption,
} from '../../utils/commands.js';

export function injectCommands(program: Command, log: LogFn, debugLogger: DebugLogger) {
  program
    .command('add-contract')
    .description(
      'Adds an existing contract to the PXE. This is useful if you have deployed a contract outside of the PXE and want to use it with the PXE.',
    )
    .requiredOption(
      '-c, --contract-artifact <fileLocation>',
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts.js",
    )
    .requiredOption('-ca, --contract-address <address>', 'Aztec address of the contract.', parseAztecAddress)
    .requiredOption('--init-hash <init hash>', 'Initialization hash', parseFieldFromHexString)
    .option('--salt <salt>', 'Optional deployment salt', parseFieldFromHexString)
    .option('-p, --public-key <public key>', 'Optional public key for this contract', parsePublicKey)
    .option('--portal-address <address>', 'Optional address to a portal contract on L1', parseEthereumAddress)
    .option('--deployer-address <address>', 'Optional address of the contract deployer', parseAztecAddress)
    .addOption(pxeOption)
    .action(async options => {
      const { addContract } = await import('./add_contract.js');
      await addContract(
        options.rpcUrl,
        options.contractArtifact,
        options.contractAddress,
        options.initHash,
        options.salt ?? Fr.ZERO,
        options.publicKey,
        options.deployerAddress,
        debugLogger,
        log,
      );
    });

  program
    .command('get-tx')
    .description('Gets the receipt for the specified transaction hash.')
    .argument('<txHash>', 'A transaction hash to get the receipt for.', parseTxHash)
    .addOption(pxeOption)
    .action(async (txHash, options) => {
      const { getTx } = await import('./get_tx.js');
      await getTx(options.rpcUrl, txHash, debugLogger, log);
    });

  program
    .command('get-block')
    .description('Gets info for a given block or latest.')
    .argument('[blockNumber]', 'Block height', parseOptionalInteger)
    .option('-f, --follow', 'Keep polling for new blocks')
    .addOption(pxeOption)
    .action(async (blockNumber, options) => {
      const { getBlock } = await import('./get_block.js');
      await getBlock(options.rpcUrl, blockNumber, options.follow, debugLogger, log);
    });

  program
    .command('get-contract-data')
    .description('Gets information about the Aztec contract deployed at the specified address.')
    .argument('<contractAddress>', 'Aztec address of the contract.', parseAztecAddress)
    .addOption(pxeOption)
    .option('-b, --include-bytecode <boolean>', "Include the contract's public function bytecode, if any.", false)
    .action(async (contractAddress, options) => {
      const { getContractData } = await import('./get_contract_data.js');
      await getContractData(options.rpcUrl, contractAddress, options.includeBytecode, debugLogger, log);
    });

  program
    .command('get-logs')
    .description('Gets all the unencrypted logs from an intersection of all the filter params.')
    .option('-tx, --tx-hash <txHash>', 'A transaction hash to get the receipt for.', parseOptionalTxHash)
    .option(
      '-fb, --from-block <blockNum>',
      'Initial block number for getting logs (defaults to 1).',
      parseOptionalInteger,
    )
    .option('-tb, --to-block <blockNum>', 'Up to which block to fetch logs (defaults to latest).', parseOptionalInteger)
    .option('-al --after-log <logId>', 'ID of a log after which to fetch the logs.', parseOptionalLogId)
    .option('-ca, --contract-address <address>', 'Contract address to filter logs by.', parseOptionalAztecAddress)
    .addOption(pxeOption)
    .option('--follow', 'If set, will keep polling for new logs until interrupted.')
    .action(async ({ txHash, fromBlock, toBlock, afterLog, contractAddress, rpcUrl, follow }) => {
      const { getLogs } = await import('./get_logs.js');
      await getLogs(txHash, fromBlock, toBlock, afterLog, contractAddress, rpcUrl, follow, debugLogger, log);
    });

  program
    .command('register-recipient')
    .description('Register a recipient in the PXE.')
    .requiredOption('-a, --address <aztecAddress>', "The account's Aztec address.", parseAztecAddress)
    .requiredOption('-p, --public-key <publicKey>', 'The account public key.', parsePublicKey)
    .requiredOption(
      '-pa, --partial-address <partialAddress>',
      'The partially computed address of the account contract.',
      parsePartialAddress,
    )
    .addOption(pxeOption)
    .action(async ({ address, publicKey, partialAddress, rpcUrl }) => {
      const { registerRecipient } = await import('./register_recipient.js');
      await registerRecipient(address, publicKey, partialAddress, rpcUrl, debugLogger, log);
    });

  program
    .command('get-accounts')
    .description('Gets all the Aztec accounts stored in the PXE.')
    .addOption(pxeOption)
    .option('--json', 'Emit output as json')
    .action(async (options: any) => {
      const { getAccounts } = await import('./get_accounts.js');
      await getAccounts(options.rpcUrl, options.json, debugLogger, log, logJson(log));
    });

  program
    .command('get-account')
    .description('Gets an account given its Aztec address.')
    .argument('<address>', 'The Aztec address to get account for', parseAztecAddress)
    .addOption(pxeOption)
    .action(async (address, options) => {
      const { getAccount } = await import('./get_account.js');
      await getAccount(address, options.rpcUrl, debugLogger, log);
    });

  program
    .command('get-recipients')
    .description('Gets all the recipients stored in the PXE.')
    .addOption(pxeOption)
    .action(async (options: any) => {
      const { getRecipients } = await import('./get_recipients.js');
      await getRecipients(options.rpcUrl, debugLogger, log);
    });

  program
    .command('get-recipient')
    .description('Gets a recipient given its Aztec address.')
    .argument('<address>', 'The Aztec address to get recipient for', parseAztecAddress)
    .addOption(pxeOption)
    .action(async (address, options) => {
      const { getRecipient } = await import('./get_recipient.js');
      await getRecipient(address, options.rpcUrl, debugLogger, log);
    });

  program
    .command('call')
    .description(
      'Simulates the execution of a view (read-only) function on a deployed contract, without modifying state.',
    )
    .argument('<functionName>', 'Name of function to call')
    .option('-a, --args [functionArgs...]', 'Function arguments', [])
    .requiredOption(
      '-c, --contract-artifact <fileLocation>',
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts.js",
    )
    .requiredOption('-ca, --contract-address <address>', 'Aztec address of the contract.', parseAztecAddress)
    .option('-f, --from <string>', 'Aztec address of the caller. If empty, will use the first account from RPC.')
    .addOption(pxeOption)
    .action(async (functionName, options) => {
      const { call } = await import('./call.js');
      await call(
        functionName,
        options.args,
        options.contractArtifact,
        options.contractAddress,
        options.from,
        options.rpcUrl,
        debugLogger,
        log,
      );
    });

  program
    .command('add-note')
    .description('Adds a note to the database in the PXE.')
    .argument('<address>', 'The Aztec address of the note owner.', parseAztecAddress)
    .argument('<contractAddress>', 'Aztec address of the contract.', parseAztecAddress)
    .argument('<storageSlot>', 'The storage slot of the note.', parseField)
    .argument('<noteTypeId>', 'The type ID of the note.', parseField)
    .argument('<txHash>', 'The tx hash of the tx containing the note.', parseTxHash)
    .requiredOption('-n, --note [note...]', 'The members of a Note serialized as hex strings.', [])
    .addOption(pxeOption)
    .action(async (address, contractAddress, storageSlot, noteTypeId, txHash, options) => {
      const { addNote } = await import('./add_note.js');
      await addNote(
        address,
        contractAddress,
        storageSlot,
        noteTypeId,
        txHash,
        options.note,
        options.rpcUrl,
        debugLogger,
      );
    });

  program
    .command('block-number')
    .description('Gets the current Aztec L2 block number.')
    .addOption(pxeOption)
    .action(async (options: any) => {
      const { blockNumber } = await import('./block_number.js');
      await blockNumber(options.rpcUrl, debugLogger, log);
    });

  program
    .command('get-node-info')
    .description('Gets the information of an aztec node at a URL.')
    .addOption(pxeOption)
    .action(async options => {
      const { getNodeInfo } = await import('./get_node_info.js');
      await getNodeInfo(options.rpcUrl, debugLogger, log);
    });

  program
    .command('get-pxe-info')
    .description('Gets the information of a PXE at a URL.')
    .addOption(pxeOption)
    .action(async options => {
      const { getPXEInfo } = await import('./get_pxe_info.js');
      await getPXEInfo(options.rpcUrl, debugLogger, log);
    });

  return program;
}
