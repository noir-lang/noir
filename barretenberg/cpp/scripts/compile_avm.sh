#!/bin/bash
use_zsh_alias() {
    # Run Zsh command, source .zshrc, and then execute the alias
    zsh -i -c "$1"
}

# Compile
use_zsh_alias "bb_pil pil/avm/avm_main.pil --name Avm"

# Format generated folders
root_dir="src"

# Find all directories named 'generate' under the specified root directory
find "$root_dir" -type d -name 'generate' | while read dir_path; do
    echo "Processing directory: $dir_path"
    
    # Find all C/C++ source files in these directories and format them
    find "$dir_path" -type f \( -iname '*.hpp' -o -iname '*.cpp' \) -exec clang-format -i {} +
done


# Build vm tests
cmake --build --preset clang16 --target vm_tests