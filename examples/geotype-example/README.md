---
title: `shapefile-rs`: Examples
---

**Requirements**

Enable shapefiles `geo-types`-feature

`Cargo.toml`

```toml
[dependencies]
shapefile = {version = "0.3.0", features = ["geo-types"]}
...
```

# Example how to read a shapefile, convert it to a `geo` data structure and check for polygon-point intersection

The example program in `./src/bin/geotype-contains-example.rs`

1. reads the polygons in `./tests/data/polygons.shp` (ESRI Shapefile) and the points in `./tests/data/points.shp` (ESRI Shapefile)
2. Converts them to the `geo` data-structure
3. Checks which polygons contain which points and
4. Emit a message with the corresponding feature-ids.

# Example how to extract field-values from a polygon-shapefile of a csv-point dataset

The example program in `src/bin/geotype-csv-example.rs`

1. reads a polygon shapefile and point dataset in csv-Format
2. checks which polygon overlaps with the point
3. Adds the polygon-field-data to the point-dataset and updates the point
   field-data if field names are in polygon-shapefile _and_ point-csv-file
4. Writes point-csv-data with additional polygon field-values as output
