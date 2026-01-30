use lib3mf_core::model::Model;
use lib3mf_core::archive::ZipArchiver;
use lib3mf_core::archive::ArchiveReader;
use std::io::Cursor;

#[test]
fn test_write_package_with_attachments() -> anyhow::Result<()> {
    let mut model = Model::default();
    
    // Add a dummy texture attachment
    let texture_data = b"fake png content";
    let texture_path = "3D/Textures/texture.png";
    model.attachments.insert(texture_path.to_string(), texture_data.to_vec());

    // Add a dummy thumbnail
    let thumb_data = b"fake thumbnail";
    let thumb_path = "Metadata/thumbnail.png";
    model.attachments.insert(thumb_path.to_string(), thumb_data.to_vec());

    // Write to buffer
    let mut buffer = Cursor::new(Vec::new());
    model.write(&mut buffer)?;

    // Read back
    let buffer_data = buffer.into_inner();
    let reader = Cursor::new(buffer_data);
    let mut archiver = ZipArchiver::new(reader)?;

    // Verify entries exist
    let entries = archiver.list_entries()?;
    assert!(entries.contains(&"3D/Textures/texture.png".to_string()));
    assert!(entries.contains(&"Metadata/thumbnail.png".to_string()));
    assert!(entries.contains(&"3D/_rels/3dmodel.model.rels".to_string()));

    // Verify content matches
    let read_tex = archiver.read_entry("3D/Textures/texture.png")?;
    assert_eq!(read_tex, texture_data);

    // Verify rels content contains the texture link
    let rels = archiver.read_entry("3D/_rels/3dmodel.model.rels")?;
    let rels_str = String::from_utf8(rels)?;
    assert!(rels_str.contains("Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel/relationship/texture\""));
    assert!(rels_str.contains("Target=\"/3D/Textures/texture.png\""));

    Ok(())
}
