import { type AccountWalletWithSecretKey, type AztecAddress, Contract } from '@aztec/aztec.js';
import { prepTx } from '@aztec/cli/utils';
import { type LogFn } from '@aztec/foundation/log';

export async function createAuthwit(
  wallet: AccountWalletWithSecretKey,
  functionName: string,
  caller: AztecAddress,
  functionArgsIn: any[],
  contractArtifactPath: string,
  contractAddress: AztecAddress,
  log: LogFn,
) {
  const { functionArgs, contractArtifact, isPrivate } = await prepTx(
    contractArtifactPath,
    functionName,
    functionArgsIn,
    log,
  );

  if (!isPrivate) {
    throw new Error(
      'Cannot create an authwit for a public function. To allow a third party to call a public function, please authorize the action via the authorize-action command',
    );
  }

  const contract = await Contract.at(contractAddress, contractArtifact, wallet);
  const action = contract.methods[functionName](...functionArgs);

  const witness = await wallet.createAuthWit({ caller, action });

  log(`Created authorization witness for action ${functionName} on contract ${contractAddress} for caller ${caller}`);

  return witness;
}
