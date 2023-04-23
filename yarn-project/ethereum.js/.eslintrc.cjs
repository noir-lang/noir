// module.exports = require('@aztec/foundation/eslint');

// Uncomment line 1 and remove the following to reenable the rule
const config = require('@aztec/foundation/eslint');
config.rules['jsdoc/require-param'] = 'off';
module.exports = config;
