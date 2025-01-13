use metadata::MetaDataFrame;
use std::{io::Cursor, sync::LazyLock};

pub(crate) static AGILENT: LazyLock<MetaDataFrame> = LazyLock::new(|| {
    let bytes = include_bytes!("Agilent.ipc");
    MetaDataFrame::read(Cursor::new(bytes)).expect("read metadata Agilent.ipc")
});
