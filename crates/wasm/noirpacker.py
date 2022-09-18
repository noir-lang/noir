#!/usr/bin/python3
import os
import re
import sys

def read_dir(path):
    ret = ""
    modules = {}

    # First find all modules and group together modules of the same name.
    # This can happen when we have a `foo.nr` file and `foo` directory.
    for entry in os.scandir(path):
        if entry.is_dir():
            append_entry(modules, entry.name, entry)

        elif entry.is_file() and entry.name.endswith(".nr"):
            if entry.name == "main.nr" or entry.name == "lib.nr":
                append_entry(modules, "", entry)
            else:
                append_entry(modules, entry.name[:-3], entry)

    for name, entries in modules.items():
        # No mod construct for the main module
        if name != "":
            ret += f"mod {name} {{\n"

        for entry in entries:
            if entry.is_dir():
                ret += read_dir(entry)
            elif entry.is_file():
                ret += sanitize(open(entry).read())

        if name != "":
            ret += "\n}\n"

    return ret

def append_entry(modules, name, entry):
    if name in modules:
        modules[name].append(entry)
    else:
        modules[name] = [entry]

# Remove 'mod foo;' declarations from files
def sanitize(contents):
    return re.sub(r'\bmod \w+;', '', contents)

if len(sys.argv) < 2:
    print('usage: ./noirpacker.py <path-to-src-dir> [output-file]')
else:
    path = sys.argv[1]

    packed_dir = read_dir(path)
    if len(sys.argv) == 2:
        print(packed_dir)
    else:
        f = open(sys.argv[2], 'w')
        f.write(packed_dir)
        f.close()
