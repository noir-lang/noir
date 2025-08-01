from fuzzer_output_types import NoirProgramData
from base64 import b64decode
from tempfile import NamedTemporaryFile
import json
import os
import dataclasses
import subprocess
from typing import Tuple
import logging

logging.basicConfig(
    level=logging.WARNING, format="%(asctime)s - %(levelname)s - %(message)s"
)
bb_executable_path = os.getenv("BB_EXECUTABLE_PATH", "/root/.bb/bb")


def create_program_and_witness_files(noir_data: NoirProgramData) -> Tuple[str, str]:
    program_file = NamedTemporaryFile(mode="w", suffix=".json", delete=False)
    program_dict = dataclasses.asdict(noir_data.program)

    json.dump(program_dict, program_file, indent=2)
    program_file.close()

    witness_file = NamedTemporaryFile(mode="wb", suffix=".gz", delete=False)
    witness_data = b64decode(noir_data.witness_map_b64)
    witness_file.write(witness_data)
    witness_file.close()

    return program_file.name, witness_file.name


def cleanup_temp_files(*file_paths: str) -> None:
    for file_path in file_paths:
        try:
            if os.path.exists(file_path):
                os.unlink(file_path)
        except OSError as e:
            logging.warning(
                f"Warning: Could not delete temporary file {file_path}: {e}"
            )


def prove(noir_data: NoirProgramData) -> None:
    program_file, witness_file = create_program_and_witness_files(noir_data)

    try:
        os.makedirs("./target", exist_ok=True)

        cmd = [
            bb_executable_path,
            "prove",
            "-b",
            f"{program_file}",
            "-w",
            f"{witness_file}",
            "-o",
            "./target",
        ]

        result = subprocess.run(
            cmd, cwd=".", stdout=subprocess.PIPE, stderr=subprocess.PIPE
        )

        stderr = result.stderr.decode("utf-8")
        if "Proof saved to" not in stderr:
            logging.error(f"bb prove failed: {stderr} for test {noir_data.test_id}")
        else:
            logging.info(f"bb prove completed successfully for test {noir_data.test_id}")
    finally:
        cleanup_temp_files(program_file, witness_file)


if __name__ == "__main__":
    example_json = '{"program":{"abi":{"error_types":{},"parameters":[],"return_type":null},"brillig_names":["directive_integer_quotient","directive_invert"],"bytecode":"H4sIAAAAAAAA/+1Z227iMBB1oJcQNrC7hW23TyvtD9i5FOdp+ZWlDf//Ca1VjzoMLlLJmQqkjhQZEnNyZuZ4xiGZebX7l+Nv/Jy9HOM4BluLc+H4I85dJOZdsvl07orNDzYy+8bvG8wOMwfEsgm6R2N78Z3HIo/jRCYkTwQ1TPonSI0HkhO2E8TaPjRNv6p6V7v/tuo2vrVNu3nwzrvWt0+Vr+veN37VbbqV7exj3btt29XbiJUfj+WaXSw7wflov0R5WJRFHKcUEBJgYfZFGSadkygLgxPl1GBFSbF9T4zH8u23rxYwwqIcKebHfsycPJEDY8pj+U07qJqitx+zg0EdKvrSYCsauuKGRJcs8Si/U11DQ0uFOe0FWhidjobWAVKnPOezOM6JNHXCmdnfG88ZCTJ0oSgNTuRzINZ3IBaPKceVCwW1AFG8y6iLU24OSH/nRqc4oPJMPGdArB8DfPYJn9FNJY+aRmsQuFOD7sp/AjWY2pWToRvWDMib873RJHyjgLsAikHL7wULMAhXg2sVilNYEOgdJbKALs15NA2kLn8Bc/GZRQrZ4DnfW03Ctwq4d+a0i1Tw+44FGISrskMJ3T/wHbhD0XxUdcgno9/mPAoeUuP3wFxo+jwG8rwG5/ma+Z1F3wPf0ADCW7grcV3+hmK189bJvP3hT0/ZawznivCnOviW+2aEL/y+0v+LxO+yd76PxHho7qEGXCauESbt0Dlf8mMixiXDRWqV8Bc6+MlcLdnnhfCTx3sN4kB41Gcuzb6NxDWaK9dMhufnJJdx4l5kpBn+IELxfAat80GG7h8AAA==","debug":[{"acir_locations":{},"brillig_locations":{},"brillig_procedure_locs":{},"functions":{},"location_tree":{"locations":[{"parent":null,"value":{"file":0,"span":{"end":0,"start":0}}}]},"types":{},"variables":{}}],"file_map":{},"hash":1,"names":["main"],"noir_version":"1.0.0-beta.8+e294e66d7d8c6f18a92f708742c6fabbe7f6828f","warnings":[]},"test_id":"42caac3e439d5a243636ef488a6d53cd46da5403","witness_map_b64":"H4sIAAAAAAAC/62USW5EIQxE05nnec4lbIyxvctVgG/uf4T8RUdpdaRI6e/agEA8VZWA1d6PPjbmn+sR/imnUsVdqLkPq3W1gPUtBUggY+z/YpmQ59an0SShGldrKgpMefypCgcbLIKSs0tyJKyQZgZD5lYUFVl5SkrkmlWsmYBiJsfBRmPt73BBxqkyY+9lIAtIxaOAvtbC4zgWnAT2dRqY8WwBq2zd1fNAXxeBvi43WNiHQzdJWW3eoy7FuPSamLMMSVQm5tEs6fwSCKw6tWxQgeZDWqerBb6wGyui9owIhct1YF83gXf1Ni4j3y1gydavcx+Y8SGw+8dAX0+Bvp4DWS+BGV8Dfb3tzsLthffAjF+M3HrQGAgAAA=="}'

    program_data = NoirProgramData.from_json(example_json)
    prove(program_data)
