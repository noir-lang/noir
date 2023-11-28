import { expect } from '@jest/globals';

import { equalL2Blocks } from './eq_testers.js';

expect.addEqualityTesters([equalL2Blocks]);
