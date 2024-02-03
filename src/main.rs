use calamine::{Reader, Xlsx, open_workbook};
use clap::{App, Arg};
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
struct Row {
    // Use a vector of BTreeMaps to handle dynamic columns with consistent key order
    Markers: Vec<BTreeMap<String, serde_json::Value>>,
}

fn process_sheet(file_path: &str, sheet_name: &str, verbose: bool) {
    let mut workbook: Xlsx<_> = open_workbook(file_path).expect("Unable to open XLSX file");

    if let Ok(range) = workbook.worksheet_range(sheet_name) {
        let mut rows: Vec<BTreeMap<String, serde_json::Value>> = Vec::new();
        let mut header_order: Vec<String> = Vec::new();

        // Extract headers and build header_order
        let header_row = range.rows().next().unwrap_or_default();
        for header in header_row {
            let header_str = header.get_string().unwrap_or_default().to_string();
            header_order.push(header_str.clone());
        }

        for row in range.rows().skip(1) {
            let mut row_data = BTreeMap::new();

            // Insert values into row_data using header_order
			for (header, cell) in header_order.iter().zip(row.iter()) {
				let cell_value = match *cell {
					calamine::DataType::Int(value) if (value as f64) == value as f64 => {
						serde_json::Value::Number(serde_json::Number::from(value))
					}
					calamine::DataType::Float(value) => {
						serde_json::Value::Number(if value.fract() == 0.0 {
							serde_json::Number::from(value as i64)
						} else {
							serde_json::Number::from_f64(value).expect("Failed to convert number to JSON value")
						})
					}
					_ => {
						let cell_str = cell.get_string().unwrap_or_default().to_string();
						serde_json::Value::String(cell_str)
					}
				};
				row_data.insert(header.clone(), cell_value);
			}

            rows.push(row_data);
        }



        let json_data =
            serde_json::to_string_pretty(&Row { Markers: rows }).expect("Failed to serialize to JSON");

        let json_filename = format!("{}-{}.json", file_path, sheet_name);
        let mut json_file =
            File::create(json_filename.clone()).expect("Unable to create JSON file");
        json_file
            .write_all(json_data.as_bytes())
            .expect("Failed to write to JSON file");

        if verbose {
            println!(
                "Generated JSON file for sheet '{}': {}",
                sheet_name, json_filename
            );
        }
    } else {
        eprintln!("Failed to read sheet '{}' from the workbook.", sheet_name);
    }
}

fn main() {
    let matches = App::new("map-json")
        .version("0.1.0")
        .author("Lyam Witherow")
        .about("Converts XLSX sheets to JSON for Map RCA animations")
        .arg(Arg::with_name("FILE").required(true).help("Sets the input XLSX file"))
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Enables verbose output"),
        )
        .get_matches();

    let file_path = matches.value_of("FILE").unwrap();
    let verbose = matches.is_present("verbose");

    let workbook: Xlsx<_> = open_workbook(file_path).expect("Unable to open XLSX file");

    for sheet_name in workbook.sheet_names() {
        process_sheet(file_path, &sheet_name, verbose);
    }
}
