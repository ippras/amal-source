#[cfg(not(target_arch = "wasm32"))]
pub use self::native::save;
#[cfg(target_arch = "wasm32")]
pub use self::web::save;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use anyhow::Result;
    use metadata::{IpcWriterExt as _, Metadata};
    use polars::prelude::*;
    use std::fs::File;

    pub fn save(name: &str, meta: Option<&Metadata>, data: &mut DataFrame) -> Result<()> {
        let file = File::create(name)?;
        let mut writer = IpcWriter::new(file);
        if let Some(meta) = meta {
            writer.metadata(meta.clone());
        }
        writer.finish(data)?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use anyhow::{Result, anyhow};
    use eframe::{
        wasm_bindgen::{JsCast, JsValue, prelude::*},
        web_sys::{File, FilePropertyBag, HtmlAnchorElement, Url, window},
    };
    use gloo_utils::errors::JsError;
    use js_sys::{Array, ArrayBuffer, Uint8Array};
    use metadata::{IpcWriterExt as _, Metadata};
    use polars::prelude::*;

    pub fn save(name: &str, meta: Option<&Metadata>, data: &mut DataFrame) -> Result<()> {
        let mut bytes = Vec::new();
        let mut writer = IpcWriter::new(&mut bytes);
        if let Some(meta) = meta {
            writer.metadata(meta.clone());
        }
        writer.finish(data)?;
        download(name, &bytes).map_err(|error| {
            anyhow!(match JsError::try_from(error) {
                Ok(error) => error.to_string(),
                Err(error) => error.to_string(),
            })
        })
    }

    /// * https://stackoverflow.com/questions/3665115/how-to-create-a-file-in-memory-for-user-to-download-but-not-through-server
    /// * https://stackoverflow.com/questions/69556755/web-sysurlcreate-object-url-with-blobblob-not-formatting-binary-data-co
    /// * https://github.com/emilk/egui/discussions/3571
    fn download(name: &str, bytes: &[u8]) -> Result<(), JsValue> {
        let window = window().expect("window not found");
        let document = window.document().expect("document not found");
        let body = document.body().expect("body not found");

        let output: HtmlAnchorElement = document.create_element("a")?.dyn_into()?;
        output.style().set_property("display", "none")?;
        output.set_href(&file(name, bytes)?);
        output.set_download(name);
        output.click();
        Ok(())
    }

    fn file(name: &str, bytes: &[u8]) -> Result<String, JsValue> {
        let array = Uint8Array::from(bytes);
        let bits = Array::new();
        bits.push(&array.buffer());
        let mut options = FilePropertyBag::new();
        options.set_type("application/octet-stream");
        let file = File::new_with_blob_sequence_and_options(&bits, name, &options)?;
        Ok(Url::create_object_url_with_blob(&file)?)
    }
}
