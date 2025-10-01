# SPDX-License-Identifier: SSPL-1.0
import pathlib, sys

HEADER = """SPDX-License-Identifier: SSPL-1.0
Copyright (C) 2025

This file is part of fgt-webparser and is licensed under the
Server Side Public License, version 1. See the LICENSE file.
"""

EXTS = {".py": "# ", ".js": "// ", ".ts": "// ", ".html": "<!-- ", ".css": "/* ", ".toml": "# ", ".yml": "# ", ".yaml": "# ", ".dockerfile": "# "}

def build_header(ext: str) -> str:
    if ext == ".html":
        return f"<!-- {HEADER} -->\n"
    if ext == ".css":
        return f"/* {HEADER} */\n"
    prefix = EXTS.get(ext, "# ")
    return "\n".join(prefix + line for line in HEADER.splitlines()) + "\n\n"

def should_skip(text: str) -> bool:
    return "SPDX-License-Identifier: SSPL-1.0" in text

def main(root=".", dry=False):
    for p in pathlib.Path(root).rglob("*"):
        if not p.is_file(): 
            continue
        ext = p.suffix.lower() if p.suffix else (".dockerfile" if p.name.lower()=="dockerfile" else "")
        if ext not in EXTS: 
            continue
        t = p.read_text(encoding="utf-8", errors="ignore")
        if should_skip(t): 
            continue
        h = build_header(ext)
        if not dry:
            p.write_text(h + t, encoding="utf-8")
        print(f"header -> {p}")

if __name__ == "__main__":
    main(*sys.argv[1:])
