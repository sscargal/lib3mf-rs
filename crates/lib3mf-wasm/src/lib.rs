//! # lib3mf-wasm
//!
//! WebAssembly bindings for lib3mf-rs, enabling browser-based 3MF file processing.
//!
//! ## Overview
//!
//! This crate provides JavaScript-friendly bindings for the lib3mf-core library, compiled to WebAssembly.
//! It enables client-side 3MF file parsing, validation, and analysis in web browsers without server-side processing.
//!
//! ## When to Use This Crate
//!
//! - **Browser-based 3MF viewers**: Display 3D models directly in the browser
//! - **Online validation tools**: Client-side 3MF file validation without uploading to a server
//! - **Web-based model inspection**: Extract metadata, statistics, and geometry information
//! - **Privacy-focused applications**: Process 3MF files entirely on the client side
//!
//! ## JavaScript Usage
//!
//! First, build the WASM module using wasm-pack:
//!
//! ```bash
//! wasm-pack build crates/lib3mf-wasm --target web
//! ```
//!
//! Then use it in JavaScript:
//!
//! ```javascript
//! import init, { WasmModel, set_panic_hook } from './lib3mf_wasm.js';
//!
//! // Initialize the WASM module
//! await init();
//!
//! // Optional: Set up better error messages for debugging
//! set_panic_hook();
//!
//! // Load a 3MF file from a file input
//! const fileInput = document.getElementById('file-input');
//! fileInput.addEventListener('change', async (e) => {
//!     const file = e.target.files[0];
//!     const buffer = await file.arrayBuffer();
//!
//!     try {
//!         // Parse the 3MF file
//!         const model = WasmModel.from_bytes(new Uint8Array(buffer));
//!
//!         // Access model properties
//!         console.log(`Unit: ${model.unit()}`);
//!         console.log(`Objects: ${model.object_count()}`);
//!     } catch (error) {
//!         console.error(`Failed to parse 3MF: ${error}`);
//!     }
//! });
//! ```
//!
//! ## Module Structure
//!
//! This crate exposes a single primary API surface:
//!
//! - [`WasmModel`]: The main wrapper around [`lib3mf_core::Model`], providing JavaScript-accessible methods
//!   for parsing 3MF files and accessing model data.
//! - [`set_panic_hook()`]: Optional panic handler for better error messages in browser console.
//!
//! ## Current Limitations
//!
//! This is an early-stage binding layer with limited API surface. Currently supported:
//!
//! - Parsing 3MF files from byte arrays
//! - Accessing basic model metadata (unit, object count)
//!
//! **Not yet exposed:**
//!
//! - Validation
//! - Geometry access (vertices, triangles)
//! - Materials and textures
//! - Writing/serialization
//!
//! For the full Rust API, see [`lib3mf_core`].

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

/// Set up better panic messages for debugging in browser console.
///
/// This function configures the panic hook to provide detailed error messages
/// when the WASM module panics. Without this, panics will show generic
/// "unreachable executed" messages that are difficult to debug.
///
/// # When to Call
///
/// Call this once during initialization, before any other API calls:
///
/// ```javascript
/// import init, { set_panic_hook } from './lib3mf_wasm.js';
///
/// await init();
/// set_panic_hook();  // Call once at startup
///
/// // Now make other API calls...
/// ```
///
/// # Performance
///
/// This adds a small amount of overhead to panics, but has no impact on normal
/// execution. It's recommended for development builds but can be omitted in
/// production if you want minimal bundle size.
#[wasm_bindgen]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    console_error_panic_hook::set_once();
}

/// WebAssembly wrapper around the core 3MF Model.
///
/// This struct provides JavaScript-accessible methods for working with 3MF files
/// in the browser. It wraps [`lib3mf_core::Model`] and exposes a subset of its
/// functionality through WASM bindings.
///
/// # Primary API Surface
///
/// The main way to use this from JavaScript:
///
/// 1. Parse a 3MF file from bytes using [`WasmModel::from_bytes()`]
/// 2. Query model properties: [`unit()`](WasmModel::unit), [`object_count()`](WasmModel::object_count)
///
/// # JavaScript Usage
///
/// ```javascript
/// const model = WasmModel.from_bytes(fileBytes);
/// console.log(`Unit: ${model.unit()}`);
/// console.log(`Objects: ${model.object_count()}`);
/// ```
///
/// # Full Workflow Example
///
/// ```javascript
/// import init, { WasmModel, set_panic_hook } from './lib3mf_wasm.js';
///
/// await init();
/// set_panic_hook();
///
/// // Load from file input
/// const file = document.getElementById('file-input').files[0];
/// const buffer = await file.arrayBuffer();
/// const model = WasmModel.from_bytes(new Uint8Array(buffer));
///
/// // Display basic info
/// document.getElementById('unit').textContent = model.unit();
/// document.getElementById('count').textContent = model.object_count();
/// ```
#[wasm_bindgen]
pub struct WasmModel {
    inner: lib3mf_core::Model,
}

