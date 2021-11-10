use shapefile::*;
use std::io::Write;
fn main() {
    let table_builder = dbase::TableWriterBuilder::new()
        .add_character_field(shapefile::dbase::FieldName::try_from("Name").unwrap(), 80)
        .add_integer_field(shapefile::dbase::FieldName::try_from("Id").unwrap())
        .add_numeric_field(
            shapefile::dbase::FieldName::try_from("Value").unwrap(),
            10,
            3,
        );
    let mut writer =
        Writer::from_path("_testdata.shp", table_builder).expect("Could not open shapefile");
    let points = vec![
        Point::new(1.0, 5.0),
        Point::new(5.0, 5.0),
        Point::new(5.0, 1.0),
        Point::new(3.0, 3.0),
        Point::new(1.0, 1.0),
    ];

    let polygon = Polygon::new(PolygonRing::Outer(points));
    let record = TestRecord {
        name: "Jakob".to_string(),
        id: 1,
        value: 10.0,
    };
    writer
        .write_shape_and_record(&polygon, &record)
        .expect("Could write data to shapefile");
}

struct TestRecord {
    name: String,
    id: i32,
    value: f64,
}

impl dbase::WritableRecord for TestRecord {
    fn write_using<'a, W: Write>(
        &self,
        field_writer: &mut shapefile::dbase::FieldWriter<'a, W>,
    ) -> Result<(), shapefile::dbase::FieldIOError> {
        field_writer.write_next_field_value(&self.name)?;
        field_writer.write_next_field_value(&self.id)?;
        field_writer.write_next_field_value(&self.value)?;
        Ok(())
    }
}
