const testData: { [key: string]: { toBuffer(): Buffer }[] } = {};

/** Returns whether test data generation is enabled */
export function isGenerateTestDataEnabled() {
  return process.env.AZTEC_GENERATE_TEST_DATA === '1' && typeof expect !== 'undefined';
}

/** Pushes test data with the given name, only if test data generation is enabled. */
export function pushTestData(itemName: string, data: { toBuffer(): Buffer }) {
  if (!isGenerateTestDataEnabled()) {
    return;
  }

  if (typeof expect === 'undefined') {
    return;
  }

  const testName = expect.getState().currentTestName;
  const fullItemName = `${testName} ${itemName}`;

  if (!testData[fullItemName]) {
    testData[fullItemName] = [];
  }
  testData[fullItemName].push(data);
}

/** Returns all instances of pushed test data with the given name, or empty if test data generation is not enabled. */
export function getTestData(itemName: string): { toBuffer(): Buffer }[] {
  if (!isGenerateTestDataEnabled()) {
    return [];
  }

  const testName = expect.getState().currentTestName;
  const fullItemName = `${testName} ${itemName}`;
  return testData[fullItemName];
}
