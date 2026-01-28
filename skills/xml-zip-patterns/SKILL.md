---
name: xml-zip-patterns
description: Efficient XML parsing patterns and ZIP handling strategies.
---

# XML & ZIP Patterns

Best practices for File I/O in lib3mf-rs.

## ZIP Archive Handling
- **Crate**: `zip`
- **Reading**:
  - Use `ZipArchive<R>`.
  - Locate `_rels/.rels` first to find the root model.
  - **Do not assume** `3D/3dmodel.model` path. Follow the relationships.
- **Writing**:
  - Use `ZipWriter<W>`.
  - Set compression method to `Deflated` (usually).
  - Ensure `[Content_Types].xml` is written.

## XML Parsing Strategy
- **Crate**: `quick-xml`
- **Mode**: Streaming Event Reader (`Reader::from_reader`).
- **Buffer Management**: Reuse buffers to minimize allocation.

### Streaming Pattern
```rust
let mut reader = Reader::from_reader(buf_reader);
let mut buf = Vec::new();

loop {
    match reader.read_event_into(&mut buf) {
        Ok(Event::Start(ref e)) => {
            match e.name().as_ref() {
                b"mesh" => parse_mesh(&mut reader)?,
                _ => (),
            }
        }
        Ok(Event::End(ref e)) => { ... }
        Ok(Event::Eof) => break,
        Err(e) => return Err(e),
        _ => (),
    }
    buf.clear();
}
```

## Namespace Handling
3MF uses namespaces heavily.
- **Pattern**: Resolve namespace URIs, don't just rely on prefixes (prefixes can be arbitrary).
- `quick-xml` provides namespace support. Keep a stack of active namespaces if manual handling is required, or use the iterator with namespace support.

## Error Recovery
- Attempt to continue if non-critical structure is malformed (only if configured to "permissive").
- Strict mode should fail fast.
