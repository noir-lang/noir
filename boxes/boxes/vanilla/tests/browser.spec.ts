import { test, expect } from '@playwright/test';

test('Deploying, setting, and getting a number', async ({ page }) => {
  test.slow();
  await page.goto('/');

  const handleDialog = (expectedMessage: string) => {
    return new Promise<void>(resolve => {
      page.once('dialog', async dialog => {
        expect(dialog.message()).toContain(expectedMessage);
        await dialog.accept();
        resolve();
      });
    });
  };

  // Deploy contract
  const deployDialogPromise = handleDialog('Contract deployed at');
  await page.getByRole('button', { name: 'Deploy' }).click();
  await deployDialogPromise;
  await expect(page.locator('#number')).toHaveValue('0');

  // Get number
  const getNumberDialogPromise = handleDialog('Number is:');
  await page.getByRole('button', { name: 'Get Number' }).click();
  await getNumberDialogPromise;

  // Set number
  await page.locator('#number').fill('1');
  const setNumberDialogPromise = handleDialog('Number set!');
  await page.getByRole('button', { name: 'Set Number' }).click();
  await setNumberDialogPromise;

  // Verifying number
  const verifyNumberDialogPromise = handleDialog('Number is: 1');
  await page.getByRole('button', { name: 'Get Number' }).click();
  await verifyNumberDialogPromise;
});
