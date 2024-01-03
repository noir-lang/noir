/* eslint-disable no-console */
import * as AztecJs from '@aztec/aztec.js';
import { TokenContractArtifact } from '@aztec/noir-contracts/Token';

import { Server } from 'http';
import Koa from 'koa';
import serve from 'koa-static';
import path, { dirname } from 'path';
import { Browser, Page, launch } from 'puppeteer';

declare global {
  /**
   * Helper interface to declare aztec.js within browser context.
   */
  interface Window {
    /**
     * The aztec.js library.
     */
    AztecJs: typeof AztecJs;
  }
}

const __filename = AztecJs.fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const PORT = 3000;

const { PXE_URL } = process.env;

const conditionalDescribe = () => (PXE_URL ? describe : describe.skip);
const privKey = AztecJs.GrumpkinScalar.random();

export const browserTestSuite = (setup: () => Server, pageLogger: AztecJs.DebugLogger) =>
  conditionalDescribe()('e2e_aztec.js_browser', () => {
    const initialBalance = 33n;
    const transferAmount = 3n;

    let contractAddress: AztecJs.AztecAddress;

    let app: Koa;
    let testClient: AztecJs.PXE;
    let server: Server;

    let browser: Browser;
    let page: Page;

    beforeAll(async () => {
      server = setup();
      testClient = AztecJs.createPXEClient(PXE_URL!);
      await AztecJs.waitForSandbox(testClient);

      app = new Koa();
      app.use(serve(path.resolve(__dirname, './web')));

      browser = await launch({
        executablePath: process.env.CHROME_BIN,
        headless: 'new',
        args: [
          '--no-sandbox',
          '--headless',
          '--disable-gpu',
          '--disable-dev-shm-usage',
          '--disable-software-rasterizer',
          '--remote-debugging-port=9222',
        ],
      });
      page = await browser.newPage();
      page.on('console', msg => {
        pageLogger(msg.text());
      });
      page.on('pageerror', err => {
        pageLogger.error(err.toString());
      });
      await page.goto(`http://localhost:${PORT}/index.html`);
      while (!(await page.evaluate(() => !!window.AztecJs))) {
        pageLogger('Waiting for window.AztecJs...');
        await AztecJs.sleep(1000);
      }
    }, 120_000);

    afterAll(async () => {
      await browser.close();
      server.close();
    });

    it('Loads Aztec.js in the browser', async () => {
      const generatePublicKeyExists = await page.evaluate(() => {
        const { generatePublicKey } = window.AztecJs;
        return typeof generatePublicKey === 'function';
      });
      expect(generatePublicKeyExists).toBe(true);
    });

    it('Creates an account', async () => {
      const result = await page.evaluate(
        async (rpcUrl, privateKeyString) => {
          const { GrumpkinScalar, createPXEClient: createPXEClient, getUnsafeSchnorrAccount } = window.AztecJs;
          const pxe = createPXEClient(rpcUrl!);
          const privateKey = GrumpkinScalar.fromString(privateKeyString);
          const account = getUnsafeSchnorrAccount(pxe, privateKey);
          await account.waitDeploy();
          const completeAddress = account.getCompleteAddress();
          const addressString = completeAddress.address.toString();
          console.log(`Created Account: ${addressString}`);
          return addressString;
        },
        PXE_URL,
        privKey.toString(),
      );
      const accounts = await testClient.getRegisteredAccounts();
      const stringAccounts = accounts.map(acc => acc.address.toString());
      expect(stringAccounts.includes(result)).toBeTruthy();
    }, 15_000);

    it('Deploys Token contract', async () => {
      await deployTokenContract();
    }, 60_000);

    it('Can access CompleteAddress class in browser', async () => {
      const result: string = await page.evaluate(() => {
        const completeAddress = window.AztecJs.CompleteAddress.fromString(
          '0x115f123bbc6cc6af9890055821cfba23a7c4e8832377a32ccb719a1ba3a86483',
        );
        // NOTE: browser does not know how to serialize CompleteAddress for return, so return a string
        // otherwise returning a CompleteAddress makes result undefined.
        return completeAddress.toString();
      });
      // a lot of trailing 0s get added in the return value
      expect(result.slice(0, 66)).toBe('0x115f123bbc6cc6af9890055821cfba23a7c4e8832377a32ccb719a1ba3a86483');
    });

    it("Gets the owner's balance", async () => {
      const result = await page.evaluate(
        async (rpcUrl, contractAddress, TokenContractArtifact) => {
          const { Contract, AztecAddress, createPXEClient: createPXEClient } = window.AztecJs;
          const pxe = createPXEClient(rpcUrl!);
          const owner = (await pxe.getRegisteredAccounts())[0].address;
          const [wallet] = await AztecJs.getSandboxAccountsWallets(pxe);
          const contract = await Contract.at(AztecAddress.fromString(contractAddress), TokenContractArtifact, wallet);
          const balance = await contract.methods.balance_of_private(owner).view({ from: owner });
          return balance;
        },
        PXE_URL,
        (await getTokenAddress()).toString(),
        TokenContractArtifact,
      );
      expect(result).toEqual(initialBalance);
    });

    it('Sends a transfer TX', async () => {
      const result = await page.evaluate(
        async (rpcUrl, contractAddress, transferAmount, TokenContractArtifact) => {
          console.log(`Starting transfer tx`);
          const { AztecAddress, Contract, createPXEClient: createPXEClient } = window.AztecJs;
          const pxe = createPXEClient(rpcUrl!);
          const accounts = await pxe.getRegisteredAccounts();
          const receiver = accounts[1].address;
          const [wallet] = await AztecJs.getSandboxAccountsWallets(pxe);
          const contract = await Contract.at(AztecAddress.fromString(contractAddress), TokenContractArtifact, wallet);
          await contract.methods.transfer(accounts[0].address, receiver, transferAmount, 0).send().wait();
          console.log(`Transferred ${transferAmount} tokens to new Account`);
          return await contract.methods.balance_of_private(receiver).view({ from: receiver });
        },
        PXE_URL,
        (await getTokenAddress()).toString(),
        transferAmount,
        TokenContractArtifact,
      );
      expect(result).toEqual(transferAmount);
    }, 60_000);

    const deployTokenContract = async () => {
      const txHash = await page.evaluate(
        async (rpcUrl, privateKeyString, initialBalance, TokenContractArtifact) => {
          const {
            GrumpkinScalar,
            DeployMethod,
            createPXEClient,
            getUnsafeSchnorrAccount,
            Contract,
            Fr,
            ExtendedNote,
            Note,
            computeMessageSecretHash,
            getSandboxAccountsWallets,
          } = window.AztecJs;
          const pxe = createPXEClient(rpcUrl!);
          let accounts = await pxe.getRegisteredAccounts();
          if (accounts.length === 0) {
            // This test needs an account for deployment. We create one in case there is none available in the PXE.
            const privateKey = GrumpkinScalar.fromString(privateKeyString);
            await getUnsafeSchnorrAccount(pxe, privateKey).waitDeploy();
            accounts = await pxe.getRegisteredAccounts();
          }
          const [owner] = await getSandboxAccountsWallets(pxe);
          const ownerAddress = owner.getAddress();
          const tx = new DeployMethod(
            accounts[0].publicKey,
            pxe,
            TokenContractArtifact,
            a => Contract.at(a, TokenContractArtifact, owner),
            [owner.getCompleteAddress()],
          ).send();
          const { contract: token, txHash } = await tx.wait();

          console.log(`Contract Deployed: ${token.address}`);
          const secret = Fr.random();
          const secretHash = computeMessageSecretHash(secret);
          const mintPrivateReceipt = await token.methods.mint_private(initialBalance, secretHash).send().wait();

          const storageSlot = new Fr(5);
          const note = new Note([new Fr(initialBalance), secretHash]);
          const extendedNote = new ExtendedNote(
            note,
            ownerAddress,
            token.address,
            storageSlot,
            mintPrivateReceipt.txHash,
          );
          await pxe.addNote(extendedNote);

          await token.methods.redeem_shield(ownerAddress, initialBalance, secret).send().wait();

          return txHash.toString();
        },
        PXE_URL,
        privKey.toString(),
        initialBalance,
        TokenContractArtifact,
      );

      const txResult = await testClient.getTxReceipt(AztecJs.TxHash.fromString(txHash));
      expect(txResult.status).toEqual(AztecJs.TxStatus.MINED);
      contractAddress = txResult.contractAddress!;
    };

    const getTokenAddress = async () => {
      if (!contractAddress) {
        await deployTokenContract();
      }
      return contractAddress;
    };
  });
