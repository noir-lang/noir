#!/usr/bin/env python3
"""
Convert binary barretenberg output files to field JSON format.
Each field element is 32 bytes in big-endian format.
"""
import sys
import json

def binary_to_fields_json(binary_file_path, json_file_path):
    """Read binary file and convert to JSON array of field strings."""
    with open(binary_file_path, 'rb') as f:
        data = f.read()

    # Each field element is 32 bytes
    field_size = 32
    if len(data) % field_size != 0:
        raise ValueError(f"File size {len(data)} is not a multiple of {field_size}")

    num_fields = len(data) // field_size
    fields = []

    for i in range(num_fields):
        start = i * field_size
        end = start + field_size
        field_bytes = data[start:end]
        # Convert to integer (big-endian)
        field_int = int.from_bytes(field_bytes, byteorder='big')
        # Convert to hex string with 0x prefix
        fields.append(f"0x{field_int:064x}")

    with open(json_file_path, 'w') as f:
        json.dump(fields, f)

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <input_binary_file> <output_json_file>")
        sys.exit(1)

    binary_to_fields_json(sys.argv[1], sys.argv[2])
