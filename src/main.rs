// SPDX-License-Identifier: MIT
use eframe::{egui, egui::TextEdit};
use parking_lot::Mutex;
use std::sync::Arc;

mod parser;
mod exporters;
use exporters::*;
use parser::parse_forti;

#[derive(Default)]
struct AppState {
    input: String,
    parsed: Option<serde_json::Value>,
    filter: String,
    yaml: bool,
    generic_path: String,
    status: String,
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    let state = Arc::new(Mutex::new(AppState::default()));
    eframe::run_native("SamHan-fgt-parser", options, Box::new(move |_cc| {
        Box::new(MyApp { state: state.clone() })
    }))
}

struct MyApp { state: Arc<Mutex<AppState>> }
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let mut s = self.state.lock();
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("Open .conf…").clicked() {
                    if let Some(path) = rfd::FileDialog::new().add_filter("FortiGate conf", &["conf","txt"]).pick_file() {
                        if let Ok(t) = std::fs::read_to_string(&path) { s.input = t; s.status = format!("Loaded {}", path.display()); }
                    }
                }
                if ui.button("Parse").clicked() {
                    match parse_forti(&s.input) { Ok(v) => { s.parsed = Some(v); s.status = "Parsed".into(); }, Err(e) => s.status = format!("Error: {e}") }
                }
                ui.separator();
                ui.label("Search:"); ui.add(TextEdit::singleline(&mut s.filter).desired_width(180.0));
                ui.checkbox(&mut s.yaml, "YAML output");
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |cols| {
                cols[0].heading("Input"); cols[0].add(TextEdit::multiline(&mut s.input).desired_rows(28).code_editor());
                ui_exporters(&mut s, &mut cols[0]);
                cols[1].heading("Output");
                let out = render_output(s.parsed.as_ref(), &s.filter, s.yaml);
                cols[1].add(TextEdit::multiline(&mut out.clone()).desired_rows(32).code_editor().interactive(false));
                cols[1].label(&s.status);
            });
        });
    }
}

fn render_output(p: Option<&serde_json::Value>, q: &str, yaml: bool) -> String {
    let Some(p) = p else { return String::new(); };
    let f = if q.trim().is_empty() { p.clone() } else { filter_deep(p, q) };
    if yaml { serde_yaml::to_string(&f).unwrap_or_default() } else { serde_json::to_string_pretty(&f).unwrap_or_default() }
}

fn filter_deep(v: &serde_json::Value, q: &str) -> serde_json::Value {
    let ql = q.to_lowercase();
    match v {
        serde_json::Value::Object(m) => {
            let mut out = serde_json::Map::new();
            for (k, vv) in m {
                let c = filter_deep(vv, q);
                if !c.is_null() || k.to_lowercase().contains(&ql) { out.insert(k.clone(), if c.is_null(){ vv.clone() } else { c }); }
            }
            if out.is_empty() { serde_json::Value::Null } else { serde_json::Value::Object(out) }
        }
        serde_json::Value::Array(a) => {
            let mut out = Vec::new();
            for it in a { let c = filter_deep(it, q); if !c.is_null() { out.push(c); } }
            if out.is_empty() { serde_json::Value::Null } else { serde_json::Value::Array(out) }
        }
        _ => {
            let s = v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string());
            if s.to_lowercase().contains(&ql) { v.clone() } else { serde_json::Value::Null }
        }
    }
}

fn ui_exporters(s: &mut AppState, ui: &mut egui::Ui) {
    ui.collapsing("CSV exports", |ui| {
        ui.horizontal(|ui| {
            if ui.button("Addresses v4").clicked() {
                if let Some(ref p) = s.parsed {
                    let rows = extract_addresses(p);
                    let csv = to_csv(&rows, Some(&["name","type","subnet","fqdn","interface","comment"]));
                    save_csv("addresses.csv", &csv);
                    s.status = format!("Exported {}", rows.len());
                }
            }
            if ui.button("Policies v4").clicked() {
                if let Some(ref p) = s.parsed {
                    let rows = extract_policies(p);
                    let csv = to_csv(&rows, Some(&["id","name","action","srcintf","dstintf","srcaddr","dstaddr","service","schedule","nat","logtraffic","comments"]));
                    save_csv("policies.csv", &csv);
                    s.status = format!("Exported {}", rows.len());
                }
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Generic path:");
            ui.add(egui::TextEdit::singleline(&mut s.generic_path).desired_width(240.0));
            if ui.button("Export").clicked() {
                if let Some(ref p) = s.parsed {
                    let node = get_by_path(p, &s.generic_path);
                    let rows = rows_from_node(&node);
                    let csv = to_csv(&rows, None);
                    save_csv("generic.csv", &csv);
                    s.status = format!("Exported {}", rows.len());
                }
            }
        });
    });
}

fn save_csv(file: &str, content: &str) {
    if let Some(path) = rfd::FileDialog::new().set_file_name(file).save_file() {
        let _ = std::fs::write(path, content);
    }
}
