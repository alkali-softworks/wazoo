#!/usr/bin/env python3
"""
Wazoo i18n Parity & Integrity Checker
Alkali Softworks

Scans all locale files in crates/wazoo-core/locales/ to verify:
1. Complete Key Parity:
   Every key defined in ANY language must exist across ALL languages.
   If a key is added only to English, or only to Arabic, or only to Japanese,
   it is immediately detected and flagged.
2. Placeholder Consistency:
   Placeholders like {key}, {count}, {title} must match across all translations.
3. Non-Empty Values:
   Detects empty or whitespace-only translation strings.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Set, Tuple


class Colors:
    def __init__(self, enabled: bool = True):
        self.enabled = enabled

    def _c(self, code: str, text: str) -> str:
        return f"\033[{code}m{text}\033[0m" if self.enabled else text

    def bold(self, text: str) -> str:
        return self._c("1", text)

    def cyan(self, text: str) -> str:
        return self._c("36", text)

    def green(self, text: str) -> str:
        return self._c("32", text)

    def yellow(self, text: str) -> str:
        return self._c("33", text)

    def red(self, text: str) -> str:
        return self._c("31", text)

    def magenta(self, text: str) -> str:
        return self._c("35", text)


PLACEHOLDER_REGEX = re.compile(r"\{([a-zA-Z0-9_]+)\}")


def flatten_json(data: Any, prefix: str = "") -> Dict[str, str]:
    """Recursively flattens a nested dict matching Wazoo's dot-notation convention."""
    items: Dict[str, str] = {}
    if isinstance(data, dict):
        for k, v in data.items():
            new_prefix = f"{prefix}.{k}" if prefix else k
            items.update(flatten_json(v, new_prefix))
    elif isinstance(data, str):
        items[prefix] = data
    return items


def load_locales(locales_dir: Path) -> Dict[str, Dict[str, str]]:
    """Loads all JSON locale files from locales_dir."""
    locales: Dict[str, Dict[str, str]] = {}
    json_files = sorted(locales_dir.glob("*.json"))
    if not json_files:
        raise FileNotFoundError(f"No JSON locale files found in {locales_dir}")

    for file_path in json_files:
        lang = file_path.stem
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                data = json.load(f)
            locales[lang] = flatten_json(data)
        except Exception as e:
            print(f"Error loading {file_path}: {e}", file=sys.stderr)
            raise
    return locales


