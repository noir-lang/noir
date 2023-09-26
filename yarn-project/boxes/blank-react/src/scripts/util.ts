import { AccountWallet, Fr, getSandboxAccountsWallets } from '@aztec/aztec.js';
import { FunctionAbi, encodeArguments } from '@aztec/foundation/abi';
import { AztecRPC, CompleteAddress } from '@aztec/types';

export function convertArgs(functionAbi: FunctionAbi, args: any): Fr[] {
  const untypedArgs = functionAbi.parameters.map(param => {
    switch (param.type.kind) {
      case 'field':
        // hack: addresses are stored as string in the form to avoid bigint compatibility issues with formik
        // convert those back to bigints before turning into Fr
        return BigInt(args[param.name]);
      default:
        // they are all fields in the privatetoken contract, need more testing on other types
        return args[param.name];
    }
  });

  return encodeArguments(functionAbi, untypedArgs);
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
