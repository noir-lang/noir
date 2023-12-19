"use strict";
/// <reference types="node" />
Object.defineProperty(exports, "__esModule", { value: true });
exports.read_file = exports.initializeResolver = void 0;
const index_js_1 = require("./index.js");
Object.defineProperty(exports, "initializeResolver", { enumerable: true, get: function () { return index_js_1.initializeResolver; } });
Object.defineProperty(exports, "read_file", { enumerable: true, get: function () { return index_js_1.read_file; } });
(0, index_js_1.initializeResolver)((source_id) => {
    let fileContent = '';
    try {
        // eslint-disable-next-line @typescript-eslint/no-var-requires
        const fs = require('fs');
        fileContent = fs.readFileSync(source_id, { encoding: 'utf8' });
    }
    catch (e) {
        console.log(e);
    }
    return fileContent;
});
//# sourceMappingURL=index_node.js.map