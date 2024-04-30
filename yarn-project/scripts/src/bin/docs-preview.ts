import { COMMENT_TYPES } from '../types.js';
import main from '../utils/comment.js';
import { getPrNumber } from '../utils/pr-number.js';

void main(getPrNumber(), COMMENT_TYPES.DOCS).catch(err => {
  // eslint-disable-next-line no-console
  console.error(err.message);
  process.exit(1);
});
