# Put these package.json files in the cjs and
# mjs directory respectively, so that 
#  tools can recognise that the .js files are either
# commonjs or ESM files.
cat >lib/cjs/package.json <<!EOF
{
    "type": "commonjs"
}
!EOF

cat >lib/esm/package.json <<!EOF
{
    "type": "module"
}
!EOF