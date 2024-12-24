use anyhow::Result;
use polars::prelude::*;
use ron::{extensions::Extensions, ser::PrettyConfig};
use std::{fs::write, iter::empty, path::Path};
use walkdir::WalkDir;
// use special::expressions::fatty_acid::{ExprExt, FattyAcid as _};

fn main() -> Result<()> {
    let data_frame = df! {
        "FattyAcid" => df! {
            "Carbons" => &[
                10u8,
                12,
                14,
                15,
                15,
                16,
                16,
                16,
                17,
                17,
                18,
                18,
                18,
                18,
                18,
                20,
                20,
                21,
                20,
                20,
                20,
                20,
                20,
                22,
                22,
                22,
                23,
                22,
                22,
                22,
                22,
                24,
                24,
            ],
            "Indices" => &[
                Series::from_iter(empty::<i8>()),
                Series::from_iter(empty::<i8>()),
                Series::from_iter(empty::<i8>()),
                Series::from_iter(empty::<i8>()),
                Series::from_iter([None::<i8>]),
                Series::from_iter(empty::<i8>()),
                Series::from_iter([9i8]),
                Series::from_iter([None::<i8>, None::<i8>]),
                Series::from_iter(empty::<i8>()),
                Series::from_iter([None::<i8>]),
                Series::from_iter(empty::<i8>()),
                Series::from_iter([9i8]),
                Series::from_iter([9, 12i8]),
                Series::from_iter([6, 9, 12i8]),
                Series::from_iter([9, 12, 15i8]),
                Series::from_iter(empty::<i8>()),
                Series::from_iter([11i8]),
                Series::from_iter(empty::<i8>()),
                Series::from_iter([11, 14i8]),
                Series::from_iter([8, 11, 14i8]),
                Series::from_iter([5, 8, 11, 14i8]),
                Series::from_iter([11, 14, 17i8]),
                Series::from_iter([5, 8, 11, 14, 17i8]),
                Series::from_iter(empty::<i8>()),
                Series::from_iter([13i8]),
                Series::from_iter([13, 16i8]),
                Series::from_iter(empty::<i8>()),
                Series::from_iter([7, 10, 13, 16i8]),
                Series::from_iter([7, 10, 13, 16, 19i8]),
                Series::from_iter([4, 7, 10, 13, 16i8]),
                Series::from_iter([4, 7, 10, 13, 16, 19i8]),
                Series::from_iter(empty::<i8>()),
                Series::from_iter([None::<i8>]),
            ],
            "Bounds" => &[
                Series::from_iter(empty::<u8>()),
                Series::from_iter(empty::<u8>()),
                Series::from_iter(empty::<u8>()),
                Series::from_iter(empty::<u8>()),
                Series::from_iter([2u8]),
                Series::from_iter(empty::<u8>()),
                Series::from_iter([2u8]),
                Series::from_iter([2u8, 2u8]),
                Series::from_iter(empty::<u8>()),
                Series::from_iter([2u8]),
                Series::from_iter(empty::<u8>()),
                Series::from_iter([2u8]),
                Series::from_iter([2, 2u8]),
                Series::from_iter([2, 2, 2u8]),
                Series::from_iter([2, 2, 2u8]),
                Series::from_iter(empty::<u8>()),
                Series::from_iter([2u8]),
                Series::from_iter(empty::<u8>()),
                Series::from_iter([2, 2u8]),
                Series::from_iter([2, 2, 2u8]),
                Series::from_iter([2, 2, 2, 2u8]),
                Series::from_iter([2, 2, 2u8]),
                Series::from_iter([2, 2, 2, 2, 2u8]),
                Series::from_iter(empty::<u8>()),
                Series::from_iter([2u8]),
                Series::from_iter([2, 2u8]),
                Series::from_iter(empty::<u8>()),
                Series::from_iter([2, 2, 2, 2u8]),
                Series::from_iter([2, 2, 2, 2, 2u8]),
                Series::from_iter([2, 2, 2, 2, 2u8]),
                Series::from_iter([2, 2, 2, 2, 2, 2u8]),
                Series::from_iter(empty::<u8>()),
                Series::from_iter([2u8]),
            ],
        }?.into_struct(PlSmallStr::EMPTY),
        "Median" => [
            Some(0.91f64),
            Some(3.61f64),
            Some(3.50f64),
            Some(0.08f64),
            Some(0.00f64),
            Some(20.22f64),
            Some(2.44f64),
            Some(0.00f64),
            Some(0.19f64),
            Some(0.11f64),
            Some(5.29f64),
            Some(36.96f64),
            Some(20.85f64),
            Some(0.07f64),
            Some(0.83f64),
            Some(0.21f64),
            Some(0.53f64),
            None,
            Some(0.48f64),
            Some(0.38f64),
            Some(0.54f64),
            Some(0.00f64),
            Some(0.17f64),
            Some(0.00f64),
            Some(0.10f64),
            Some(0.07f64),
            Some(0.10f64),
            Some(0.14f64),
            Some(0.12f64),
            Some(0.23f64),
            Some(0.44f64),
            Some(0.10f64),
            Some(0.00f64),
        ],
        "InterquartileRange" => [
            Some(0.38),
            Some(2.16),
            Some(2.32),
            Some(0.11),
            Some(0.06),
            Some(3.29),
            Some(1.44),
            Some(0.08),
            Some(0.07),
            Some(0.07),
            Some(1.55),
            Some(3.31),
            Some(4.60),
            Some(0.18),
            Some(0.67),
            Some(0.17),
            Some(0.10),
            None,
            Some(0.18),
            Some(0.16),
            Some(0.19),
            Some(0.05),
            Some(0.19),
            Some(0.04),
            Some(0.18),
            Some(0.13),
            Some(0.23),
            Some(0.12),
            Some(0.17),
            Some(0.21),
            Some(0.58),
            Some(0.46),
            Some(0.19),
        ],
        "ReferenceRangeMin" => [
            Some(0.52),
            Some(1.23),
            Some(1.20),
            Some(0.00),
            Some(0.00),
            Some(17.02),
            Some(1.12),
            Some(0.00),
            Some(0.00),
            Some(0.00),
            Some(3.89),
            Some(28.50),
            Some(16.58),
            Some(0.00),
            Some(0.46),
            Some(0.12),
            Some(0.38),
            None,
            Some(0.05),
            Some(0.18),
            Some(0.37),
            Some(0.00),
            Some(0.00),
            Some(0.00),
            Some(0.00),
            Some(0.00),
            Some(0.00),
            Some(0.00),
            Some(0.00),
            Some(0.00),
            Some(0.00),
            Some(0.00),
            Some(0.00),
        ],
        "ReferenceRangeMax" => [
            Some(1.64),
            Some(6.41),
            Some(5.29),
            Some(0.18),
            Some(0.14),
            Some(24.39),
            Some(3.47),
            Some(0.21),
            Some(0.28),
            Some(0.19),
            Some(7.37),
            Some(42.37),
            Some(27.29),
            Some(0.20),
            Some(2.04),
            Some(0.53),
            Some(0.95),
            None,
            Some(1.89),
            Some(0.63),
            Some(1.77),
            Some(0.85),
            Some(0.87),
            Some(0.35),
            Some(1.80),
            Some(1.14),
            Some(1.39),
            Some(2.74),
            Some(0.51),
            Some(1.15),
            Some(3.69),
            Some(1.68),
            Some(0.41),
        ],
    }?;
    let data_frame = data_frame
        .clone()
        .lazy()
        .select([
            col("FattyAcid"),
            col("Median"),
            col("InterquartileRange"),
            as_struct(vec![
                col("ReferenceRangeMin").alias("Min"),
                col("ReferenceRangeMax").alias("Max"),
            ])
            .alias("ReferenceRange"),
        ])
        .collect()
        .unwrap();
    println!("data_frame: {}", data_frame);
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
    write("df.amal.ron", contents)?;
    Ok(())
}

fn format(path: &Path) -> Result<()> {
    let source = std::fs::read_to_string(path)?;
    let data_frame: DataFrame = ron::de::from_str(&source)?;
    let formated = ron::ser::to_string_pretty(
        &data_frame,
        PrettyConfig::default().extensions(Extensions::IMPLICIT_SOME),
    )?;
    if source != formated {
        // std::fs::copy(path, format!("{path}.bk"))?;
        std::fs::write(path, formated)?;
    }
    Ok(())
}

#[test]
fn test_format() -> Result<()> {
    for entry in WalkDir::new("input/data/") {
        let entry = entry?;
        if entry.metadata()?.is_file() {
            println!("{}", entry.path().display());
            format(entry.path())?;
        }
    }
    Ok(())
}
