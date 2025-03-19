use std::collections::HashMap;

use infer::MatcherType;

use crate::FileEntry;

/// Convert an infer MatcherType enum to a human-readable string
///
/// # Arguments
/// * `matcher_type` - The MatcherType from infer to convert
///
/// # Returns
/// A String representing the file category
fn matcher_type_to_string(matcher_type: MatcherType) -> String {
    match matcher_type {
        MatcherType::App => "Application".to_string(),
        MatcherType::Archive => "Archive".to_string(),
        MatcherType::Audio => "Audio".to_string(),
        MatcherType::Book => "Book".to_string(),
        MatcherType::Doc => "Document".to_string(),
        MatcherType::Font => "Font".to_string(),
        MatcherType::Image => "Image".to_string(),
        MatcherType::Text => "Text".to_string(),
        MatcherType::Video => "Video".to_string(),
        MatcherType::Custom => "Custom".to_string(),
    }
}

/// Checks if a file extension matches its actual content type
///
/// This function reads the file and attempts to determine its true content type
/// using file signatures (magic numbers). It then compares this to the extension.
///
/// # Arguments
/// * `file` - A reference to a FileEntry to validate
///
/// # Returns
/// A tuple containing:
/// - Boolean: whether the extension correctly matches the content type
/// - Option<String>: the detected MIME type of the file (None if detection failed)
pub fn validate_file_extension(file: &FileEntry) -> (bool, Option<String>) {
    // Try to detect the file type
    let kind = match infer::get_from_path(file.path()) {
        Ok(Some(k)) => k,
        _ => return (true, None), // Couldn't determine type, assume extension is correct
    };
    let mime_type = kind.mime_type().to_string();

    // Get the file extension (if any)
    let extension = match file.extension() {
        Some(ext) => ext.to_lowercase(),
        None => return (false, Some(mime_type)), // No extension but we detected a type
    };

    // Common extension mappings by MIME type
    let valid = match kind.mime_type() {
        // Images
        "image/jpeg" => extension == "jpg" || extension == "jpeg",
        "image/png" => extension == "png",
        "image/gif" => extension == "gif",
        "image/webp" => extension == "webp",
        "image/bmp" => extension == "bmp",

        // Audio
        "audio/mpeg" => extension == "mp3",
        "audio/wav" => extension == "wav",
        "audio/ogg" => extension == "ogg",
        "audio/flac" => extension == "flac",

        // Video
        "video/mp4" => extension == "mp4",
        "video/x-matroska" => extension == "mkv",
        "video/webm" => extension == "webm",
        "video/quicktime" => extension == "mov",
        "video/x-msvideo" => extension == "avi",

        // Documents
        "application/pdf" => extension == "pdf",
        "application/msword" => extension == "doc",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            extension == "docx"
        }
        "application/vnd.ms-excel" => extension == "xls",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => extension == "xlsx",

        // Archives
        "application/zip" => extension == "zip",
        "application/x-rar-compressed" => extension == "rar",
        "application/gzip" => extension == "gz" || extension == "gzip",
        "application/x-7z-compressed" => extension == "7z",

        // For other types, return true by default
        _ => true,
    };

    (valid, Some(mime_type))
}

/// Sorts files into categories based on their content type
///
/// This function analyzes each file's content to determine its true type,
/// regardless of extension, and groups them by category.
///
/// # Arguments
/// * `file_entries` - A vector of FileEntry objects to categorize
///
/// # Returns
/// A HashMap where keys are category names (e.g., "Image", "Document") and
/// values are vectors of FileEntry objects belonging to that category
///
/// # Note
/// Files that cannot be identified will be skipped and not included in the results
pub fn identify_files(file_entries: Vec<FileEntry>) -> HashMap<String, Vec<FileEntry>> {
    let mut identified_files = HashMap::new();

    for file in file_entries {
        // Attempt to identify file type, skip files that can't be identified
        match infer::get_from_path(file.path()) {
            Err(e) => {
                eprintln!("Error identifying file: {:?}", e);
                continue;
            }
            Ok(Some(kind)) => {
                let category = matcher_type_to_string(kind.matcher_type());
                identified_files
                    .entry(category)
                    .or_insert_with(Vec::new)
                    .push(file);
            }
            Ok(None) => {
                eprintln!("Could not identify file: {:?}", file.path());
                identified_files
                    .entry("Unknown".to_string())
                    .or_insert_with(Vec::new)
                    .push(file);
            }
        }
    }

    identified_files
}

