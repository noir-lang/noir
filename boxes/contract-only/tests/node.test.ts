import { createPXEClient, PXE, GrumpkinScalar, AccountManager, Wallet } from '@aztec/aztec.js';
import { SingleKeyAccountContract } from '@aztec/accounts/single_key';
import { derivePublicKey } from '@aztec/circuits.js';

describe('Account Tests', () => {
  const pxeURL = process.env.PXE_URL || 'http://localhost:8080';
  let pxe: PXE;
  let account: AccountManager;
  let wallet: Wallet;

  const privateKey = GrumpkinScalar.fromString('0x1234');
  const expectedPublicKey = derivePublicKey(privateKey).toString();

  test('Can start the PXE server', async () => {
    pxe = createPXEClient(pxeURL);
    const { chainId } = await pxe.getNodeInfo();
    expect(chainId).toBe(31337);
  });

  beforeEach(() => {
    const accountContract = new SingleKeyAccountContract(privateKey);
    account = new AccountManager(pxe, privateKey, accountContract);
  });

  test('Can create an account contract with a known address', async () => {
    const publicKey = account.getCompleteAddress().publicKey.toString();
    expect(publicKey).toEqual(expectedPublicKey);
  });

  test('Can deploy a contract with a known address', async () => {
    ({ wallet } = await (await account.deploy()).wait());
    const publicKey = wallet.getCompleteAddress().publicKey.toString();
    expect(publicKey).toEqual(expectedPublicKey);
  });
});
