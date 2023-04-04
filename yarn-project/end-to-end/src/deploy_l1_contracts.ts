import { EthereumRpc } from '@aztec/ethereum.js/eth_rpc';
import { WalletProvider } from '@aztec/ethereum.js/provider';
import { Rollup, UnverifiedDataEmitter } from '@aztec/l1-contracts';

export const deployRollupContract = async (provider: WalletProvider, ethRpc: EthereumRpc) => {
  const deployAccount = provider.getAccount(0);
  const contract = new Rollup(ethRpc, undefined, { from: deployAccount, gas: 1000000 });
  await contract.deploy().send().getReceipt();
  return contract.address;
};

export const deployUnverifiedDataEmitterContract = async (provider: WalletProvider, ethRpc: EthereumRpc) => {
  const deployAccount = provider.getAccount(0);
  const contract = new UnverifiedDataEmitter(ethRpc, undefined, { from: deployAccount, gas: 1000000 });
  await contract.deploy().send().getReceipt();
  return contract.address;
};

export const createProvider = (host: string, mnemonic: string, accounts: number) => {
  const walletProvider = WalletProvider.fromHost(host);
  walletProvider.addAccountsFromMnemonic(mnemonic, accounts);
  return walletProvider;
};
