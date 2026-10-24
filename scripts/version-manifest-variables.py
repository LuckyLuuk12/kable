# This script is used as dev tool to find all variables inside version manifests, it will find the .roaming/.minecraft/versions/*/*.json files and print out all variables used in them.
# Variables are in the format ${variableName} and can be used in various places in the manifest, such as in the main class, arguments, etc. 
# This script will help us identify all variables used in the manifests so we can implement support for them in our resolver.

import os
import re
import json
import argparse
from pathlib import Path
from collections import defaultdict

PATTERN = re.compile(r"\$\{([^}]+)\}")

LOADERS = [
    ("neoforge", "neoforge"),
    ("fabric", "fabric"),
    ("quilt", "quilt"),
    ("forge", "forge"),
]


def detect_group(version_name: str) -> str:
    lower = version_name.lower()

    for key, group in LOADERS:
        if key in lower:
            return group

    return "vanilla"


def find_minecraft_dir():
    home = Path.home()
    mc = home / ".minecraft"
    if mc.exists():
        return mc

    appdata = os.environ.get("APPDATA")
    if appdata:
        mc = Path(appdata) / ".minecraft"
        if mc.exists():
            return mc

    return None


def extract_vars_with_paths(obj, path=""):
    results = set()

    if isinstance(obj, dict):
        for k, v in obj.items():
            new_path = f"{path}.{k}" if path else k
            results.update(extract_vars_with_paths(v, new_path))

    elif isinstance(obj, list):
      for v in obj:
          results.update(extract_vars_with_paths(v, path))

    elif isinstance(obj, str):
        for match in PATTERN.findall(obj):
            results.add((path, match))

    return results


def extract_from_file(file_path: Path):
    try:
        content = file_path.read_text(encoding="utf-8", errors="ignore")
        obj = json.loads(content)
    except Exception:
        return set()

    return extract_vars_with_paths(obj)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--show-paths", action="store_true")
    args = parser.parse_args()

    mc_dir = find_minecraft_dir()
    if not mc_dir:
        print("Could not locate .minecraft")
        return

    versions_dir = mc_dir / "versions"
    if not versions_dir.exists():
        print("No versions directory found")
        return

    grouped = defaultdict(set)

    for version_dir in versions_dir.iterdir():
        if not version_dir.is_dir():
            continue

        group = detect_group(version_dir.name)

        for json_file in version_dir.glob("*.json"):
            vars_found = extract_from_file(json_file)
            grouped[group].update(vars_found)

    print("\n=== RESULTS ===")

    for group in ["vanilla", "fabric", "forge", "neoforge", "quilt"]:
      values = grouped.get(group, set())
      if not values:
          continue

      print(f"\n[{group.upper()}]")

      # compute alignment width for this group
      items = sorted(values)
      max_path_len = max(len(path) for path, _ in items)

      for path, var in items:
          print(f"{path.ljust(max_path_len)}  ->  {var}")


if __name__ == "__main__":
    main()