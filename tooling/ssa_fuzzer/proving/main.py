#!/usr/bin/env python3

import redis
import time
import os
import logging
from prover import prove
from fuzzer_output_types import NoirProgramData

logging.basicConfig(
    level=logging.WARNING, format="%(asctime)s - %(levelname)s - %(message)s"
)
redis_client = redis.Redis(
    host=os.getenv("REDIS_HOST", "localhost"), port=os.getenv("REDIS_PORT", 6379), db=0
)


def main():
    while True:
        program_data = redis_client.rpop("fuzzer_output")
        if program_data is None:
            time.sleep(1)
            continue

        program_data = NoirProgramData.from_json(program_data)
        prove(program_data)


if __name__ == "__main__":
    main()
