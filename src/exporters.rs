// SPDX-License-Identifier: MIT
use serde_json::Value;
use std::collections::BTreeMap;

fn as_list(v: &Value) -> Vec<String> {
    match v {
        Value::Array(a) => a
            .iter()
            .map(|x| x.as_str().map(|s| s.to_string()).unwrap_or_else(|| x.to_string()))
            .collect(),
        Value::Null => vec![],
        Value::String(s) => vec![s.clone()],
        other => vec![other.to_string()],
    }
}

pub fn extract_addresses(parsed: &Value) -> Vec<BTreeMap<String, String>> {
    let mut rows = Vec::new();
    if let Some(tbl) = parsed.get("firewall").and_then(|f| f.get("address")).and_then(|v| v.as_object()) {
        for (name, e) in tbl {
            let subnet = match e.get("subnet") {
                Some(Value::Array(a)) if a.len() == 2 => format!("{} {}", a[0], a[1]),
                Some(Value::String(s)) => s.clone(),
                _ => String::new()
            };
            let mut row = BTreeMap::new();
            row.insert("name".into(), name.clone());
            row.insert("type".into(), e.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string());
            row.insert("subnet".into(), subnet);
            row.insert("fqdn".into(), e.get("fqdn").and_then(|v| v.as_str()).unwrap_or("").to_string());
            row.insert("interface".into(), e.get("associated-interface").or_else(|| e.get("interface")).and_then(|v| v.as_str()).unwrap_or("").to_string());
            row.insert("comment".into(), e.get("comment").or_else(|| e.get("comments")).and_then(|v| v.as_str()).unwrap_or("").to_string());
            rows.push(row);
        }
    }
    rows
}

pub fn extract_policies(parsed: &Value) -> Vec<BTreeMap<String, String>> {
    let mut rows = Vec::new();
    if let Some(tbl) = parsed.get("firewall").and_then(|f| f.get("policy")).and_then(|v| v.as_object()) {
        for (id, e) in tbl {
            let mut row = BTreeMap::new();
            row.insert("id".into(), id.clone());
            row.insert("name".into(), e.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string());
            row.insert("action".into(), e.get("action").and_then(|v| v.as_str()).unwrap_or("").to_string());
            let join = |k: &str| as_list(e.get(k).unwrap_or(&Value::Null)).join(" | ");
            row.insert("srcintf".into(), join("srcintf"));
            row.insert("dstintf".into(), join("dstintf"));
            row.insert("srcaddr".into(), join("srcaddr"));
            row.insert("dstaddr".into(), join("dstaddr"));
            row.insert("service".into(), join("service"));
            row.insert("schedule".into(), e.get("schedule").and_then(|v| v.as_str()).unwrap_or("").to_string());
            row.insert("nat".into(), e.get("nat").map(|v| v.to_string()).unwrap_or_default());
            row.insert("logtraffic".into(), e.get("logtraffic").or_else(|| e.get("logtraffic_start")).map(|v| v.to_string()).unwrap_or_default());
            row.insert("comments".into(), e.get("comments").or_else(|| e.get("comment")).and_then(|v| v.as_str()).unwrap_or("").to_string());
            rows.push(row);
        }
    }
    rows
}

pub fn to_csv(rows: &Vec<BTreeMap<String, String>>, preferred: Option<&[&str]>) -> String {
    if rows.is_empty() { return String::new(); }
    let mut keys: Vec<String> = rows.iter().flat_map(|r| r.keys().cloned()).collect();
    keys.sort(); keys.dedup();
    if let Some(pref) = preferred {
        let mut ord: Vec<String> = pref.iter().map(|s| s.to_string()).collect();
        for k in &keys { if !ord.contains(k) { ord.push(k.clone()); } }
        keys = ord;
    }
    let mut out = String::new();
    out.push_str(&keys.join(",")); out.push('\n');
    for r in rows {
        let line: Vec<String> = keys.iter().map(|k| csv_escape(r.get(k).cloned().unwrap_or_default())).collect();
        out.push_str(&line.join(",")); out.push('\n');
    }
    out
}
fn csv_escape(s: String) -> String {
    if s.contains(',') || s.contains('\n') || s.contains('\"') { format!("\"{}\"", s.replace('\"', "\"\"")) } else { s }
}

// generic helpers
pub fn get_by_path(root: &Value, path: &str) -> Value {
    if path.trim().is_empty() { return Value::Null; }
    let mut cur = root;
    for part in path.split('.').filter(|p| !p.is_empty()) {
        match cur.get(part) { Some(v) => { cur = v; }, None => return Value::Null }
    }
    cur.clone()
}
pub fn rows_from_node(node: &Value) -> Vec<BTreeMap<String, String>> {
    if node.is_null() { return vec![]; }
    if let Some(obj) = node.as_object() {
        if obj.values().all(|v| v.is_object()) {
            return obj.iter().map(|(name, v)| {
                let mut f = flatten(v);
                f.insert("name".into(), name.clone());
                f
            }).collect();
        }
        return vec![flatten(node)];
    }
    vec![{
        let mut m = BTreeMap::new();
        m.insert("value".into(), node.to_string());
        m
    }]
}
fn flatten(v: &Value) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    fn walk(p: &str, v: &Value, out: &mut BTreeMap<String, String>) {
        match v {
            Value::Null => { out.insert(p.to_string(), String::new()); }
            Value::Bool(b) => { out.insert(p.to_string(), b.to_string()); }
            Value::Number(n) => { out.insert(p.to_string(), n.to_string()); }
            Value::String(s) => { out.insert(p.to_string(), s.clone()); }
            Value::Array(a) => {
                out.insert(
                    p.to_string(),
                    a.iter()
                        .map(|x| x.as_str().map(|s| s.to_string()).unwrap_or_else(|| x.to_string()))
                        .collect::<Vec<_>>()
                        .join(" | "),
                );
            }
            Value::Object(obj) => {
                for (k, vv) in obj {
                    let key = if p.is_empty() { k.clone() } else { format!("{p}.{k}") };
                    walk(&key, vv, out);
                }
            }
        }
    }
    walk("", v, &mut out);
    out
}
