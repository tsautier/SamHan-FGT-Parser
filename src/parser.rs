// SPDX-License-Identifier: MIT
use serde_json::{Value, Map};
use anyhow::Result;

pub fn parse_forti(text: &str) -> Result<Value> {
    let mut root = Value::Object(Map::new());
    let mut stack: Vec<String> = Vec::new();
    let mut current: Option<String> = None;

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() { continue; }
        let tokens = tokenize(line);
        if tokens.is_empty() { continue; }
        let cmd = tokens[0].to_lowercase();
        match cmd.as_str() {
            "config" => {
                let path = tokens[1..].to_vec();
                ensure_path(&mut root, &path);
                stack = path;
                current = None;
            }
            "edit" => {
                if stack.is_empty() { continue; }
                let name = tokens.get(1).cloned().unwrap_or_default();
                let table = get_table_mut(&mut root, &stack).unwrap();
                if !table.contains_key(&name) { table.insert(name.clone(), Value::Object(Map::new())); }
                current = Some(name);
            }
            "next" => current = None,
            "end" => { if !stack.is_empty() { stack.pop(); } current = None; }
            "set" | "append" | "unset" => {
                let key = tokens.get(1).cloned().unwrap_or_default();
                let vals = tokens[2..].to_vec();
                let tgt_obj = if let Some(ref cur) = current {
                    get_table_mut(&mut root, &stack).unwrap().get_mut(cur).unwrap().as_object_mut().unwrap()
                } else {
                    get_table_mut(&mut root, &stack).unwrap()
                };
                match cmd.as_str() {
                    "unset" => { tgt_obj.insert(key, Value::Null); }
                    "set" => {
                        if vals.len()==1 { tgt_obj.insert(key, Value::String(vals[0].clone())); }
                        else { tgt_obj.insert(key, Value::Array(vals.into_iter().map(Value::String).collect())); }
                    }
                    "append" => {
                        match tgt_obj.get_mut(&key) {
                            Some(Value::Array(a)) => a.extend(vals.into_iter().map(Value::String)),
                            Some(Value::String(s)) => { let mut arr = vec![Value::String(s.clone())]; arr.extend(vals.into_iter().map(Value::String)); *tgt_obj.get_mut(&key).unwrap() = Value::Array(arr); }
                            _ => { tgt_obj.insert(key, Value::Array(vals.into_iter().map(Value::String).collect())); }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(root)
}

fn ensure_path(root: &mut Value, parts: &[String]) {
    let mut cur = root;
    for p in parts {
        if cur.get(p).is_none() {
            cur.as_object_mut().unwrap().insert(p.clone(), Value::Object(Map::new()));
        }
        cur = cur.get_mut(p).unwrap();
    }
}
fn get_table_mut<'a>(root: &'a mut Value, parts: &[String]) -> Option<&'a mut Map<String, Value>> {
    let mut cur = root;
    for p in parts { cur = cur.get_mut(p)?; }
    cur.as_object_mut()
}
fn tokenize(line: &str) -> Vec<String> {
    let mut out = Vec::new(); let mut buf = String::new();
    let mut in_q = false; let mut q_ch = '\0'; let mut esc = false;
    for c in line.chars() {
        if !in_q && c == '#' { break; }
        if esc { buf.push(c); esc=false; continue; }
        if c == '\\' { esc=true; continue; }
        if in_q { if c==q_ch { in_q=false; } else { buf.push(c); } continue; }
        if c=='"' || c=='\'' { in_q=true; q_ch=c; continue; }
        if c.is_whitespace() { if !buf.is_empty() { out.push(std::mem::take(&mut buf)); } continue; }
        buf.push(c);
    }
    if esc { buf.insert(0, '\\'); }
    if !buf.is_empty() { out.push(buf); }
    out
}
