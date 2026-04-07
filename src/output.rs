use comfy_table::{Table, presets::UTF8_FULL_CONDENSED};
use serde_json::Value;

use crate::OutputFormat;

pub fn render_single_record(format: &OutputFormat, fields: &[(&str, String)]) {
    match format {
        OutputFormat::Table => {
            let max_key = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
            for (key, value) in fields {
                println!(
                    "  {:<width$}  {}",
                    format!("{key}:"),
                    value,
                    width = max_key + 1
                );
            }
        }
        OutputFormat::Json => {
            let mut map = serde_json::Map::new();
            for (key, value) in fields {
                map.insert((*key).to_string(), Value::String(value.clone()));
            }
            println!("{}", serde_json::to_string(&Value::Object(map)).unwrap());
        }
        OutputFormat::Csv => {
            eprintln!(
                "Warning: --format csv is not applicable to single records, using table format"
            );
            render_single_record(&OutputFormat::Table, fields);
        }
    }
}

pub fn render_table(format: &OutputFormat, headers: &[&str], rows: &[Vec<String>]) {
    match format {
        OutputFormat::Table => {
            let mut table = Table::new();
            table.load_preset(UTF8_FULL_CONDENSED);
            table.set_header(headers.iter().map(|h| h.to_string()).collect::<Vec<_>>());
            for row in rows {
                table.add_row(row);
            }
            println!("{table}");
        }
        OutputFormat::Json => {
            for row in rows {
                let mut map = serde_json::Map::new();
                for (i, header) in headers.iter().enumerate() {
                    map.insert(
                        (*header).to_string(),
                        Value::String(row.get(i).cloned().unwrap_or_default()),
                    );
                }
                println!("{}", serde_json::to_string(&Value::Object(map)).unwrap());
            }
        }
        OutputFormat::Csv => {
            let stdout = std::io::stdout();
            let mut wtr = csv::Writer::from_writer(stdout.lock());
            let _ = wtr.write_record(headers);
            for row in rows {
                let _ = wtr.write_record(row);
            }
            let _ = wtr.flush();
        }
    }
}

pub fn render_json_value(format: &OutputFormat, value: &Value) {
    match format {
        OutputFormat::Table => {
            println!("{}", serde_json::to_string_pretty(value).unwrap());
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(value).unwrap());
        }
        OutputFormat::Csv => {
            eprintln!("Warning: --format csv is not applicable to this output, using table format");
            println!("{}", serde_json::to_string_pretty(value).unwrap());
        }
    }
}

pub fn render_message(msg: &str) {
    println!("{msg}");
}

pub fn render_table_streaming(
    format: &OutputFormat,
    headers: &[&str],
    rows: impl Iterator<Item = Vec<String>>,
) {
    match format {
        OutputFormat::Table => {
            let rows_vec: Vec<Vec<String>> = rows.collect();
            render_table(format, headers, &rows_vec);
        }
        OutputFormat::Json => {
            for row in rows {
                let mut map = serde_json::Map::new();
                for (i, header) in headers.iter().enumerate() {
                    map.insert(
                        (*header).to_string(),
                        Value::String(row.get(i).cloned().unwrap_or_default()),
                    );
                }
                println!("{}", serde_json::to_string(&Value::Object(map)).unwrap());
            }
        }
        OutputFormat::Csv => {
            let stdout = std::io::stdout();
            let mut wtr = csv::Writer::from_writer(stdout.lock());
            let _ = wtr.write_record(headers);
            for row in rows {
                let _ = wtr.write_record(&row);
            }
            let _ = wtr.flush();
        }
    }
}
