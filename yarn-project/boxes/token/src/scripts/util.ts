import { AccountWallet, Fr, getSandboxAccountsWallets } from '@aztec/aztec.js';
import { FunctionArtifact, encodeArguments } from '@aztec/foundation/abi';
import { CompleteAddress, PXE } from '@aztec/types';

function convertBasicArg(paramType: string, value: any) {
  switch (paramType) {
    case 'field':
      // hack: addresses are stored as string in the form to avoid bigint compatibility issues with formik
      // convert those back to bigints before turning into Fr
      return BigInt(value);
    default:
      return value;
  }
}

export function convertArgs(functionAbi: FunctionArtifact, args: any): Fr[] {
  const untypedArgs = functionAbi.parameters
    .map(param => {
      if (['field', 'array', 'boolean'].includes(param.type.kind)) {
        return convertBasicArg(param.type.kind, args[param.name]);
      } else if (param.type.kind === 'struct') {
        const structParams = param.type.fields;
        // struct an object for the struct input type
        const structArgs = [];
        for (const field of structParams) {
          structArgs.push(convertBasicArg(field.type.kind, args[param.name][field.name]));
        }
        return structArgs;
      }
    })
    .flat();

  return encodeArguments(functionAbi, untypedArgs);
}

/**
 * terminology is confusing, but the `account` points to a smart contract's public key information
 * while the "wallet" has the account's private key and is used to sign transactions
 * we need the "wallet" to actually submit transactions using the "account" identity
 * @param account
 * @param pxe
 * @returns
 */
export async function getWallet(account: CompleteAddress, pxe: PXE): Promise<AccountWallet> {
  const accountWallets: AccountWallet[] = await getSandboxAccountsWallets(pxe);
  const selectedWallet: AccountWallet = accountWallets.find(w => w.getAddress().equals(account.address))!;
  if (!selectedWallet) {
    throw new Error(`Wallet for account ${account.address.toShortString()} not found in the PXE.`);
  }
  return selectedWallet;
}
