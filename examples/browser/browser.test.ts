import { test, expect } from '@playwright/test';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';
import * as http from 'http';
import * as fs from 'fs';
import * as path from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

let server: http.Server;
const SERVER_PORT = 8080;

// Simple static file server for pre-built files
function createStaticServer(rootDir: string): http.Server {
  return http.createServer((req, res) => {
    // Normalize and resolve the requested path to prevent directory traversal
    const requestedPath = req.url === '/' ? 'index.html' : req.url!.slice(1); // Remove leading slash
    const candidatePath = path.resolve(rootDir, requestedPath);

    let filePath: string;
    try {
      filePath = fs.realpathSync(candidatePath);
    } catch (_e) {
      res.writeHead(404);
      res.end('Not found');
      return;
    }
    // Ensure the resolved path is within the rootDir
    const rootRealPath = fs.realpathSync(rootDir);
    if (!filePath.startsWith(rootRealPath)) {
      res.writeHead(403);
      res.end('Forbidden');
      return;
    }

    fs.readFile(filePath, (err, data) => {
      if (err) {
        res.writeHead(404);
        res.end('Not found');
        return;
      }

      // Set appropriate content type
      const ext = path.extname(filePath);
      const contentType =
        {
          '.html': 'text/html',
          '.js': 'application/javascript',
          '.wasm': 'application/wasm',
        }[ext] || 'application/octet-stream';

      res.writeHead(200, { 'Content-Type': contentType });
      res.end(data);
    });
  });
}

test.describe('Noir Web App', () => {
  test.beforeAll(async () => {
    // Serve the pre-built files from the dist directory
    const distDir = resolve(__dirname, 'dist');

    // Start static file server for the already built files
    server = createStaticServer(distDir);
    await new Promise<void>((resolve) => {
      server.listen(SERVER_PORT, () => {
        console.log(`Static server running at http://localhost:${SERVER_PORT}`);
        resolve();
      });
    });
  });

  test.afterAll(async () => {
    // Close the server
    if (server) {
      await new Promise<void>((resolve) => {
        server.close(() => resolve());
      });
    }
  });

  test('should generate and verify proof for valid age', async ({ page }) => {
    // Increase test timeout as proof generation can take time
    test.setTimeout(30000);

    await page.goto(`http://localhost:${SERVER_PORT}`);
    await page.waitForLoadState('networkidle');

    // Wait for the page to load with increased timeout
    await expect(page.locator('h1')).toHaveText('Noir app', { timeout: 10000 });

    // Enter a valid age (greater than 18)
    await page.fill('#age', '25');
    await page.click('#submit');

    // Wait for witness generation
    await expect(page.locator('#logs')).toContainText('Generating witness... ⏳');
    await expect(page.locator('#logs')).toContainText('Generated witness... ✅');

    // Wait for proof generation (this can take longer)
    await expect(page.locator('#logs')).toContainText('Generating proof... ⏳');
    await expect(page.locator('#logs')).toContainText('Generated proof... ✅', { timeout: 20000 });

    // Wait for proof verification
    await expect(page.locator('#logs')).toContainText('Verifying proof... ⌛');
    await expect(page.locator('#logs')).toContainText('Proof is valid... ✅');

    // Check that proof is displayed
    const proofText = await page.locator('#results').textContent();
    expect(proofText).toContain('Proof');
    expect(proofText?.length).toBeGreaterThan(50); // Proof should be non-trivial
  });

  test('should fail for invalid age', async ({ page }) => {
    test.setTimeout(30000);

    await page.goto(`http://localhost:${SERVER_PORT}`);
    await page.waitForLoadState('networkidle');

    // Wait for the page to load with increased timeout
    await expect(page.locator('h1')).toHaveText('Noir app', { timeout: 10000 });

    // Enter an invalid age (less than or equal to 18)
    await page.fill('#age', '15');
    await page.click('#submit');

    // Should show error
    await expect(page.locator('#logs')).toContainText('Oh 💔');
  });
});
