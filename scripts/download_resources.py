import shutil
import ssl
from pathlib import Path
from urllib.request import urlopen

import certifi

from utils import REPO_ROOT, ensure_file, repo_path

RESOURCES: dict[str, dict[str, str]] = {
	"words.txt": {
		"url": "https://github.com/dwyl/english-words/raw/20f5cc9b3f0ccc8ce45d814c532b7c2031bba31c/words.txt",
		"hash": "39a4ead879cc8283de87f2ec58e7b4b340d6caa724db1b52a91dccf2f273c5e7",
		"destination": "scripts/data/words.txt",
	},
}

def download(url: str, destination: Path) -> None:
	context = ssl.create_default_context(cafile=certifi.where())
	with urlopen(url, context=context) as response, destination.open("wb") as output:
		shutil.copyfileobj(response, output)

def run(seed: int) -> None:
	"""Download the pinned resources"""
	for dl in RESOURCES.values():
		url = dl["url"]
		expected_hash = dl["hash"]
		destination = REPO_ROOT / dl["destination"]
		created = ensure_file(
			destination,
			expected_hash,
			lambda temporary: download(url, temporary),
		)
		if created:
			print(f"downloaded {url} to {repo_path(destination)}")
		else:
			print(f"✅ ./{repo_path(destination)}")
