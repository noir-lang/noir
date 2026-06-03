const fs = require('fs/promises');
const path = require('path');
const { compile, createFileManager } = require('@noir-lang/noir_wasm');

const clientDir = path.resolve(__dirname, '..');
const repoRoot = path.resolve(clientDir, '..', '..');
const circuitDir = path.join(repoRoot, 'zk', 'noir', 'recovery');
const serverArtifactPath = path.join(repoRoot, 'demo', 'server', 'artifacts', 'recovery.json');
const clientArtifactPath = path.join(clientDir, 'public', 'recovery.json');

async function readDirRecursive(dir) {
    const entries = await fs.readdir(dir);
    const files = [];

    for (const entry of entries) {
        const entryPath = path.join(dir, entry);
        const stat = await fs.stat(entryPath);

        if (stat.isFile()) {
            files.push(entryPath);
        } else {
            files.push(...await readDirRecursive(entryPath));
        }
    }

    return files;
}

async function main() {
    const fileManager = createFileManager(circuitDir);

    // Keep Windows source-map keys identical to the compiler entrypoint path.
    fileManager.readdir = async (dir, options) => {
        if (options?.recursive) {
            return readDirRecursive(dir);
        }

        return fs.readdir(dir);
    };

    const { program, warnings } = await compile(fileManager, circuitDir, () => {}, () => {});

    if (warnings?.length) {
        console.warn(JSON.stringify(warnings, null, 2));
    }

    const artifact = `${JSON.stringify(program, null, 2)}\n`;

    await fs.mkdir(path.dirname(serverArtifactPath), { recursive: true });
    await fs.mkdir(path.dirname(clientArtifactPath), { recursive: true });
    await fs.writeFile(serverArtifactPath, artifact);
    await fs.writeFile(clientArtifactPath, artifact);

    console.log(`Wrote ${serverArtifactPath}`);
    console.log(`Wrote ${clientArtifactPath}`);
}

main().catch((error) => {
    console.error(error);
    process.exit(1);
});
