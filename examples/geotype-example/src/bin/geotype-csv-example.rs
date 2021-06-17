use clap::{AppSettings, Clap};
use geo::prelude::Contains;
use shapefile::dbase;
use shapefile::Polygon;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::process;

// Command-Line Arguments

/// Program to get the polygon-field-value for each point in the csv-file
///
/// The program
/// 1. reads a polygon shapefile and point dataset in csv-Format
/// 2. checks which polygon overlaps with the point
/// 3. Adds the polygon-field-data to the point-dataset and updates the point
///    field-data if field names are in polygon-shapefile _and_ point-csv-file
/// 4. Writes point-csv-data with additional polygon field-values as output

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Polygon Shapefile
    #[clap(short, long)]
    polygon_file: String,
    /// Field seperator of xyz-point file
    #[clap(short, long, default_value = " ")]
    sep: String,
    /// Value in output for missing (non overlapping) data
    #[clap(short, long, default_value = "NA")]
    na: String,
    /// Optional output xyz-file else stdout
    #[clap(short, long)]
    output: Option<String>,
    /// xyz-point file with header-line
    input: Option<String>,
}

fn main() {
    let opts: Opts = Opts::parse();

    let exit = match (&opts.input, &opts.output) {
        (None, None) => run(io::stdin(), io::stdout(), &opts),
        (None, Some(f)) => run(io::stdin(), create_output_file(f), &opts),
        (Some(f), None) => run(open_input_file(f), io::stdout(), &opts),
        (Some(fin), Some(fout)) => run(open_input_file(fin), create_output_file(fout), &opts),
    };
    if let Err(err) = exit {
        println!("{}", err);
        process::exit(1);
    }
}

fn open_input_file(f: &String) -> File {
    File::open(f).expect("Can't open input-file")
}

fn create_output_file(f: &String) -> File {
    File::create(f).expect("Can't open output-file")
}

fn run(input: impl Read, output: impl Write, opts: &Opts) -> Result<(), Box<dyn Error>> {
    // Read the polygon shapefile
    let polygons: Vec<(Polygon, dbase::Record)> =
        shapefile::read_as::<_, shapefile::Polygon, dbase::Record>(&opts.polygon_file)
            .expect("Could not open shapefile");

    // Field delimiter of csv-file is specified on the command-line
    let sep = opts.sep.clone().into_bytes()[0];

    // Specifies the csv-reader options
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(sep)
        .comment(Some(b'#'))
        .from_reader(input);

    // Specifies the csv-reader options
    let mut wtr = csv::WriterBuilder::new().delimiter(sep).from_writer(output);

    // Cache the last matched polygon; initialize with the first polygon with (shape, data) tuple
    let mut last_matched_polygon: (geo::MultiPolygon<f64>, dbase::Record) =
        (polygons[0].0.clone().into(), polygons[0].1.clone());

    // Get Polygon-dataset fieldnames
    let polygon_record_hashmap: HashMap<String, dbase::FieldValue> =
        last_matched_polygon.1.clone().into();
    let polygon_fieldnames: Vec<String> = polygon_record_hashmap.keys().cloned().collect();

    // Get Point-dataset fieldnames
    let point_fieldnames: Vec<String> = rdr.headers()?.into_iter().map(String::from).collect();

    // New headerline of output dataset
    //let new_header_vec = vec![point_fieldnames.clone(), polygon_fieldnames.clone()].concat();
    let mut new_header_vec: Vec<String> = point_fieldnames.clone();
    for k in &polygon_fieldnames {
        if !&point_fieldnames.contains(k) {
            new_header_vec.push(k.clone())
        }
    }
    let new_header = csv::StringRecord::from(new_header_vec.clone());

    // Write header line to output
    wtr.write_record(&new_header)?;

    // Iterate over csv-input points
    for result in rdr.records() {
        let csv_in_record: csv::StringRecord = result?;
        // Create a HashMap with key-value pairs
        let mut value_hash_map: HashMap<String, String> = HashMap::new();
        for k in &point_fieldnames {
            let v = get_value_from_string_record(&csv_in_record, &point_fieldnames, &k)
                .unwrap()
                .to_string();
            value_hash_map.insert(k.to_string(), v);
        }

        for k in &polygon_fieldnames {
            value_hash_map.insert(k.to_string(), opts.na.clone());
        }

        let pt = geo::Point::<f64>::new(
            csv_in_record
                .get(0)
                .expect("Could not read first point-dataset column")
                .parse()
                .expect("Could not parse x coordinate as numeric"),
            csv_in_record
                .get(1)
                .expect("Could not read second point-dataset column")
                .parse()
                .expect("Could not parse y coordinate as numeric"),
        );
        if last_matched_polygon.0.contains(&pt) {
            update_value_hash_map(
                &mut value_hash_map,
                &polygon_fieldnames,
                &last_matched_polygon.1,
                &opts.na,
            );
        } else {
            for (polygon, polydata) in &polygons {
                let geo_polygon: geo::MultiPolygon<f64> = polygon.clone().into();
                if geo_polygon.contains(&pt) {
                    last_matched_polygon = (geo_polygon, polydata.clone());
                    update_value_hash_map(
                        &mut value_hash_map,
                        &polygon_fieldnames,
                        &last_matched_polygon.1,
                        &opts.na,
                    );
                    break;
                }
            }
        }
        let csv_out_record_vec: Vec<String> = new_header_vec
            .iter()
            .map(|k| value_hash_map.get(k).unwrap().clone())
            .collect();
        let csv_out_record: csv::StringRecord = csv::StringRecord::from(csv_out_record_vec);
        wtr.write_record(&csv_out_record)?;
    }
    wtr.flush()?;
    Ok(())
}

fn get_fieldvalue_as_string(r: &dbase::Record, k: &String, na_str: &String) -> String {
    match r.get(k) {
        Some(dbase::FieldValue::Character(Some(s))) => s.clone(),
        Some(dbase::FieldValue::Numeric(Some(f))) => format!("{}", f),
        _ => na_str.clone(),
    }
}

fn update_value_hash_map(
    value_hash_map: &mut HashMap<String, String>,
    polygon_fieldnames: &Vec<String>,
    polygon: &dbase::Record,
    na_str: &String,
) {
    for k in polygon_fieldnames {
        value_hash_map.insert(k.to_string(), get_fieldvalue_as_string(polygon, k, na_str));
    }
}
fn get_value_from_string_record<'a>(
    r: &'a csv::StringRecord,
    record_names: &Vec<String>,
    x: &String,
) -> Option<&'a str> {
    let i = record_names.iter().position(|e| e == x)?;
    r.get(i)
}