#[wasm_bindgen]
impl WasmModel {
    /// Create a new empty 3MF Model.
    ///
    /// This creates a model with default values (millimeters unit, no objects, empty build).
    /// In most cases, you should use [`WasmModel::from_bytes()`] instead to parse an existing
    /// 3MF file.
    ///
    /// # JavaScript Usage
    ///
    /// ```javascript
    /// const model = new WasmModel();
    /// // Model is empty - unit is Millimeter by default
    /// ```
    ///
    /// # Note
    ///
    /// This constructor has limited utility since the WASM bindings don't currently expose
    /// model-building APIs. It's primarily for internal use and testing.
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmModel {
        WasmModel {
            inner: lib3mf_core::Model::default(),
        }
    }
}

/// Default implementation delegates to new().
impl Default for WasmModel {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmModel {
    /// Parse a 3MF file from a byte array (e.g. from a file upload).
    ///
    /// This is the primary way to load 3MF files in the browser. It handles the full
    /// parsing pipeline:
    ///
    /// 1. Interprets bytes as a ZIP archive
    /// 2. Parses OPC relationships to locate the model XML
    /// 3. Parses the XML into the in-memory model structure
    /// 4. Returns a [`WasmModel`] ready for inspection
    ///
    /// # Arguments
    ///
    /// * `data` - The bytes of the .3mf file (ZIP archive). Typically obtained from a
    ///   browser file input via `FileReader` or `File.arrayBuffer()`.
    ///
    /// # Returns
    ///
    /// Returns a [`WasmModel`] on success, or throws a JavaScript error on failure.
    ///
    /// # Errors
    ///
    /// This function can fail in several ways:
    ///
    /// - **Invalid ZIP**: The bytes don't represent a valid ZIP archive
    /// - **Missing model part**: The archive lacks a valid OPC relationship to a 3D model
    /// - **Malformed XML**: The model XML is invalid or doesn't conform to the 3MF schema
    /// - **Parser errors**: Semantic errors in the 3MF structure (invalid references, etc.)
    ///
    /// Errors are returned as JavaScript exceptions with descriptive messages.
    ///
    /// # Example (Rust)
    ///
    /// ```ignore
    /// use lib3mf_wasm::WasmModel;
    ///
    /// let file_bytes = std::fs::read("model.3mf")?;
    /// let model = WasmModel::from_bytes(&file_bytes)?;
    /// ```
    ///
    /// # JavaScript Usage
    ///
    /// ```javascript
    /// try {
    ///     const buffer = await file.arrayBuffer();
    ///     const model = WasmModel.from_bytes(new Uint8Array(buffer));
    ///     console.log("Parsed successfully");
    /// } catch (error) {
    ///     console.error(`Parse failed: ${error}`);
    /// }
    /// ```
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
    ///
    /// Returns the unit as a string for display in JavaScript.
    ///
    /// # Possible Return Values
    ///
    /// - `"Millimeter"` (default, most common)
    /// - `"Centimeter"`
    /// - `"Inch"`
    /// - `"Foot"`
    /// - `"Meter"`
    /// - `"MicroMeter"`
    ///
    /// # JavaScript Usage
    ///
    /// ```javascript
    /// const unit = model.unit();
    /// console.log(`Model uses ${unit} units`);
    /// ```
    #[wasm_bindgen]
    pub fn unit(&self) -> String {
        format!("{:?}", self.inner.unit)
    }

    /// Get the total number of objects in the model resources.
    ///
    /// This counts all objects in the model's resource collection, not just
    /// objects referenced by build items.
    ///
    /// # JavaScript Usage
    ///
    /// ```javascript
    /// const count = model.object_count();
    /// console.log(`Model contains ${count} objects`);
    /// ```
    ///
    /// # Note
    ///
    /// Build items may reference the same object multiple times (instances),
    /// so the number of build items may differ from the object count.
    #[wasm_bindgen]
    pub fn object_count(&self) -> usize {
        self.inner.resources.iter_objects().count()
    }
}
