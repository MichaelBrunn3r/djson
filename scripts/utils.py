from hashlib import sha256
from pathlib import Path
from typing import Callable

REPO_ROOT = Path(__file__).resolve().parents[1]

def repo_path(path: Path) -> Path:
	"""Return path relative to the repository root."""
	try:
		return path.relative_to(REPO_ROOT)
	except ValueError:
		return path

class HashMismatchError(ValueError):
	def __init__(self, path: Path, expected: str, actual: str) -> None:
		self.path = path
		self.expected = expected
		self.actual = actual
		super().__init__(self._message())

	def _message(self) -> str:
		return (
			f"hash verification failed for {repo_path(self.path)}\n"
			f"  expected SHA-256: {self.expected}\n"
			f"  actual SHA-256:   {self.actual}\n"
			"  the generated file was discarded; check the generator inputs "
			"or update the expected hash intentionally"
		)

def sha256_file(path: Path) -> str:
	digest = sha256()
	with path.open("rb") as input_file:
		for chunk in iter(lambda: input_file.read(1024 * 1024), b""):
			digest.update(chunk)
	return digest.hexdigest()

def ensure_file(
	destination: Path,
	expected_hash: str,
	producer: Callable[[Path], None],
) -> bool:
	if destination.exists() and sha256_file(destination) == expected_hash:
		return False

	destination.parent.mkdir(parents=True, exist_ok=True)
	temporary = destination.with_suffix(destination.suffix + ".tmp")
	try:
		producer(temporary)
		actual_hash = sha256_file(temporary)
		if actual_hash != expected_hash:
			raise HashMismatchError(destination, expected_hash, actual_hash)
		temporary.replace(destination)
		return True
	finally:
		temporary.unlink(missing_ok=True)