/// Identifies files with incorrect or misleading extensions
///
/// This function checks each file to determine if its extension accurately
/// reflects its actual content type. Files without extensions or with
/// extensions that don't match their content are reported.
///
/// # Arguments
/// * `file_entries` - A slice of FileEntry objects to check
///
/// # Returns
/// A vector of tuples containing:
/// - The FileEntry with mismatched extension
/// - A String containing the actual MIME type of the file
pub fn find_mismatched_extensions(file_entries: &[FileEntry]) -> Vec<(FileEntry, String)> {
    let mut mismatched = Vec::new();

    for file in file_entries {
        let (is_valid, mime_type) = validate_file_extension(file);

        if !is_valid {
            if let Some(mime) = mime_type {
                mismatched.push((file.clone(), mime));
            }
        }
    }

    mismatched
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;

    /// Helper function to create a test FileEntry
    fn create_test_file_entry(path: &PathBuf) -> FileEntry {
        // Use From<DirEntry> implementation that's available
        let dir_entry = walkdir::WalkDir::new(path)
            .into_iter()
            .next()
            .unwrap()
            .unwrap();

        FileEntry::from(dir_entry)
    }

    #[test]
    fn test_validate_file_extension() {
        let temp_dir = tempdir().unwrap();

        // Test case 1: JPEG file with correct extension
        let jpeg_correct_path = temp_dir.path().join("correct.jpg");
        let mut jpeg_file = File::create(&jpeg_correct_path).unwrap();
        // Write JPEG file header bytes
        let jpeg_header = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46];
        jpeg_file.write_all(&jpeg_header).unwrap();

        // Test case 2: JPEG file with incorrect extension
        let jpeg_wrong_path = temp_dir.path().join("wrong.png");
        let mut jpeg_wrong_file = File::create(&jpeg_wrong_path).unwrap();
        jpeg_wrong_file.write_all(&jpeg_header).unwrap();

        // Test case 3: PNG file with correct extension
        let png_correct_path = temp_dir.path().join("correct.png");
        let mut png_file = File::create(&png_correct_path).unwrap();
        // Write PNG file header bytes
        let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        png_file.write_all(&png_header).unwrap();

        // Test case 4: PNG file with incorrect extension
        let png_wrong_path = temp_dir.path().join("wrong.jpg");
        let mut png_wrong_file = File::create(&png_wrong_path).unwrap();
        png_wrong_file.write_all(&png_header).unwrap();

        // Test case 5: PDF file with correct extension
        let pdf_correct_path = temp_dir.path().join("doc.pdf");
        let mut pdf_file = File::create(&pdf_correct_path).unwrap();
        // Write PDF file header bytes
        let pdf_header = [0x25, 0x50, 0x44, 0x46, 0x2D, 0x31, 0x2E]; // %PDF-1.
        pdf_file.write_all(&pdf_header).unwrap();

        // Test case 6: File with no extension
        let no_ext_path = temp_dir.path().join("noextension");
        let mut no_ext_file = File::create(&no_ext_path).unwrap();
        no_ext_file.write_all(&jpeg_header).unwrap();

        // Now test all the cases
        let jpeg_correct_entry = create_test_file_entry(&jpeg_correct_path);
        let (is_valid, mime_type) = validate_file_extension(&jpeg_correct_entry);
        assert!(is_valid, "JPEG file with jpg extension should be valid");
        assert_eq!(mime_type, Some("image/jpeg".to_string()));

        let jpeg_wrong_entry = create_test_file_entry(&jpeg_wrong_path);
        let (is_valid, mime_type) = validate_file_extension(&jpeg_wrong_entry);
        assert!(!is_valid, "JPEG file with png extension should be invalid");
        assert_eq!(mime_type, Some("image/jpeg".to_string()));

        let png_correct_entry = create_test_file_entry(&png_correct_path);
        let (is_valid, mime_type) = validate_file_extension(&png_correct_entry);
        assert!(is_valid, "PNG file with png extension should be valid");
        assert_eq!(mime_type, Some("image/png".to_string()));

        let png_wrong_entry = create_test_file_entry(&png_wrong_path);
        let (is_valid, mime_type) = validate_file_extension(&png_wrong_entry);
        assert!(!is_valid, "PNG file with jpg extension should be invalid");
        assert_eq!(mime_type, Some("image/png".to_string()));

        let pdf_correct_entry = create_test_file_entry(&pdf_correct_path);
        let (is_valid, mime_type) = validate_file_extension(&pdf_correct_entry);
        assert!(is_valid, "PDF file with pdf extension should be valid");
        assert_eq!(mime_type, Some("application/pdf".to_string()));

        let no_ext_entry = create_test_file_entry(&no_ext_path);
        let (is_valid, mime_type) = validate_file_extension(&no_ext_entry);
        assert!(!is_valid, "File without extension should be invalid");
        assert_eq!(mime_type, Some("image/jpeg".to_string()));
    }

    #[test]
    fn test_find_mismatched_extensions() {
        let temp_dir = tempdir().unwrap();

        // Create one correct and one incorrect file
        let correct_path = temp_dir.path().join("correct.jpg");
        let mut correct_file = File::create(&correct_path).unwrap();
        let jpeg_header = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46];
        correct_file.write_all(&jpeg_header).unwrap();

        let wrong_path = temp_dir.path().join("wrong.png");
        let mut wrong_file = File::create(&wrong_path).unwrap();
        wrong_file.write_all(&jpeg_header).unwrap();

        let correct_entry = create_test_file_entry(&correct_path);
        let wrong_entry = create_test_file_entry(&wrong_path);

        let file_entries = vec![correct_entry.clone(), wrong_entry.clone()];

        let mismatched = find_mismatched_extensions(&file_entries);

        assert_eq!(
            mismatched.len(),
            1,
            "Should find exactly one mismatched file"
        );
        assert_eq!(mismatched[0].0.path(), wrong_entry.path());
        assert_eq!(mismatched[0].1, "image/jpeg");
    }

    #[test]
    fn test_identify_files() {
        let temp_dir = tempdir().unwrap();

        // Create image file
        let image_path = temp_dir.path().join("image.jpg");
        let mut image_file = File::create(&image_path).unwrap();
        let jpeg_header = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46];
        image_file.write_all(&jpeg_header).unwrap();

        // Create PDF file
        let pdf_path = temp_dir.path().join("document.pdf");
        let mut pdf_file = File::create(&pdf_path).unwrap();
        let pdf_header = [0x25, 0x50, 0x44, 0x46, 0x2D, 0x31, 0x2E]; // %PDF-1.
        pdf_file.write_all(&pdf_header).unwrap();

        let image_entry = create_test_file_entry(&image_path);
        let pdf_entry = create_test_file_entry(&pdf_path);

        let file_entries = vec![image_entry.clone(), pdf_entry.clone()];

        let identified = identify_files(file_entries);

        assert!(
            identified.contains_key("Image"),
            "Should identify an image file"
        );
        assert!(
            identified.contains_key("Archive"),
            "Should identify an archive file"
        );

        assert_eq!(identified["Image"].len(), 1);
        assert_eq!(identified["Archive"].len(), 1);

        assert_eq!(identified["Image"][0].path(), image_entry.path());
        assert_eq!(identified["Archive"][0].path(), pdf_entry.path());
    }
}
