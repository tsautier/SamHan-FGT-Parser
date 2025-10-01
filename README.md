# fgt-webparser

**Independent, web-based parser for FortiGate configuration backups.**  
Upload a `show full-configuration` or exported `.conf` and get a structured JSON/YAML view you can search, diff, and export.

> This project is **unofficial** and **not affiliated with Fortinet**. See [Trademarks](#trademarks).

---

## Why this project?

- **Original implementation** (no reused code) to avoid look-alike designs.
- **Web-first UX**: drag-and-drop config, instant JSON/YAML output.
- **Structured data**: consistent keys for tables, entries, and `set/append/unset`.
- **SSPL-1.0 license**: if you offer this tool **as a service**, you must publish the **complete service source** (UI, backend, orchestration).

---

## Features (v0.1)

- Parse FortiGate CLI config blocks: `config … / edit … / set / append / unset / next / end`.
- Handle quotes, inline comments (`#`), and multi-value `set`/`append`.
- Output as **JSON** (default) or **YAML**.
- REST API:
  - `POST /api/parse` → structured config (option `format=yaml`)
  - `GET /healthz` → readiness probe
- Browser UI: minimal single-page upload & display.

Planned:
- VDOM mapping (`config vdom`) into a `vdoms` object
- Structured **diff** (`POST /api/diff`) between two configs
- CSV extractors for common sections (e.g., firewall policies, addresses)
- JSON Schema validation

---

## Quickstart

### Docker (recommended)

```bash
docker build -t fgt-webparser .
docker run --rm -p 8080:8080 fgt-webparser
# Open: http://localhost:8080/
```

### Local (Python 3.11+)

```bash
pip install -r requirements.txt  # or use pyproject.toml with uv/pip
uvicorn app.main:app --host 0.0.0.0 --port 8080 --reload
# Open: http://localhost:8080/
```

---

## API

### `POST /api/parse`

- **Form fields**:
  - `file`: the `.conf` text file (preferred)
  - or `text`: raw configuration text
  - `format`: `"json"` (default) or `"yaml"`

- **Response**:
  - `200 OK` with JSON body (or `text/yaml` if `format=yaml`)
  - `400` if no input, unsupported content type, etc.

**curl example**

```bash
curl -s -F "file=@backup.conf" -F "format=json" http://localhost:8080/api/parse | jq .
```

---

## Output shape (example)

```json
{
  "system": {
    "interface": {
      "port1": {
        "ip": "192.0.2.1/24",
        "allowaccess": ["ping", "https"],
        "alias": "WAN"
      },
      "port2": {
        "mode": "dhcp"
      }
    }
  }
}
```

> Notes  
> • `set k v1 v2` becomes a single string if one value, otherwise a list.  
> • `append k …` extends lists (or converts a scalar to list).  
> • `unset k` sets the key to `null` (explicitly present).

---

## Project structure

```
app/
  main.py            # FastAPI app & endpoints
  parser_core.py     # Original tokenizer + state machine parser
public/
  index.html         # Minimal single-page UI
scripts/
  add_license_headers.py  # Adds SSPL SPDX headers to all source files
Dockerfile
pyproject.toml
LICENSE              # SSPL-1.0 (add the official text)
README.md
```

---

## Security & privacy

- **Stateless by default**: files are processed in memory; nothing is persisted.
- Max upload size and content-type checks on `/api/parse`.
- CORS allowed for quick testing; tighten for production.
- If you deploy a public SaaS, review rate limiting and add an allow-list if needed.

---

## Development

- Keep parser rules conservative; prefer emitting `_unknown` lines instead of guessing.
- Add fixtures under `tests/fixtures/` with real-world but anonymized snippets.
- PRs should include unit tests for new CLI constructs.

---

## License

This project is licensed under the **Server Side Public License, v1 (SSPL-1.0)**.  
See the `LICENSE` file for the full text.

> **Summary (non-legal)**: You may use, modify, and distribute the software. If you offer it **as a service to third parties**, you must release the **complete service source code** (including management scripts, UI, and orchestration used to run the service).

Each source file includes an `SPDX-License-Identifier: SSPL-1.0` header. Use `scripts/add_license_headers.py` to ensure headers are present.

---

## Trademarks

**Fortinet®, FortiGate®, and FortiOS® are trademarks or registered trademarks of Fortinet, Inc.**  
This project is an independent, community-developed tool intended to interoperate with FortiGate configuration files. It is **not** affiliated with, endorsed, or sponsored by Fortinet, Inc.

- Use “FortiGate” only to describe compatibility (nominative use).
- Do not use Fortinet logos or trade dress in the UI or repository branding.

---

## Naming guidance

- Repository/package name: **fgt-webparser**.
- Avoid putting “Fortinet”/“FortiGate” in the **project name** or domain. Use it only in descriptions.

---

## Roadmap

- VDOM segmentation
- Rich diff views (added/removed/changed keys)
- CSV/NDJSON exporters for policies/objects
- Optional client-side encryption for uploads
- Web UI enhancements (search & filters)

---

## Acknowledgments

Built with FastAPI and Python. FortiGate CLI structure is documented publicly and referenced **only for interoperability**.


---

### Vendoring the full SSPL text

This repo includes an SPDX identifier in `LICENSE`. To vendor the **official SSPL v1 full text**, run:

```bash
python scripts/fetch_sspl_license.py
```

This fetches the canonical license text from SPDX/MongoDB and overwrites `LICENSE`.
