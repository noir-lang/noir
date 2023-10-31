/**
 *  Below tests are commented because they require
 *  "type": "module", in package.json
 *  Seems that both CJS and MJS modes are not going to work.
*/
import test from 'ava';

import { initializeResolver, read_file } from "../lib-node/index.js";

test('It communicates error when read_file was called before initializeResolver.', t => {

    const error = t.throws(() => {
        const readResult = read_file("./package.json");
    }, { instanceOf: Error });

    t.is(error.message, 'Not yet initialized. Use initializeResolver(() => string)');

});

test('It calls function from initializer within read_file function.', t => {

    const RESULT_RESPONSE = "TEST";

    initializeResolver((source) => {
        return source;
    });

    const readResult = read_file(RESULT_RESPONSE);

    t.is(readResult, RESULT_RESPONSE);

});

test('It communicates error when resolver returns non-String to read_file function.', t => {

    const RESULT_RESPONSE = "TEST";

    initializeResolver((source) => {
        return Promise.resolve(source);
    });

    const error = t.throws(() => {
        read_file(RESULT_RESPONSE);
    }, { instanceOf: Error });

    t.is(error.message, 'Noir source resolver function MUST return String synchronously. Are you trying to return anything else, eg. `Promise`?');

});

test('It communicates error when resolver is initialized to anything but a function.', t => {

    const error = t.throws(() => {
        initializeResolver(null);
    }, { instanceOf: Error });

    t.is(error.message, 'Provided Noir Resolver is not a function, hint: use function(module_id) => NoirSource as second parameter');

});
