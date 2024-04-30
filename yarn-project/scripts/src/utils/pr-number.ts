/** Inpsects process.env to returns the number of the current PR */
export function getPrNumber(): number {
  let prString;
  if (process.env.PR_NUMBER) {
    prString = process.env.PR_NUMBER;
  }
  if (process.env.CIRCLE_PULL_REQUEST) {
    const fragments = process.env.CIRCLE_PULL_REQUEST.split('/');
    prString = fragments[fragments.length - 1];
  }
  if (!prString) {
    throw new Error(`PR number not found. Either set it as PR_NUMBER or a url with CIRCLE_PULL_REQUEST.`);
  }

  const prNumber = parseInt(prString, 10);

  if (isNaN(prNumber)) {
    throw new Error(`Invalid PR number: ${prString}`);
  }

  return prNumber;
}
