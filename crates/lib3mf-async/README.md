# lib3mf-async

Asynchronous I/O support for [`lib3mf-core`](https://crates.io/crates/lib3mf-core).

This crate provides async/await compatible interfaces for reading and parsing 3MF files using [Tokio](https://tokio.rs/).

## Features

- Async ZIP archive reading using `async-zip`
- Non-blocking I/O operations
- Compatible with Tokio runtime
- Seamless integration with `lib3mf-core` types

## Example

```rust
use lib3mf_async::AsyncZipArchiver;
use lib3mf_core::archive::{ArchiveReader, find_model_path};
use lib3mf_core::parser::parse_model;
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("model.3mf").await?;
    let mut archiver = AsyncZipArchiver::new(file).await?;

    let model_path = find_model_path(&mut archiver)?;
    let model_data = archiver.read_entry(&model_path).await?;
    let model = parse_model(std::io::Cursor::new(model_data))?;

    println!("Loaded model with {} objects", model.resources.objects.len());
    Ok(())
}
```

## Documentation

See the [full documentation](https://docs.rs/lib3mf-async) for more details.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
