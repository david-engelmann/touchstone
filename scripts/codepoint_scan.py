#!/usr/bin/env python3
"""Count U+2014 and U+2013 in every Markdown and text file in the repo.
Zero of each is required. Exit 1 when any are found."""
import pathlib, sys
root = pathlib.Path(__file__).resolve().parent.parent
counts = {"U+2014": 0, "U+2013": 0}
for p in sorted(root.rglob("*")):
    if ".git" in p.parts or not p.is_file():
        continue
    if p.suffix not in {".md", ".txt"}:
        continue
    text = p.read_text(encoding="utf-8", errors="replace")
    counts["U+2014"] += text.count("\u2014")
    counts["U+2013"] += text.count("\u2013")
print(f"codepoint scan U+2014={counts['U+2014']} U+2013={counts['U+2013']}")
sys.exit(1 if any(counts.values()) else 0)
