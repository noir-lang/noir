const test = require('ava');

const { initialiseResolver, read_file } = require("../lib-node/index_node.js");

test('It reads file from file system within read_file using default implementation.', t => {

    const readResult = read_file("./package.json");

    t.assert(readResult, "return from `read_file` should by truthy");

});

test('It calls function from initializer within read_file function.', t => {

    const RESULT_RESPONSE = "TEST";

    initialiseResolver((source) => {
        return source;
    });

    const readResult = read_file(RESULT_RESPONSE);

    t.is(readResult, RESULT_RESPONSE);

});

test('It communicates error when resolver returns non-String to read_file function.', t => {

    const RESULT_RESPONSE = "TEST";

    initialiseResolver((source) => {
        return Promise.resolve(source);
    });

    const error = t.throws(() => {
        read_file(RESULT_RESPONSE);
    }, { instanceOf: Error });

    t.is(error.message, 'Noir source resolver funtion MUST return String synchronously. Are you trying to return anything else, eg. `Promise`?');

});

test('It communicates error when resolver is initialized to anything but a function.', t => {

    const error = t.throws(() => {
        initialiseResolver(null);
    }, { instanceOf: Error });

    t.is(error.message, 'Provided Noir Resolver is not a function, hint: use function(module_id) => NoirSource as second parameter');

});
