import { test, expect } from '@playwright/test';

test('test', async ({ page }) => {
  test.slow();
  await page.goto('/');

  // Deploy contract
  await page.getByRole('button', { name: 'Deploy dummy contract' }).click();
  await expect(page.getByText('Deploying contract...')).toBeVisible();
  await expect(page.getByText('Address:')).toBeVisible();

  // Read number
  await page.getByRole('button', { name: 'Read' }).click();
  await expect(page.getByText('Number is:')).toBeVisible();

  // Set number
  await page.locator('#numberToSet').fill('1');
  await page.getByRole('button', { name: 'Write' }).click();
  await expect(page.getByText('Setting number...')).toBeVisible();
  await expect(page.getByText('Number set to: 1')).toBeVisible();

  // Read number
  await page.getByRole('button', { name: 'Read' }).click();
  await expect(page.getByText('Number is: 1')).toBeVisible();
});
