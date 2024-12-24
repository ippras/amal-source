use polars::frame::DataFrame;
use std::sync::LazyLock;

pub(crate) static AGILENT: LazyLock<DataFrame> = LazyLock::new(|| {
    ron::de::from_str(include_str!("Agilent.ron")).expect("deserialize Agilent.ron")
});
