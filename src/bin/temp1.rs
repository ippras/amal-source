use anyhow::Result;
use polars::prelude::*;
use ron::{extensions::Extensions, ser::PrettyConfig};
use std::{fs::write, iter::empty, path::Path};
use walkdir::WalkDir;

// lazy_frame = lazy_frame
// .with_row_index("IDX", None)
// .unnest([col("FA")])
// .explode([col("Indices"), col("Bounds")])
// .with_columns([
//     col("Indices").alias("Index"),
//     col("Bounds").sign().alias("Isomerism"),
//     col("Bounds")
//         .abs()
//         .cast(DataType::UInt8)
//         .alias("Unsaturation"),
// ]);
// println!("lazy_frame1: {}", lazy_frame.clone().collect().unwrap());
// lazy_frame = lazy_frame.group_by(["IDX"]).agg([
// col("Mode").first(),
// col("Carbons").first(),
// as_struct(vec![
//     col("Index"),
//     col("Isomerism"),
//     col("Unsaturation"),
// ])
// .alias("Unsaturated"),
// col("Label").first(),
// col("Time").first(),
// ]);
// println!("lazy_frame2: {}", lazy_frame.clone().collect().unwrap());
// lazy_frame = lazy_frame.cache().with_columns([
// when(
//     col("Unsaturated").eq(concat_list([as_struct(vec![
//         lit(NULL).alias("Index"),
//         lit(NULL).alias("Isomerism"),
//         lit(NULL).alias("Unsaturation"),
//     ])])
//     .unwrap()),
// )
// .then(lit(Scalar::new(
//     DataType::List(Box::new(DataType::Null)),
//     AnyValue::List(Series::new_empty(
//         PlSmallStr::EMPTY,
//         &DataType::Null,
//     )),
// )))
// // .then(lit(NULL))
// .otherwise(col("Unsaturated"))
// .alias("Unsaturated"),
// // true,
// ]);
// // .with_columns([col("Unsaturated").list().eval(
// //     when(col("").eq(as_struct(vec![
// //         lit(NULL).alias("Index"),
// //         lit(NULL).alias("Isomerism"),
// //         lit(NULL).alias("Unsaturation"),
// //     ])))
// //     .then(lit(Scalar::new(
// //         DataType::List(Box::new(DataType::Null)),
// //         AnyValue::List(Series::new_empty(PlSmallStr::EMPTY, &DataType::Null)),
// //     )))
// //     // .then(lit(NULL))
// //     .otherwise(col("")),
// //     true,
// // )]);
// println!("lazy_frame3: {}", lazy_frame.clone().collect().unwrap());
// lazy_frame = lazy_frame.select([
// col("IDX").alias("Index"),
// col("Mode"),
// as_struct(vec![col("Carbons"), col("Unsaturated")])
//     .alias("FattyAcid"),
// col("Label"),
// col("Time"),
// ]);
// println!("lazy_frame4: {}", lazy_frame.clone().collect().unwrap());
// let data_frame = lazy_frame.clone().collect().unwrap();
// data::save("df.ron", data::Format::Ron, data_frame).unwrap();
// std::process::exit(0);
fn main() -> Result<()> {
    // let content = include_str!("../../df.ron");
    let data_frame: DataFrame = ron::de::from_str(content).unwrap();
    println!("data_frame: {data_frame}");
    // println!(
    //     "data_frame: {}",
    //     data_frame
    //         .clone()
    //         .lazy()
    //         .with_columns([
    //             col("FA").fa().ecn().alias("ECN"),
    //             col("FA").fa().mass().alias("Mass"),
    //             col("FA").fa().rcooch3().mass().alias("MethylEsterMass"),
    //             col("FA").fa().rcoo().mass().alias("RCOOMass"),
    //             // col("Time").list().mean().alias("Mean"),
    //             // col("Time").list().std(0).alias("StandardDeviation"),
    //             // col("FA").fa().unsaturation().alias("Unsaturation"),
    //         ])
    //         .collect()?
    // );
    // println!("data_frame: {}", data_frame.unnest(["FA"]).unwrap());
    let contents = ron::ser::to_string_pretty(
        &data_frame,
        PrettyConfig::default().extensions(Extensions::IMPLICIT_SOME),
    )?;
    write("df.out.ron", contents)?;
    Ok(())
}
