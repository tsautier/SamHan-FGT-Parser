# SPDX-License-Identifier: SSPL-1.0
"""
Fetch the official SSPL v1 text and overwrite the project's LICENSE file.

Usage:
  python scripts/fetch_sspl_license.py
"""
from __future__ import annotations
import sys, re, html
from urllib.request import urlopen, Request

SSPL_HTML_URLS = [
    "https://spdx.org/licenses/SSPL-1.0.html",
    "https://www.mongodb.com/legal/licensing/server-side-public-license",
]

def fetch(url: str) -> str:
    req = Request(url, headers={"User-Agent": "Mozilla/5.0"})
    with urlopen(req, timeout=30) as r:
        return r.read().decode("utf-8", errors="replace")

def extract_from_spdx(html_text: str) -> str | None:
    # SPDX pages usually contain <pre id="license-text">…</pre>
    m = re.search(r'<pre[^>]*id=["\\\']?license-text["\\\']?[^>]*>(.*?)</pre>', html_text, flags=re.S|re.I)
    if not m:
        # Fallback: first <pre> block
        m = re.search(r'<pre[^>]*>(.*?)</pre>', html_text, flags=re.S|re.I)
    if not m:
        return None
    txt = m.group(1)
    txt = re.sub(r'<br\\s*/?>', '\n', txt, flags=re.I)
    txt = re.sub(r'<[^>]+>', '', txt)
    txt = html.unescape(txt)
    return txt.strip()

def extract_from_mongodb(html_text: str) -> str | None:
    # MongoDB page contains headings and license; try to locate the "TERMS AND CONDITIONS" to end
    txt = html.unescape(re.sub(r'<[^>]+>', '\n', html_text))
    # Collapse multiple newlines
    txt = re.sub(r'\n{3,}', '\n\n', txt)
    # Try to detect from "Server Side Public License VERSION 1" to end of "TERMS AND CONDITIONS" section
    idx = txt.find("Server Side Public License")
    if idx == -1:
        return None
    return txt[idx:].strip()

def main():
    license_text = None
    # Try SPDX first (clean pre block)
    try:
        h = fetch(SSPL_HTML_URLS[0])
        license_text = extract_from_spdx(h)
    except Exception as e:
        print(f"[warn] SPDX fetch failed: {e}")
    # Fallback to MongoDB page
    if not license_text:
        try:
            h = fetch(SSPL_HTML_URLS[1])
            license_text = extract_from_mongodb(h)
        except Exception as e:
            print(f"[warn] MongoDB fetch failed: {e}")
    if not license_text or "Server Side Public License" not in license_text:
        print("[error] Could not retrieve SSPL text. Check your network and try again.")
        sys.exit(1)
    # Write LICENSE
    with open("LICENSE", "w", encoding="utf-8") as f:
        f.write("SPDX-License-Identifier: SSPL-1.0\n\n")
        f.write(license_text.strip() + "\n")
    print("[ok] LICENSE updated with official SSPL v1 text.")

if __name__ == "__main__":
    main()
