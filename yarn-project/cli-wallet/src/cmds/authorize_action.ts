import { type AccountWalletWithSecretKey, type AztecAddress, Contract } from '@aztec/aztec.js';
import { prepTx } from '@aztec/cli/utils';
import { type LogFn } from '@aztec/foundation/log';

export async function authorizeAction(
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

  if (isPrivate) {
    throw new Error(
      'Cannot authorize private function. To allow a third party to call a private function, please create an authorization witness via the create-authwit command',
    );
  }

  const contract = await Contract.at(contractAddress, contractArtifact, wallet);
  const action = contract.methods[functionName](...functionArgs);

  const witness = await wallet.setPublicAuthWit({ caller, action }, true).send().wait();

  log(`Authorized action ${functionName} on contract ${contractAddress} for caller ${caller}`);

  return witness;
}
