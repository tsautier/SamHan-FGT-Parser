# SPDX-License-Identifier: SSPL-1.0
from __future__ import annotations
from typing import Dict, Any, List

KEYWORDS = {"config", "edit", "next", "end", "set", "unset", "append", "rename", "delete", "unselect"}

def _tokenize(line: str) -> List[str]:
    # Strip comments starting with '#'
    raw = line.split("#", 1)[0].strip()
    if not raw:
        return []
    out, buf, q = [], [], False
    i = 0
    while i < len(raw):
        c = raw[i]
        if c == '"':
            q = not q
            i += 1
            start = i
            while i < len(raw) and (raw[i] != '"' or (raw[i] == '"' and raw[i-1] == '\\')):
                i += 1
            out.append(raw[start:i])
            i += 1  # skip closing quote
            if buf:
                out.insert(-1, "".join(buf))
                buf.clear()
            continue
        if not q and c.isspace():
            if buf:
                out.append("".join(buf)); buf.clear()
        else:
            buf.append(c)
        i += 1
    if buf:
        out.append("".join(buf))
    return out

def _ensure_path(root: Dict[str, Any], parts: List[str]) -> Dict[str, Any]:
    node = root
    for p in parts:
        node = node.setdefault(p, {})
    return node

class FGTParser:
    """
    Minimal FortiOS CLI parser:
    - Supports 'config [a] [b]' blocks (object or table)
    - 'edit "<name>"' entries in tables
    - 'set k v1 v2 ...' assigns str or list[str]
    - 'unset k' sets None (explicit)
    - 'append k v1 v2 ...' extends list
    - 'next' closes entry; 'end' closes config
    """
    def __init__(self) -> None:
        self.root: Dict[str, Any] = {}
        self.stack: List[Dict[str, Any]] = []  # stack of dict nodes
        self.current_entry: Dict[str, Any] | None = None

    def parse_text(self, text: str) -> Dict[str, Any]:
        self.root, self.stack, self.current_entry = {}, [], None
        for raw in text.splitlines():
            tokens = _tokenize(raw)
            if not tokens:
                continue
            cmd = tokens[0].lower()
            if cmd == "config":
                path = tokens[1:]
                if not path:
                    continue
                node = _ensure_path(self.root, path)
                self.stack.append(node)
                self.current_entry = None
            elif cmd == "edit":
                if not self.stack:
                    continue
                name = tokens[1] if len(tokens) > 1 else ""
                table = self.stack[-1]
                entry = table.setdefault(name, {})
                self.current_entry = entry
            elif cmd == "next":
                self.current_entry = None
            elif cmd == "end":
                if self.stack:
                    self.stack.pop()
                self.current_entry = None
            elif cmd in ("set", "append", "unset", "rename", "delete", "unselect"):
                target = self.current_entry if self.current_entry is not None else (self.stack[-1] if self.stack else self.root)
                if cmd == "unset" and len(tokens) >= 2:
                    target[tokens[1]] = None
                elif cmd == "set" and len(tokens) >= 3:
                    k, vs = tokens[1], tokens[2:]
                    target[k] = vs if len(vs) > 1 else vs[0]
                elif cmd == "append" and len(tokens) >= 3:
                    k, vs = tokens[1], tokens[2:]
                    cur = target.get(k)
                    if cur is None:
                        target[k] = vs
                    elif isinstance(cur, list):
                        cur.extend(vs)
                    else:
                        target[k] = [cur] + vs
                # 'rename', 'delete', 'unselect' reserved for future behavior
            else:
                self.root.setdefault("_unknown", []).append(" ".join(tokens))
        return self.root
