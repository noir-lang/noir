import { initialiseResolver, read_file } from './index.js';

initialiseResolver((source_id: String) => {
    let fileContent = "";
    try {
        // @ts-ignore
        const fs = require("fs");
        fileContent =
            fs.readFileSync(source_id, { encoding: "utf8" }) as string
            ;
    } catch (e) {
        console.log(e);
    }
    return fileContent;
});

export { initialiseResolver, read_file };


