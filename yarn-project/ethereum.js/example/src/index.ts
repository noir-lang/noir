import sourceMapSupport from 'source-map-support';
sourceMapSupport.install();
import { EthAddress } from '@aztec/foundation/eslint-legacy';
import { WalletProvider } from '@aztec/ethereum.js/provider';
import { EthereumRpc } from '@aztec/ethereum.js/eth_rpc';
import { fromBaseUnits, toBaseUnits } from '@aztec/ethereum.js/units';
import { ERC20Permit } from './contracts/ERC20Permit.js';
import { EthAccount } from '@aztec/ethereum.js/eth_account';
import { EthWallet } from '@aztec/ethereum.js/eth_wallet';
import { Contract, ContractAbi } from '@aztec/ethereum.js/contract';
import { RollupProcessorContract } from './contracts/RollupProcessorContract.js';
import { DaiContract } from './contracts/DaiContract.js';
import { ERC20Mintable } from './contracts/ERC20Mintable.js';
import { createPermitData } from './create_permit_data.js';

/**
 * Launch forked local chain e.g: `anvil -f https://mainnet.infura.io/v3/<api_key>`
 * Set ETHEREUM_HOST to e.g: `http://127.0.0.1:8545`.
 */
const { ETHEREUM_HOST } = process.env;

// Contract addresses.
const rollupProcessorAddr = EthAddress.fromString('0xFF1F2B4ADb9dF6FC8eAFecDcbF96A2B351680455');
const daiAddr = EthAddress.fromString('0x6B175474E89094C44Da98b954EedeAC495271d0F');

/**
 * A few examples of how to use ethereum.js.
 * The imported contracts are created by gen_def and provide complete type safety on methods, logs, receipts etc.
 * See `contracts.json` in project root, and run `yarn contract_gen_def` to rebuild contract definitions.
 */
async function main() {
  if (!ETHEREUM_HOST) {
    throw new Error('No ETHEREUM_HOST provided.');
  }

  // We could allow the ETHEREUM_HOST to do the signing if the host is something like anvil.
  // However, that's not very realistic, most hosts don't have accounts on them.
  // We construct a local wallet using the same mnemonic that anvil uses, and create a provider from it.
  // This means all signing happens locally before being sent to the ETHEREUM_HOST.
  const wallet = EthWallet.fromMnemonic('test test test test test test test test test test test junk', 2);
  const provider = WalletProvider.fromHost(ETHEREUM_HOST, wallet);
  const ethRpc = new EthereumRpc(provider);

  // Grab a couple of account addresses from our wallet.
  const [acc1, acc2] = wallet.accounts.map(a => a.address);

  console.log(`Chain Id: ${await ethRpc.getChainId()}`);
  console.log(`ETH balance of ${acc1}: ${fromBaseUnits(await ethRpc.getBalance(acc1), 18, 2)}`);
  console.log('');

  await demoUsefulReceiptError(ethRpc, acc1);
  await demoPrintingEventLogs(ethRpc);
  await demoDaiContractCalls(ethRpc);
  await demoGenericFunctionCall(ethRpc);
  await demoERC20DeployAndTransfer(ethRpc, acc1, acc2);
  await demoPermit(ethRpc, wallet, acc1, acc2);
  await demoEncryptDecryptWallet(wallet);
  demoSignMessage(wallet.accounts[0]);
}

/**
 * Demonstrate a failed tx receipt has a useful error message.
 */
async function demoUsefulReceiptError(ethRpc: EthereumRpc, acc1: EthAddress) {
  console.log('Demoing decoded errors on receipts, should see INVALID_PROVIDER...');
  const contract = new RollupProcessorContract(ethRpc, rollupProcessorAddr, { from: acc1, gas: 5000000 });
  const { 0: escapeHatchOpen, 1: until } = await contract.methods.getEscapeHatchStatus().call();

  if (!escapeHatchOpen) {
    // First we can do a call that will fail (or estimateGas). This will throw.
    try {
      await contract.methods.processRollup(Buffer.alloc(0), Buffer.alloc(0)).call();
    } catch (err: any) {
      console.log(`Call failed (expectedly) on RollupProcessor with: ${err.message}`);
    }

    // Second make an actual send, request not to throw on error, and explicitly check the receipt.
    const receipt = await contract.methods.processRollup(Buffer.alloc(0), Buffer.alloc(0)).send().getReceipt(false);
    if (receipt.error) {
      console.log(`Send receipt shows failure (expectedly) on RollupProcessor with: ${receipt.error.message}`);
    }
  } else {
    console.log(`Skipping until escape hatch closes in ${until} blocks.`);
  }
  console.log('');
}

/**
 *  Demonstrate printing some event logs from the rollup processor.
 */
async function demoPrintingEventLogs(ethRpc: EthereumRpc) {
  console.log('Demoing fetching RollupProcessed event logs from last 1000 blocks...');
  const contract = new RollupProcessorContract(ethRpc, rollupProcessorAddr);
  const blockNumber = await ethRpc.blockNumber();
  const events = await contract.getLogs('RollupProcessed', { fromBlock: blockNumber - 1000 });
  events.forEach(e =>
    console.log(
      `Rollup ${e.args.rollupId}: from: ${e.args.sender}, defiHashes: ${e.args.nextExpectedDefiHashes.length}`,
    ),
  );
  console.log('');
}

/**
 * Get Dai balance of rollup processor.
 * Doesn't really need the DaiContract explicitly to do this, but there are other methods unique to Dai.
 */
