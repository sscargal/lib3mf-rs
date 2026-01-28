use lib3mf_core::archive::ZipArchiver;
use lib3mf_core::error::Lib3mfError;
use std::fs::File;
use std::path::PathBuf;

#[test]
fn test_error_file_not_found() {
    let path = PathBuf::from("non_existent_file.3mf");
    let result = File::open(&path);
    // This assumes we are using std::fs::File directly.
    // The library currently takes a Reader, so the error happens *before* lib3mf.
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    println!("File Not Found Error: {}", err);
}

#[test]
fn test_error_is_directory() {
    // Attempt to open the current directory as a file
    let path = PathBuf::from(".");
    // Note: On some OS `File::open` on dir might succeed but reading fails, or fail immediately.
    // On Linux it often succeeds opening, but `read` fails with IsADirectory.
    let file_result = File::open(&path);

    if let Ok(file) = file_result {
        // Now try to trigger error in ZipArchiver
        let archiver_result = ZipArchiver::new(file);
        assert!(archiver_result.is_err());
        match archiver_result.unwrap_err() {
            Lib3mfError::Io(e) => {
                println!("Got expected IO error for directory: {}", e);
                // On Linux reading a dir usually gives EISDIR
                // But zip crate might just fail to read magic signature.
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    } else {
        println!("File::open failed for directory (expected on some OS)");
    }
}

#[test]
fn test_error_invalid_zip() {
    // Create a dummy file that is not a zip
    let path = std::env::temp_dir().join("test_invalid.3mf");
    std::fs::write(&path, "This is not a zip file").unwrap();
    let file = File::open(&path).unwrap();

    let result = ZipArchiver::new(file);
    assert!(result.is_err());
    match result.unwrap_err() {
        Lib3mfError::Io(e) => {
            // Zip crate wrapper might return Io error with specific kind or custom error
            println!("Got expected error for invalid zip: {}", e);
        }
        // Or it might be a library wrapper
        e => panic!("Unexpected error type: {:?}", e),
    }
    let _ = std::fs::remove_file(path);
}