def analyze_i18n(
    locales: Dict[str, Dict[str, str]]
) -> Tuple[
    Set[str],
    Dict[str, List[str]],  # key -> list of missing lang codes
    Dict[str, List[str]],  # key -> list of present lang codes (for keys with missing)
    List[Tuple[str, str, Set[str], Set[str]]],  # (key, lang, expected, actual)
    List[Tuple[str, str]],  # (key, lang) empty values
]:
    all_keys: Set[str] = set().union(*(m.keys() for m in locales.values()))
    all_langs = sorted(locales.keys())

    missing_by_key: Dict[str, List[str]] = {}
    present_by_key: Dict[str, List[str]] = {}
    placeholder_mismatches: List[Tuple[str, str, Set[str], Set[str]]] = []
    empty_values: List[Tuple[str, str]] = []

    for key in sorted(all_keys):
        present = [lang for lang in all_langs if key in locales[lang]]
        missing = [lang for lang in all_langs if key not in locales[lang]]

        if missing:
            missing_by_key[key] = missing
            present_by_key[key] = present

        # Check placeholder consistency
        # Use English placeholders as the baseline if available; otherwise the first present language
        ref_lang = "en" if "en" in present else present[0]
        ref_text = locales[ref_lang][key]
        ref_placeholders = set(PLACEHOLDER_REGEX.findall(ref_text))

        for lang in present:
            text = locales[lang][key]
            if not text.strip():
                empty_values.append((key, lang))

            placeholders = set(PLACEHOLDER_REGEX.findall(text))
            if placeholders != ref_placeholders:
                placeholder_mismatches.append(
                    (key, lang, ref_placeholders, placeholders)
                )

    return (
        all_keys,
        missing_by_key,
        present_by_key,
        placeholder_mismatches,
        empty_values,
    )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check i18n key parity and placeholder integrity across all Wazoo locales."
    )
    parser.add_argument(
        "--locales-dir",
        type=Path,
        default=None,
        help="Path to locales directory (default: crates/wazoo-core/locales)",
    )
    parser.add_argument(
        "--no-color",
        action="store_true",
        help="Disable ANSI color codes in output",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        dest="json_output",
        help="Output results in JSON format",
    )
    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Show detailed list of all scanned keys and locales",
    )
    args = parser.parse_args()

    colors = Colors(enabled=not args.no_color and sys.stdout.isatty())

    # Resolve default locales directory relative to script root
    if args.locales_dir is None:
        script_dir = Path(__file__).resolve().parent
        repo_root = script_dir.parent
        locales_dir = repo_root / "crates" / "wazoo-core" / "locales"
    else:
        locales_dir = args.locales_dir

    if not locales_dir.exists():
        print(
            colors.red(f"Error: Locales directory '{locales_dir}' does not exist."),
            file=sys.stderr,
        )
        return 2

    locales = load_locales(locales_dir)
    all_langs = sorted(locales.keys())

    (
        all_keys,
        missing_by_key,
        present_by_key,
        placeholder_mismatches,
        empty_values,
    ) = analyze_i18n(locales)

    # Missing per language mapping: lang -> list of missing keys
    missing_by_lang: Dict[str, List[str]] = {lang: [] for lang in all_langs}
    for key, missing_langs in missing_by_key.items():
        for lang in missing_langs:
            missing_by_lang[lang].append(key)

    total_keys = len(all_keys)
    total_locales = len(all_langs)
    has_issues = bool(missing_by_key or placeholder_mismatches or empty_values)

    if args.json_output:
        report = {
            "total_locales": total_locales,
            "total_keys": total_keys,
            "locales": all_langs,
            "has_issues": has_issues,
            "missing_keys_count": sum(len(v) for v in missing_by_lang.values()),
            "missing_by_key": {
                k: {"missing": m, "present": present_by_key[k]}
                for k, m in missing_by_key.items()
            },
            "missing_by_lang": {k: v for k, v in missing_by_lang.items() if v},
            "placeholder_mismatches": [
                {
                    "key": k,
                    "lang": l,
                    "expected": sorted(list(exp)),
                    "actual": sorted(list(act)),
                }
                for k, l, exp, act in placeholder_mismatches
            ],
            "empty_values": [
                {"key": k, "lang": l} for k, l in empty_values
            ],
        }
        print(json.dumps(report, indent=2, ensure_ascii=False))
        return 1 if has_issues else 0

    # Human-readable CLI output
    print(colors.bold(colors.cyan("=== Wazoo i18n Parity & Integrity Checker ===")))
    print(f"Locales Directory: {locales_dir}")
    print(f"Scanned Locales ({total_locales}): {', '.join(all_langs)}")
    print(f"Total Unique Keys: {total_keys}\n")

    if args.verbose:
        print(colors.cyan("Scanned Key List:"))
        for k in sorted(all_keys):
            print(f"  • {k}")
        print()

    # 1. Missing keys report
    if missing_by_key:
        print(
            colors.bold(
                colors.red(
                    f"✖ Found {len(missing_by_key)} key(s) with missing translations across locales:"
                )
            )
        )
        for key in sorted(missing_by_key.keys()):
            missing = missing_by_key[key]
            present = present_by_key[key]
            if len(present) == 1:
                origin = present[0]
                print(
                    colors.yellow(
                        f"  [SOLO KEY] '{key}' is ONLY defined in '{origin}'!"
                    )
                )
                print(f"             Missing in ({len(missing)}): {', '.join(missing)}")
            elif "en" in present and len(missing) < len(all_langs) - 1:
                print(colors.red(f"  • '{key}'"))
                print(f"      Missing in ({len(missing)}): {', '.join(missing)}")
                print(f"      Present in ({len(present)}): {', '.join(present)}")
            else:
                print(colors.red(f"  • '{key}'"))
                print(f"      Missing in ({len(missing)}): {', '.join(missing)}")
                print(f"      Present in ({len(present)}): {', '.join(present)}")
        print()

        print(colors.bold("Missing Keys Summary by Locale:"))
        for lang in all_langs:
            missing_count = len(missing_by_lang[lang])
            if missing_count > 0:
                print(
                    colors.red(f"  - {lang:5s}: missing {missing_count} keys")
                )
            else:
                print(colors.green(f"  - {lang:5s}: 100% complete ({total_keys}/{total_keys})"))
        print()
    else:
        print(
            colors.green(
                f"✔ All {total_locales} locales have 100% key parity ({total_keys}/{total_keys} keys each)."
            )
        )

    # 2. Placeholder mismatches report
    if placeholder_mismatches:
        print(
            colors.bold(
                colors.red(
                    f"\n✖ Found {len(placeholder_mismatches)} placeholder mismatch(es):"
                )
            )
        )
        for key, lang, expected, actual in placeholder_mismatches:
            exp_str = ", ".join(f"{{{p}}}" for p in sorted(expected)) or "None"
            act_str = ", ".join(f"{{{p}}}" for p in sorted(actual)) or "None"
            print(f"  • Key: {colors.bold(key)} [{lang}]")
            print(f"      Expected placeholder(s): {colors.green(exp_str)}")
            print(f"      Actual placeholder(s):   {colors.red(act_str)}")
        print()
    else:
        print(colors.green("✔ All placeholder parameters ({...}) match across all locales."))

    # 3. Empty values report
    if empty_values:
        print(
            colors.bold(
                colors.red(
                    f"\n✖ Found {len(empty_values)} empty or whitespace-only translation value(s):"
                )
            )
        )
        for key, lang in empty_values:
            print(f"  • Key: {colors.bold(key)} in locale [{lang}] is empty")
        print()
    else:
        print(colors.green("✔ No empty or whitespace translation strings found."))

    # Final verdict
    print()
    if has_issues:
        print(
            colors.bold(
                colors.red(
                    "RESULT: FAILED. Please resolve the i18n discrepancies listed above."
                )
            )
        )
        return 1
    else:
        print(
            colors.bold(
                colors.green(
                    f"RESULT: PASSED. All {total_locales} locales are in complete parity and consistent!"
                )
            )
        )
        return 0


if __name__ == "__main__":
    sys.exit(main())
