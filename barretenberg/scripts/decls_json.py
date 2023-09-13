#!/usr/bin/env python3
import sys
import json
import clang.cindex
from typing import List

clang.cindex.Config.set_library_file('/usr/lib/llvm-16/lib/libclang-16.so.1')

def has_annotation(node, annotation):
    for child in node.get_children():
        if child.kind == clang.cindex.CursorKind.ANNOTATE_ATTR and annotation in child.spelling:
            return True
    return False

def print_diagnostic(diagnostic, file=sys.stdout):
    # color codes for printing
    BLUE = '\033[94m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    ENDC = '\033[0m'

    color_map = {
        clang.cindex.Diagnostic.Warning: YELLOW,
        clang.cindex.Diagnostic.Error: RED,
        clang.cindex.Diagnostic.Fatal: RED,
    }
    color = color_map.get(diagnostic.severity, BLUE)
    print(color + str(diagnostic) + ENDC, file=file)


def process_files(files: List[str]) -> List[dict]:
    result = []
    idx = clang.cindex.Index.create()
    for path in files:
        tu = idx.parse(path, args=[
            '-isystem', '/usr/include/c++/10',
            '-isystem', '/usr/include/x86_64-linux-gnu/c++/10',
            '-isystem', '/usr/include/c++/10/backward',
            '-isystem', '/usr/lib/llvm-15/lib/clang/15.0.7/include',
            '-isystem', '/usr/local/include',
            '-isystem', '/usr/include/x86_64-linux-gnu',
            '-isystem', '/usr/include',
            "-I./cpp/src",
            '-std=gnu++20', '-Wall', '-Wextra'])
        for diag in tu.diagnostics:
            print_diagnostic(diag, file=sys.stderr)
        for node in tu.cursor.walk_preorder():
            try:
                if node.kind == clang.cindex.CursorKind.FUNCTION_DECL:
                    # if node.spelling != "env_test_threads":
                    #     continue
                    # Only interested in function declarations with WASM_EXPORT token.
                    if not has_annotation(node, 'wasm_export'):
                        continue

                    if node.result_type.spelling != "void":
                        raise ValueError(f"Error: Function '{node.spelling}' must have a 'void' return type")
                    func = {
                        'functionName': node.spelling,
                        'inArgs': [
                            {
                                'name': arg.spelling,
                                'type': arg.type.spelling,
                            } for arg in node.get_arguments() if arg.type.get_canonical().get_pointee().is_const_qualified() or arg.type.get_canonical().is_const_qualified()
                        ],
                        'outArgs': [
                            {
                                'name': arg.spelling,
                                'type': arg.type.spelling,
                            } for arg in node.get_arguments() if not (arg.type.get_canonical().get_pointee().is_const_qualified() or arg.type.get_canonical().is_const_qualified())
                        ],
                        'isAsync': has_annotation(node, 'async_wasm_export')
                    }
                    result.append(func)
            except ValueError as e:
                if not str(e).startswith("Unknown template argument kind"):
                    raise
    return result

if __name__ == '__main__':
    file_list = [line.strip() for line in sys.stdin]
    processed_data = process_files(file_list)
    print(json.dumps(processed_data, indent=2))
