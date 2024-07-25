import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { BatchCall, type PXE, type Wallet } from '@aztec/aztec.js';
import { type AztecAddress, type EthAddress, Fq, Fr } from '@aztec/circuits.js';
import {
  type ContractArtifacts,
  type L1Clients,
  createEthereumChain,
  createL1Clients,
  deployL1Contract,
} from '@aztec/ethereum';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';
import { PortalERC20Abi, PortalERC20Bytecode, TokenPortalAbi, TokenPortalBytecode } from '@aztec/l1-artifacts';

import { getContract } from 'viem';
import { privateKeyToAccount } from 'viem/accounts';

import { createCompatibleClient } from '../../client.js';

export async function bootstrapDevnet(
  pxeUrl: string,
  l1Url: string,
  l1ChainId: string,
  l1PrivateKey: `0x${string}` | undefined,
  l1Mnemonic: string,
  json: boolean,
  log: LogFn,
  debugLog: DebugLogger,
) {
  const pxe = await createCompatibleClient(pxeUrl, debugLog);

  // setup a one-off account contract
  const account = getSchnorrAccount(pxe, Fr.random(), Fq.random(), Fr.random());
  const wallet = await account.deploy().getWallet();

  const l1Clients = createL1Clients(
    l1Url,
    l1PrivateKey ? privateKeyToAccount(l1PrivateKey) : l1Mnemonic,
    createEthereumChain(l1Url, +l1ChainId).chainInfo,
  );

  const { erc20Address, portalAddress } = await deployERC20(l1Clients);
  const { tokenAddress, bridgeAddress } = await deployToken(wallet, portalAddress);
  await initPortal(pxe, l1Clients, erc20Address, portalAddress, bridgeAddress);

  const fpcAddress = await deployFPC(wallet, tokenAddress);
  const counterAddress = await deployCounter(wallet);

  if (json) {
    log(
      JSON.stringify(
        {
          devCoinL1: erc20Address.toString(),
          devCoinPortalL1: portalAddress.toString(),
          devCoinL2: tokenAddress.toString(),
          devCoinBridgeL2: bridgeAddress.toString(),
          devCoinFpcL2: fpcAddress.toString(),
          counterL2: counterAddress.toString(),
        },
        null,
        2,
      ),
    );
  } else {
    log(`DevCoin L1: ${erc20Address}`);
    log(`DevCoin L1 Portal: ${portalAddress}`);
    log(`DevCoin L2: ${tokenAddress}`);
    log(`DevCoin L2 Bridge: ${bridgeAddress}`);
    log(`DevCoin FPC: ${fpcAddress}`);
    log(`Counter: ${counterAddress}`);
  }
}

/**
 * Step 1. Deploy the L1 contracts, but don't initialize
 */
async function deployERC20({ walletClient, publicClient }: L1Clients) {
  const erc20: ContractArtifacts = {
    contractAbi: PortalERC20Abi,
    contractBytecode: PortalERC20Bytecode,
  };
  const portal: ContractArtifacts = {
    contractAbi: TokenPortalAbi,
    contractBytecode: TokenPortalBytecode,
  };

  const erc20Address = await deployL1Contract(
    walletClient,
    publicClient,
    erc20.contractAbi,
    erc20.contractBytecode,
    [],
  );
  const portalAddress = await deployL1Contract(
    walletClient,
    publicClient,
    portal.contractAbi,
    portal.contractBytecode,
    [],
  );

  return {
    erc20Address,
    portalAddress,
  };
}

/**
 * Step 2. Deploy the L2 contracts
 */
async function deployToken(
  wallet: Wallet,
  l1Portal: EthAddress,
): Promise<{ tokenAddress: AztecAddress; bridgeAddress: AztecAddress }> {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore - Importing noir-contracts.js even in devDeps results in a circular dependency error. Need to ignore because this line doesn't cause an error in a dev environment
  const { TokenContract, TokenBridgeContract } = await import('@aztec/noir-contracts.js');
  const devCoin = await TokenContract.deploy(wallet, wallet.getAddress(), 'DevCoin', 'DEV', 18).send().deployed();
  const bridge = await TokenBridgeContract.deploy(wallet, devCoin.address, l1Portal).send().deployed();

  await new BatchCall(wallet, [
    devCoin.methods.set_minter(bridge.address, true).request(),
    devCoin.methods.set_admin(bridge.address).request(),
  ])
    .send()
    .wait();

  return {
    tokenAddress: devCoin.address,
    bridgeAddress: bridge.address,
  };
}

/**
 * Step 3. Initialize DevCoin's L1 portal
 */
async function initPortal(
  pxe: PXE,
  { walletClient, publicClient }: L1Clients,
  erc20: EthAddress,
  portal: EthAddress,
  bridge: AztecAddress,
) {
  const {
    l1ContractAddresses: { registryAddress },
  } = await pxe.getNodeInfo();

  const contract = getContract({
    abi: TokenPortalAbi,
    address: portal.toString(),
    client: walletClient,
  });

  const hash = await contract.write.initialize([registryAddress.toString(), erc20.toString(), bridge.toString()]);
  await publicClient.waitForTransactionReceipt({ hash });
}

async function deployFPC(wallet: Wallet, tokenAddress: AztecAddress): Promise<AztecAddress> {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore - Importing noir-contracts.js even in devDeps results in a circular dependency error. Need to ignore because this line doesn't cause an error in a dev environment
  const { FPCContract } = await import('@aztec/noir-contracts.js');
  const {
    protocolContractAddresses: { gasToken },
  } = await wallet.getPXEInfo();
  const fpc = await FPCContract.deploy(wallet, tokenAddress, gasToken).send().deployed();
  return fpc.address;
}

async function deployCounter(wallet: Wallet): Promise<AztecAddress> {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore - Importing noir-contracts.js even in devDeps results in a circular dependency error. Need to ignore because this line doesn't cause an error in a dev environment
  const { CounterContract } = await import('@aztec/noir-contracts.js');
  const counter = await CounterContract.deploy(wallet, 1, wallet.getAddress(), wallet.getAddress()).send().deployed();
  return counter.address;
}
