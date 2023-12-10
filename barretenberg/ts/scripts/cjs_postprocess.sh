#!/bin/sh
cat >dest/node-cjs/package.json <<!EOF
{
    "type": "commonjs"
}
!EOF

DIR="./dest/node-cjs"

# Iterate over all .js files in the directory
for FILE in $(find "$DIR" -name "*.js"); do
    # Use sed to replace 'import.meta.url' with '""'
    sed -i.bak 's/import\.meta\.url/""/g' "$FILE" && rm "$FILE.bak"
    # Use sed to remove any lines postfixed // POSTPROCESS ESM ONLY
    sed -i.bak '/\/\/ POSTPROCESS ESM ONLY$/d' "$FILE" && rm "$FILE.bak"
done