async function demoDaiContractCalls(ethRpc: EthereumRpc) {
  console.log('Demoing DAI contract calls...');
  const contract = new DaiContract(ethRpc, daiAddr);
  const balance = await contract.methods.balanceOf(rollupProcessorAddr).call();
  console.log(`DAI contract version: ${await contract.methods.version().call()}`);
  console.log(`DAI Balance of ${rollupProcessorAddr}: ${fromBaseUnits(balance, 18, 2)}`);
  console.log('');
}

/**
 * Demonstrates calling a function not present on the abi.
 */
async function demoGenericFunctionCall(ethRpc: EthereumRpc) {
  console.log('Demoing generic function contract calls...');
  const contract = new Contract(ethRpc, new ContractAbi([]), daiAddr);
  const balance = await contract.getMethod('balanceOf', ['address'], ['uint'])(rollupProcessorAddr).call();
  console.log(`DAI Balance of ${rollupProcessorAddr}: ${fromBaseUnits(balance, 18, 2)}`);
  console.log('');
}

/**
 * Deploy an ERC20 and do a transfer.
 */
async function demoERC20DeployAndTransfer(ethRpc: EthereumRpc, acc1: EthAddress, acc2: EthAddress) {
  console.log('Demoing ERC20 deployment, minting, transfer and log handling...');
  const contract = new ERC20Mintable(ethRpc, undefined, { from: acc1, gas: 1000000 });
  const symbol = 'AZT';
  await contract.deploy(symbol).send().getReceipt();
  console.log(`Deployed ERC20 with symbol: ${await contract.methods.symbol().call()}`);

  console.log(`Transferring from ${acc1} to ${acc2}`);
  await contract.methods.mint(acc1, toBaseUnits('1000', 18)).send().getReceipt();
  console.log(`Balance of ${acc1}: ${fromBaseUnits(await contract.methods.balanceOf(acc1).call(), 18)}`);

  const receipt = await contract.methods.transfer(acc2, toBaseUnits('0.1', 18)).send().getReceipt();
  const [{ args }] = receipt.events.Transfer;
  if (args) {
    console.log(`Log shows transfer of ${args.value} from ${args.from} to ${args.to}`);
  }
  console.log(`${symbol} balance of ${acc1}: ${fromBaseUnits(await contract.methods.balanceOf(acc1).call(), 18)}`);
  console.log(`${symbol} balance of ${acc2}: ${fromBaseUnits(await contract.methods.balanceOf(acc2).call(), 18)}`);
  console.log('');
}

/**
 * Lets use permit to demonstrate signing typed data. Normally one wouldn't call permit as a tx, as it's meant
 * to be used within another tx to save on gas. Here we'll just check it correctly updates the allowance.
 */
async function demoPermit(ethRpc: EthereumRpc, wallet: EthWallet, acc1: EthAddress, acc2: EthAddress) {
  console.log('Demoing signing typed data by using permit to increase allowance...');
  const contract = new ERC20Permit(ethRpc, undefined, { from: acc1, gas: 2000000 });
  const symbol = 'AZT';
  await contract.deploy(symbol).send().getReceipt();
  await contract.methods.mint(acc1, toBaseUnits('1000', 18)).send().getReceipt();

  console.log(`Allowance of ${acc2} to transfer from ${acc1}: ${await contract.methods.allowance(acc1, acc2).call()}`);
  const deadline = BigInt(Math.floor(Date.now() / 1000) + 5 * 60);
  const nonce = await contract.methods.nonces(acc1).call();
  const chainId = await ethRpc.getChainId();
  const permitData = createPermitData(symbol, acc1, acc2, 10n, nonce, deadline, contract.address, chainId);
  const sig = wallet.accounts[0].signTypedData(permitData);
  await contract.methods.permit(acc1, acc2, 10n, deadline, sig.v, sig.r, sig.s).send().getReceipt();
  console.log(`Allowance of ${acc2} to transfer from ${acc1}: ${await contract.methods.allowance(acc1, acc2).call()}`);

  // We can also pass the typed data to the ETHEREUM_HOST to sign.
  // In this case it's intercepted by the WalletProvider.
  const sig2 = await ethRpc.signTypedDataV4(acc1, permitData);
  console.log(`Direct sign vs provider sign equality check: ${sig.toString() === sig2.toString()}`);
  console.log('');
}

/**
 * Demonstrates encrypting a wallet to KeyStoreJson (which could be written to a file), and restoring it.
 */
async function demoEncryptDecryptWallet(wallet: EthWallet) {
  console.log('Demoing wallet encryption and decryption...');
  console.log(`Encrypting wallet with ${wallet.length} accounts...`);
  const password = 'mypassword';
  const encryptedWallet = await wallet.encrypt(password);
  const decryptedWallet = await EthWallet.fromKeystores(encryptedWallet, password);

  console.log(`Decrypted wallet has ${decryptedWallet.length} accounts:`);
  wallet.accounts.map(a => console.log(a.address.toString()));
  console.log('');
}

/**
 * Demonstrates signing a message and verifying signer.
 */
function demoSignMessage(signingAccount: EthAccount) {
  console.log('Demoing signing a message locally and recovering the signer...');

  // Sign a message.
  console.log(`Signing message with address: ${signingAccount.address}`);
  const msg = Buffer.from('My signed text');
  const sig = signingAccount.signMessage(msg);

  // Verify message was signed by account.
  if (signingAccount.signedMessage(msg, sig)) {
    console.log(`Message was signed by: ${signingAccount.address}`);
  } else {
    console.log(`Message was NOT signed by: ${signingAccount.address}`);
  }
  console.log('');
}

main().catch(console.error);
