import { test, expect } from '@playwright/test';
import { readFileSync } from 'fs';
import { join, resolve } from 'path';
import * as http from 'http';
import * as fs from 'fs';
import * as path from 'path';

let server: http.Server;
const SERVER_PORT = 8080;

// Simple static file server for pre-built files
function createStaticServer(rootDir: string): http.Server {
  return http.createServer((req, res) => {
    const filePath = path.join(rootDir, req.url === '/' ? 'index.html' : req.url!);

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
    const fixtureDir = resolve(__dirname, 'fixtures/browser');
    const distDir = join(fixtureDir, 'dist');

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

    // Wait for the page to load
    await expect(page.locator('h1')).toHaveText('Noir app');

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
    await page.goto(`http://localhost:${SERVER_PORT}`);

    // Wait for the page to load
    await expect(page.locator('h1')).toHaveText('Noir app');

    // Enter an invalid age (less than or equal to 18)
    await page.fill('#age', '15');
    await page.click('#submit');

    // Should show error
    await expect(page.locator('#logs')).toContainText('Oh 💔');
  });
});

// Test to verify that the code snippets match what's documented
test.describe('Code Snippets', () => {
  test('verify HTML structure matches documentation', () => {
    const htmlPath = join(__dirname, 'fixtures/browser/index.html');
    const htmlContent = readFileSync(htmlPath, 'utf-8');

    // Verify essential HTML elements are present
    expect(htmlContent).toContain('<input id="age" type="number" placeholder="Enter age" />');
    expect(htmlContent).toContain('<button id="submit">Submit Age</button>');
    expect(htmlContent).toContain('<div id="logs" class="inner"><h2>Logs</h2></div>');
    expect(htmlContent).toContain('<div id="results" class="inner"><h2>Proof</h2></div>');
    expect(htmlContent).toContain('<!-- docs:start:index -->');
    expect(htmlContent).toContain('<!-- docs:end:index -->');
  });

  test('verify JavaScript imports and structure', () => {
    const jsPath = join(__dirname, 'fixtures/browser/index.js');
    const jsContent = readFileSync(jsPath, 'utf-8');

    // Verify imports
    expect(jsContent).toContain("import { UltraHonkBackend } from '@aztec/bb.js';");
    expect(jsContent).toContain("import { Noir } from '@noir-lang/noir_js';");

    // Verify key functionality
    expect(jsContent).toContain('new Noir(circuit)');
    expect(jsContent).toContain('new UltraHonkBackend(circuit.bytecode)');
    expect(jsContent).toContain('await noir.execute({ age })');
    expect(jsContent).toContain('await backend.generateProof(witness)');
    expect(jsContent).toContain('await backend.verifyProof(proof)');

    // Verify docs markers
    expect(jsContent).toContain('// docs:start:imports');
    expect(jsContent).toContain('// docs:end:imports');
    expect(jsContent).toContain('// docs:start:show_function');
    expect(jsContent).toContain('// docs:end:show_function');
  });

  test('verify Noir circuit', () => {
    const nrPath = join(__dirname, 'fixtures/browser/src/main.nr');
    const nrContent = readFileSync(nrPath, 'utf-8');

    expect(nrContent).toContain('fn main(age: u8) {');
    expect(nrContent).toContain('assert(age > 18);');
    expect(nrContent).toContain('// docs:start:age_check');
    expect(nrContent).toContain('// docs:end:age_check');
  });

  test('verify Vite config', () => {
    const vitePath = join(__dirname, 'fixtures/browser/vite.config.js');
    const viteContent = readFileSync(vitePath, 'utf-8');

    expect(viteContent).toContain("esbuildOptions: { target: 'esnext' }");
    expect(viteContent).toContain("exclude: ['@noir-lang/noirc_abi', '@noir-lang/acvm_js']");
    expect(viteContent).toContain('// docs:start:config');
    expect(viteContent).toContain('// docs:end:config');
  });
});
