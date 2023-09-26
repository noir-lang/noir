import {
  AccountWallet,
  AztecAddress,
  AztecRPC,
  CompleteAddress,
  Contract,
  DeployMethod,
  Fr,
  TxReceipt,
  createAztecRpcClient,
  getSandboxAccountsWallets,
} from '@aztec/aztec.js';
import { ContractAbi, FunctionAbi, encodeArguments } from '@aztec/foundation/abi';
import { FieldsOf } from '@aztec/foundation/types';
import { BlankContractAbi } from './artifacts/blank.js';
export const contractAbi: ContractAbi = BlankContractAbi;

export const SANDBOX_URL: string = process.env.SANDBOX_URL || 'http://localhost:8080';
export const rpcClient: AztecRPC = createAztecRpcClient(SANDBOX_URL);

export const CONTRACT_ADDRESS_PARAM_NAMES = ['owner', 'contract_address', 'recipient'];
export const FILTERED_FUNCTION_NAMES = [];

export const DEFAULT_PUBLIC_ADDRESS: string = '0x25048e8c1b7dea68053d597ac2d920637c99523651edfb123d0632da785970d0';

let contractAddress: string = '';

// interaction with the buttons, but conditional check so node env can also import from this file
if (typeof document !== 'undefined') {
  document.getElementById('deploy')?.addEventListener('click', async () => {
    contractAddress = await handleDeployClick();
    console.log('Deploy Succeeded, contract deployed at', contractAddress);
  });

  document.getElementById('interact')?.addEventListener('click', async () => {
    const interactionResult = await handleInteractClick(contractAddress);
    console.log('Interaction transaction succeeded', interactionResult);
  });
}

export async function handleDeployClick(): Promise<string> {
  console.log('Deploying Contract');
  const [wallet, ..._rest] = await getSandboxAccountsWallets(rpcClient);

  const contractAztecAddress = await deployContract(
    wallet.getCompleteAddress(),
    contractAbi,
    [],
    Fr.random(),
    rpcClient,
  );

  return contractAztecAddress.toString();
}

export async function handleInteractClick(contractAddress: string) {
  const [wallet, ..._rest] = await getSandboxAccountsWallets(rpcClient);
  const callArgs = { address: wallet.getCompleteAddress().address };
  const getPkAbi = getFunctionAbi(BlankContractAbi, 'getPublicKey');
  const typedArgs = convertArgs(getPkAbi, callArgs);
  console.log('Interacting with Contract');

  return await callContractFunction(
    AztecAddress.fromString(contractAddress),
    contractAbi,
    'getPublicKey',
    typedArgs,
    rpcClient,
    wallet.getCompleteAddress(),
  );
}

export const getFunctionAbi = (contractAbi: any, functionName: string) => {
  const functionAbi = contractAbi.functions.find((f: FunctionAbi) => f.name === functionName);
  if (!functionAbi) throw new Error(`Function ${functionName} not found in abi`);
  return functionAbi;
};

export async function callContractFunction(
  address: AztecAddress,
  abi: ContractAbi,
  functionName: string,
  typedArgs: any[], // for the exposed functions, this is an array of field elements Fr[]
  rpc: AztecRPC,
  wallet: CompleteAddress,
): Promise<FieldsOf<TxReceipt>> {
  // selectedWallet is how we specify the "sender" of the transaction
  const selectedWallet = await getWallet(wallet, rpc);

  // TODO: switch to the generated typescript class?
  const contract = await Contract.at(address, abi, selectedWallet);

  return contract.methods[functionName](...typedArgs)
    .send()
    .wait();
}

/**
 * terminology is confusing, but the `account` points to a smart contract's public key information
 * while the "wallet" has the account's private key and is used to sign transactions
 * we need the "wallet" to actually submit transactions using the "account" identity
 * @param account
 * @param rpc
 * @returns
 */
export async function getWallet(account: CompleteAddress, rpc: AztecRPC): Promise<AccountWallet> {
  const accountWallets: AccountWallet[] = await getSandboxAccountsWallets(rpc);
  const selectedWallet: AccountWallet = accountWallets.find(w => w.getAddress().equals(account.address))!;
  if (!selectedWallet) {
    throw new Error(`Wallet for account ${account.address.toShortString()} not found in the RPC server.`);
  }
  return selectedWallet;
}

export async function deployContract(
  activeWallet: CompleteAddress,
  contractAbi: ContractAbi,
  typedArgs: Fr[], // encode prior to passing in
  salt: Fr,
  client: AztecRPC,
): Promise<AztecAddress> {
  const tx = new DeployMethod(activeWallet.publicKey, client, contractAbi, typedArgs).send({
    contractAddressSalt: salt,
  });
  await tx.wait();
  const receipt = await tx.getReceipt();
  if (receipt.contractAddress) {
    return receipt.contractAddress;
  } else {
    throw new Error(`Contract not deployed (${receipt.toJSON()})`);
  }
}

export function convertArgs(functionAbi: FunctionAbi, args: any): Fr[] {
  const untypedArgs = functionAbi.parameters.map(param => {
    switch (param.type.kind) {
      case 'field':
        // hack: addresses are stored as string in the form to avoid bigint compatibility issues with formik
        // convert those back to bigints before turning into Fr
        return BigInt(args[param.name]);
      default:
        // need more testing on other types
        return args[param.name];
    }
  });

  return encodeArguments(functionAbi, untypedArgs);
}
