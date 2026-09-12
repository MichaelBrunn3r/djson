#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.9"
# dependencies = [
#     "certifi>=2024.8.30",
# ]
# ///

from dataclasses import dataclass
from typing import Callable
import sys

from download_resources import run as download_resources
from generate_bench_data import run as generate_bench_data
from utils import HashMismatchError

SEED = 42


@dataclass
class Task:
	name: str
	run: Callable[[int], None]

TASKS = (
	Task("download resources", download_resources),
	Task("generate benchmarks", generate_bench_data),
)

def main() -> int:
	for index, task in enumerate(TASKS, start=1):
		print(f"[{index}/{len(TASKS)}] {task.name}", flush=True)
		try:
			task.run(seed=SEED)
		except HashMismatchError as error:
			print(f"error: {error}", file=sys.stderr)
			return 1
	return 0

if __name__ == "__main__":
	raise SystemExit(main())
