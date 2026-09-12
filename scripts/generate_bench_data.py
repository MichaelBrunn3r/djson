from dataclasses import dataclass
from pathlib import Path
from random import Random

from utils import REPO_ROOT, ensure_file, repo_path, sha256_file

LIB_BENCHES_ROOT = REPO_ROOT / "pkgs" / "lib" / "benches"

@dataclass
class Policy:
	min_bytes: int
	allow_expressions: bool = False
	max_depth: int = 3
	rand_int_range: range = range(-1_000_000, 1_000_000)
	rand_float_range: tuple[float, float] = (-1_000_000.0, 1_000_000.0)
	max_string_draws: int = 4
	string_separators: tuple[str, ...] = ("", " ", "_", "-", ",", "\\", "\\n")

BENCHMARK_FILES = (
	{
		"destination": LIB_BENCHES_ROOT / "generated" / "static_1MB.dj",
		"policy": Policy(min_bytes=1000 * 1024),
		"expected_hash": "8e1448c1d904761d5313fa19892db9384cb2ae42d9e9fbe2ca0c85fac8cca6ac",
	},
	{
			"destination": LIB_BENCHES_ROOT / "generated" / "static_50MB.dj",
			"policy": Policy(min_bytes=50 * 1000 * 1024),
			"expected_hash": "1bd1d26c68467336b0f21794cf7abc3756a6a46508b048c2c74a1700b4c5abaa",
		},
	{
		"destination": LIB_BENCHES_ROOT / "generated" / "exprs_1MB.dj",
		"policy": Policy(min_bytes=1000 * 1024, allow_expressions=True),
		"expected_hash": "f9cecf803a92f18b2e5a19ce0329052ac4ce0d39bd792a91286af87445f7eb49",
	},
	{
		"destination": LIB_BENCHES_ROOT / "generated" / "exprs_50MB.dj",
		"policy": Policy(min_bytes=50 * 1000 * 1024, allow_expressions=True),
		"expected_hash": "9abe37d8fe191bc90d7c5370f9eb6725a703a92225e968c4bb8b93bca90d6d1d",
	},
)

class DocumentGenerator:
	def __init__(
		self,
		identifiers: list[str],
		policy: Policy,
		seed: int,
	) -> None:
		self.rng = Random(seed)
		self.identifiers = identifiers
		self.policy = policy
		self.identifier_usage: dict[str, int] = {}

		if not self.identifiers:
			raise ValueError("identifiers must not be empty")

	def write_document(self, output_path: Path) -> int:
		header = b"#!/usr/bin/env djson\n\n"
		byte_count = len(header)

		with output_path.open("wb") as output:
			output.write(header)

			while byte_count < self.policy.min_bytes:
				entry = self.generate_entry().encode("utf-8")
				output.write(entry)
				output.write(b"\n")
				byte_count += len(entry) + 1

		return byte_count

	def rand_identifier(self) -> str:
		base = self.identifiers[self.rng.randrange(len(self.identifiers))]
		return self.next_identifier(base)

	def next_identifier(self, base: str) -> str:
		usage_count = self.identifier_usage.get(base, 0)
		if usage_count == 0:
			identifier = base
		elif usage_count == 1:
			identifier = f"{base}0"
		else:
			identifier = f"{base}{usage_count - 1}"
		self.identifier_usage[base] = usage_count + 1
		return identifier

	def rand_string(self) -> str:
		draws = self.rng.randrange(1, self.policy.max_string_draws + 1)
		parts = [
			self.rng.choice([self.rand_identifier, self.rand_int])()
			for _ in range(draws)
		]
		value = parts[0]
		for part in parts[1:]:
			value += self.rng.choice(self.policy.string_separators) + part
		return f'"{value}"'

	def rand_bool(self) -> str:
		return "true" if self.rng.choice([True, False]) else "false"

	def rand_int(self) -> str:
		return str(self.rng.choice(self.policy.rand_int_range))

	def rand_float(self) -> str:
		lower, upper = self.policy.rand_float_range
		return str(self.rng.uniform(lower, upper))

	def rand_int_expression(self) -> str:
		left = self.rng.choice(self.policy.rand_int_range)
		right = self.rng.choice(self.policy.rand_int_range)
		factor = self.rng.choice(range(1, 10))
		return f"({left} + {right}) * {factor}"

	def rand_none(self) -> str:
		return "none"

	def rand_scalar(self) -> str:
		value_generators = [
			self.rand_string,
			self.rand_bool,
			self.rand_int,
			self.rand_float,
			self.rand_none,
		]
		if self.policy.allow_expressions:
			value_generators.append(self.rand_int_expression)
		return self.rng.choice(value_generators)()

	def rand_list(self, indent: int, depth: int) -> str:
		items = [
			self.rand_value(indent + 1, depth + 1)
			for _ in range(self.rng.randrange(5))
		]
		if not items:
			return "[]"

		lines = ["["]
		for index, item in enumerate(items):
			item_lines = item.splitlines()
			if index < len(items) - 1:
				item_lines[-1] += ","
			lines.append("\t" * (indent + 1) + item_lines[0])
			lines.extend(item_lines[1:])
		lines.append("\t" * indent + "]")
		return "\n".join(lines)

	def rand_map(self, indent: int, depth: int) -> str:
		fields = [self.rand_identifier() for _ in range(self.rng.randrange(6))]
		if not fields:
			return "{}"

		lines = ["{"]
		for index, field in enumerate(fields):
			value_lines = self.rand_value(indent + 1, depth + 1).splitlines()
			if index < len(fields) - 1:
				value_lines[-1] += ","
			lines.append("\t" * (indent + 1) + f"{field}: {value_lines[0]}")
			lines.extend(value_lines[1:])
		lines.append("\t" * indent + "}")
		return "\n".join(lines)

	def rand_value(self, indent: int, depth: int) -> str:
		if depth >= self.policy.max_depth:
			return self.rand_scalar()
		variant = self.rng.randrange(8 if self.policy.allow_expressions else 7)
		if variant < 5:
			return [self.rand_string, self.rand_bool, self.rand_int, self.rand_float, self.rand_none][variant]()
		if self.policy.allow_expressions and variant == 5:
			return self.rand_int_expression()
		if variant == 5 + self.policy.allow_expressions:
			return self.rand_list(indent, depth)
		return self.rand_map(indent, depth)

	def generate_entry(self) -> str:
		value_lines = self.rand_value(0, 0).splitlines()
		return "\n".join([f"{self.rand_identifier()}: {value_lines[0]}", *value_lines[1:]])


IDENTIFIERS_PATH = Path(__file__).with_name("identifiers.txt")
def load_identifiers() -> list[str]:
	identifiers = [
		line.strip()
		for line in IDENTIFIERS_PATH.read_text(encoding="utf-8").splitlines()
		if line.strip() and not line.lstrip().startswith("//")
	]
	if not identifiers:
		raise ValueError(f"no identifiers found in {IDENTIFIERS_PATH}")
	return identifiers


def run(seed: int) -> None:
	identifiers = load_identifiers()
	for benchmark_file in BENCHMARK_FILES:
		destination = benchmark_file["destination"]
		expected_hash = benchmark_file["expected_hash"]
		destination.parent.mkdir(parents=True, exist_ok=True)
		policy = benchmark_file["policy"]
		generator = DocumentGenerator(identifiers, policy, seed)
		created = ensure_file(
			destination,
			expected_hash,
			generator.write_document,
		)
		byte_count = destination.stat().st_size
		digest = sha256_file(destination)
		status = "generated" if created else "✅"
		print(f"{status} ./{repo_path(destination)}")
