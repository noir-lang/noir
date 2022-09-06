// Synchronously reads a file
const fs = require("fs");

module.exports.read_file = function (path) {
    return fs.readFileSync(path, { encoding: "utf8" });
};