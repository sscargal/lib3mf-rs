use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    console_error_panic_hook::set_once();
}

/// Wrapper around the core 3MF Model.
#[wasm_bindgen]
pub struct WasmModel {
    inner: lib3mf_core::Model,
}

#[wasm_bindgen]
impl WasmModel {
    /// Create a new empty 3MF Model.
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmModel {
        WasmModel {
            inner: lib3mf_core::Model::default(),
        }
    }
}

impl Default for WasmModel {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmModel {
    /// Parse a 3MF file from a byte array (e.g. from a file upload).
    ///
    /// # Arguments
    /// * `data` - The bytes of the .3mf (zip) file.
    #[wasm_bindgen]
    pub fn from_bytes(data: &[u8]) -> Result<WasmModel, JsError> {
        use lib3mf_core::{
            archive::{ArchiveReader, ZipArchiver, find_model_path},
            parser::parse_model,
        };
        use std::io::Cursor;

        let cursor = Cursor::new(data.to_vec());
        let mut archiver = ZipArchiver::new(cursor).map_err(|e| JsError::new(&e.to_string()))?;

        let model_path =
            find_model_path(&mut archiver).map_err(|e| JsError::new(&e.to_string()))?;

        let model_data = archiver
            .read_entry(&model_path)
            .map_err(|e| JsError::new(&e.to_string()))?;

        let cursor_xml = Cursor::new(model_data);
        let model = parse_model(cursor_xml).map_err(|e| JsError::new(&e.to_string()))?;

        Ok(WasmModel { inner: model })
    }

    /// Get the unit of measurement used in the model.
    #[wasm_bindgen]
    pub fn unit(&self) -> String {
        format!("{:?}", self.inner.unit)
    }

    /// Get the total number of objects in the model resources.
    #[wasm_bindgen]
    pub fn object_count(&self) -> usize {
        self.inner.resources.iter_objects().count()
    }
}
