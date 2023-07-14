import path from 'path';
import { fileURLToPath } from 'url';

import { ContractCompiler } from './compile.js';

const getCurentDirname = () => path.dirname(fileURLToPath(import.meta.url));

it('should compile the test contract', async () => {
  const testContractPath = path.join(getCurentDirname(), 'fixtures/test_contract');
  const compiler = new ContractCompiler(testContractPath);

  const compilationResult = await compiler.compile();

  expect(compilationResult).toMatchSnapshot();
});
