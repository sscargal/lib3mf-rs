use lib3mf_core::archive::ArchiveReader;
use lib3mf_core::model::Model;
use lib3mf_core::parser::parse_model;
use lib3mf_core::utils::diff::compare_models;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("Usage: model_diff <file1.3mf> <file2.3mf>");
        return Ok(());
    }

    let file1 = PathBuf::from(&args[1]);
    let file2 = PathBuf::from(&args[2]);

    println!("Loading models...");
    let model1 = load_model(&file1)?;
    let model2 = load_model(&file2)?;

    println!("Calculating differences...");
    let diff = compare_models(&model1, &model2);

    if diff.is_empty() {
        println!("No differences found!");
    } else {
        println!("{:#?}", diff);
    }

    Ok(())
}

fn load_model(path: &PathBuf) -> anyhow::Result<Model> {
    let file = std::fs::File::open(path)?;
    let mut archiver = lib3mf_core::archive::ZipArchiver::new(file)?;
    let model_path = lib3mf_core::archive::find_model_path(&mut archiver)?;
    let model_data = archiver.read_entry(&model_path)?;
    Ok(parse_model(std::io::Cursor::new(model_data))?)
}
