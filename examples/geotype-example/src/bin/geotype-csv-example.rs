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

/// Program to get the polygon-field-value for each point in the csv-file
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Polygon Shapefile
    #[clap(short, long)]
    polygon_file: String,
    /// Field seperator of xyz-point file
    #[clap(short, long, default_value = " ")]
    sep: String,
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
    let polygons: Vec<(Polygon, dbase::Record)> =
        shapefile::read_as::<_, shapefile::Polygon, dbase::Record>(&opts.polygon_file)
            .expect("Could not open shapefile");
    let sep = opts.sep.clone().into_bytes()[0];
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(sep)
        .comment(Some(b'#'))
        .from_reader(input);
    let mut wtr = csv::WriterBuilder::new().delimiter(sep).from_writer(output);

    let mut last_matched_polygon: (geo::MultiPolygon<f64>, &dbase::Record) =
        (polygons[0].0.clone().into(), &polygons[0].1);

    let polygon_fields: &HashMap<String, dbase::FieldValue> = last_matched_polygon.1.as_ref();
    let polygon_fieldnames: Vec<String> = polygon_fields.keys().cloned().collect();

    let point_fieldnames: Vec<String> = rdr.headers()?.into_iter().map(String::from).collect();

    let new_header_vec = vec![point_fieldnames, polygon_fieldnames.clone()].concat();
    let new_header = csv::StringRecord::from(new_header_vec);

    wtr.write_record(&new_header)?;

    for result in rdr.records() {
        let mut csv_record: csv::StringRecord = result?;

        let pt = geo::Point::<f64>::new(
            csv_record
                .get(0)
                .expect("Could not read first point-dataset column")
                .parse()
                .expect("Could not parse x coordinate as numeric"),
            csv_record
                .get(1)
                .expect("Could not read second point-dataset column")
                .parse()
                .expect("Could not parse y coordinate as numeric"),
        );
        if last_matched_polygon.0.contains(&pt) {
            wtr.write_record(&csv_record)?;
        } else {
            for (polygon, polydata) in &polygons {
                let geo_polygon: geo::MultiPolygon<f64> = polygon.clone().into();
                if geo_polygon.contains(&pt) {
                    last_matched_polygon = (geo_polygon, polydata);

                    wtr.write_record(&csv_record)?;
                    continue;
                } else {
                    wtr.write_record(&csv_record)?;
                }
            }
        }
    }
    wtr.flush()?;
    Ok(())
}

fn merge_csv_and_polydata(
    mut record: &csv::StringRecord,
    new_headers: &Vec<String>,
    csv_headers: &Vec<String>,
    poly_headers: &Vec<String>,
    polydata: dbase::Record,
) {
    for k in poly_headers {
        if csv_headers.contains(&k) {
            let i_of_new_headers = new_headers.iter().position(|x| x == k).unwrap();
            let v_of_polydata = polydata.get(&k).unwrap();
            unimplemented!()
        } else {
            unimplemented!()
        }
    }
}
