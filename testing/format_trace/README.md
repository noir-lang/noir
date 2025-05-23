# Format-trace-tool
## Overview
This tool is used for formatting json files, especially ones that are generated from the command:
```bash
nargo trace
```
## Usage
You need to provide two arguments, first being the source file containing the json, second is the destination file name.  
Example:
```bash
cargo run src.json des.json 
```
This will generate a file in the current directory named "des.json" containing the output of our program.  
### Trace formatting example
Input:  
```json
[{"a":1},{"b":"bbb"},{"c":{"f1":3,"f2":"0"}}]
```
Output:
```json
[
  { "a": 1 },
  { "b": "bbb" },
  { "c": { "f1": 3, "f2": "0" } }
]
```