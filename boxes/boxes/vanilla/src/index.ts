import { createPXEClient, AccountManager, Fr, Wallet, deriveMasterIncomingViewingSecretKey } from '@aztec/aztec.js';

import { SingleKeyAccountContract } from '@aztec/accounts/single_key';
import { VanillaContract } from '../artifacts/Vanilla';

const secretKey = Fr.random();
const pxe = createPXEClient(process.env.PXE_URL || 'http://localhost:8080');

const encryptionPrivateKey = deriveMasterIncomingViewingSecretKey(secretKey);
const account = new AccountManager(pxe, secretKey, new SingleKeyAccountContract(encryptionPrivateKey));
let contract: any = null;
let wallet: Wallet | null = null;

const setWait = (state: boolean): void =>
  document.querySelectorAll('*').forEach((e: HTMLElement & HTMLButtonElement) => {
    e.style.cursor = state ? 'wait' : 'default';
    e.disabled = state;
  });

document.querySelector('#deploy').addEventListener('click', async ({ target }: any) => {
  setWait(true);
  wallet = await account.register();
  const { masterNullifierPublicKey, masterIncomingViewingPublicKey, masterOutgoingViewingPublicKey } =
    wallet.getCompleteAddress().publicKeys;
  contract = await VanillaContract.deploy(
    wallet,
    Fr.random(),
    wallet.getCompleteAddress().address,
    masterNullifierPublicKey.hash(),
    masterOutgoingViewingPublicKey.toNoirStruct(),
    masterIncomingViewingPublicKey.toNoirStruct(),
  )
    .send({ contractAddressSalt: Fr.random() })
    .deployed();
  alert(`Contract deployed at ${contract.address}`);

  target.hidden = true;
  document.querySelectorAll('#get, #set').forEach((e: HTMLButtonElement & HTMLFormElement) => (e.hidden = false));
  setWait(false);
});

document.querySelector('#set').addEventListener('submit', async (e: Event) => {
  e.preventDefault();
  setWait(true);

  const { value } = document.querySelector('#number') as HTMLInputElement;
  const { address: owner, publicKeys } = wallet.getCompleteAddress();
  const { masterNullifierPublicKey, masterIncomingViewingPublicKey, masterOutgoingViewingPublicKey } = publicKeys;
  await contract.methods
    .setNumber(
      parseInt(value),
      owner,
      masterNullifierPublicKey.hash(),
      masterOutgoingViewingPublicKey.toNoirStruct(),
      masterIncomingViewingPublicKey.toNoirStruct(),
    )
    .send()
    .wait();

  setWait(false);
  alert('Number set!');
});

document.querySelector('#get').addEventListener('click', async () => {
  const viewTxReceipt = await contract.methods.getNumber(wallet.getCompleteAddress().address).simulate();
  alert(`Number is: ${viewTxReceipt.value}`);
});
