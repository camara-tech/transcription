#!/usr/bin/env rust-script
// cargo-deps: tempfile="3.2", dirs="4.0", log="0.4", env_logger="0.9"

// To run this script, you need to have Rust and Cargo installed.
// You can install Rust and Cargo by following the instructions at https://www.rust-lang.org/tools/install
// Additionally, you need to have 'rust-script' installed. You can install it by running:
// cargo install rust-script

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use dirs;
use std::collections::HashSet;
use std::os::unix::fs::PermissionsExt;

struct FileManager;
impl FileManager {
    fn extract_mp3_files(dir: &str) -> Vec<String> {
        let mut mp3_files = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str().map(|s| s.to_lowercase())) == Some("mp3".to_string()) {
                        if let Some(path_str) = path.to_str() {
                            mp3_files.push(path_str.to_string());
                        }
                    } else if path.is_dir() {
                        let sub_dir_mp3_files = FileManager::extract_mp3_files(path.to_str().unwrap());
                        mp3_files.extend(sub_dir_mp3_files);
                    }
                }
            }
        }
        mp3_files
    }

    fn copy_transcription_files(transcription_files: Vec<String>, dest_dir: &str) {
        for transcription_file in transcription_files {
            let dest_path = Path::new(dest_dir).join(
                Path::new(&transcription_file)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    + ".md",
            );
            fs::copy(&transcription_file, &dest_path).expect("Failed to copy transcription file");
        }
    }
}

struct TranscriptionManager;
impl TranscriptionManager {
    fn get_transcribed_files(dest_dir: &str) -> HashSet<String> {
        let mut transcribed_files = HashSet::new();
        if let Ok(entries) = fs::read_dir(dest_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str().map(|s| s.to_lowercase())) == Some("md".to_string()) {
                        if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                            transcribed_files.insert(file_stem.to_string());
                        }
                    }
                }
            }
        }
        transcribed_files
    }

    fn filter_untranscribed_files(mp3_files: Vec<String>, transcribed_files: &HashSet<String>) -> Vec<String> {
        mp3_files.into_iter().filter(|file| {
            let file_stem = Path::new(file).file_stem().and_then(|s| s.to_str()).unwrap_or("");
            !transcribed_files.contains(file_stem)
        }).collect()
    }

    fn transcribe_mp3_files(mp3_files: Vec<String>) -> Vec<String> {
        let mut transcription_files = Vec::new();
        
        // Create the transcriptions directory if it doesn't exist
        let transcriptions_dir = dirs::document_dir()
            .expect("Failed to get document directory")
            .join("aTrain/transcriptions");
        fs::create_dir_all(&transcriptions_dir)
            .expect("Failed to create transcriptions directory");

        for mp3_file in mp3_files {
            println!("Transcribing file: {}", mp3_file); // Debug print
            let output = Command::new("aTrain_core")
                .arg("transcribe")
                .arg(&mp3_file)
                .output()
                .expect("Failed to execute aTrain_core");

            println!("Command executed"); // Debug print
            if !output.stdout.is_empty() {
                println!("Output: {}", String::from_utf8_lossy(&output.stdout)); // Debug print
            }

            if !output.stderr.is_empty() {
                println!("Error: {}", String::from_utf8_lossy(&output.stderr)); // Debug print
            }

            if output.status.success() {
                println!("Command succeeded"); // Debug print
                // Get the filename without path for the destination
                let file_name = Path::new(&mp3_file)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string() + ".md";

                if let Ok(entries) = fs::read_dir(&transcriptions_dir) {
                    if let Some(latest_folder) = entries
                        .filter_map(|entry| entry.ok())
                        .filter(|entry| entry.path().is_dir())
                        .max_by_key(|entry| entry.metadata().unwrap().modified().unwrap()) 
                    {
                        let latest_folder_path = latest_folder.path();
                        let metadata_file = latest_folder_path.join("metadata.txt");
                        
                        if metadata_file.exists() {
                            println!("Metadata file exists"); // Debug print
                            if let Ok(content) = fs::read_to_string(&metadata_file) {
                                for line in content.lines() {
                                    if line.starts_with("path_to_audio_file:") {
                                        let path = line.split(':').nth(1).unwrap().trim().to_string();
                                        if path == mp3_file {
                                            let transcription_file = latest_folder_path.join("transcription.txt");
                                            if transcription_file.exists() {
                                                println!("Transcription file exists"); // Debug print
                                                println!("Checking directory: {:?}", latest_folder_path); // Debug print
                                                for entry in fs::read_dir(&latest_folder_path).unwrap() {
                                                    let entry = entry.unwrap();
                                                    println!("Found entry: {:?}", entry.path()); // Debug print
                                                }
                                                let dest_file = transcriptions_dir.join(&file_name);
                                                println!("Copying transcription file from {:?} to {:?}", transcription_file, dest_file); // Debug print
                                                fs::copy(&transcription_file, &dest_file)
                                                    .expect("Failed to copy transcription file");
                                                transcription_files.push(dest_file.to_str().unwrap().to_string());
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                println!("Failed to transcribe file: {}", mp3_file); // Debug print
            }
        }
        transcription_files
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let verbose = args.contains(&"--verbose".to_string());
    if args.len() != 3 && !verbose {
        eprintln!("Usage: {} <mp3_dir> <dest_dir> [--verbose]", args[0]);
        std::process::exit(1);
    }

    let mp3_dir = &args[1];
    let dest_dir = &args[2];

    if !Path::new(mp3_dir).is_dir() {
        eprintln!("Error: {} is not a valid directory", mp3_dir);
        std::process::exit(1);
    }

    if !Path::new(dest_dir).is_dir() {
        eprintln!("Error: {} is not a valid directory", dest_dir);
        std::process::exit(1);
    }

    if verbose { println!("Extracting MP3 files from {}", mp3_dir); }
    let mp3_files = FileManager::extract_mp3_files(mp3_dir);
    if verbose { println!("Found {} MP3 files", mp3_files.len()); }

    if verbose { println!("Getting transcribed files from destination directory"); }
    let transcribed_files = TranscriptionManager::get_transcribed_files(dest_dir);
    if verbose { println!("Found {} transcribed files in destination directory", transcribed_files.len()); }

    if verbose { println!("Filtering untranscribed files"); }
    let untranscribed_files = TranscriptionManager::filter_untranscribed_files(mp3_files, &transcribed_files);
    if verbose { println!("Found {} untranscribed files", untranscribed_files.len()); }

    if verbose { println!("Transcribing MP3 files"); }
    let transcription_files = TranscriptionManager::transcribe_mp3_files(untranscribed_files);
    if verbose { println!("Transcribed {} files", transcription_files.len()); }

    if verbose { println!("Copying transcription files to {}", dest_dir); }
    FileManager::copy_transcription_files(transcription_files, dest_dir);
    if verbose { println!("Finished copying transcription files"); }
}

#[derive(Clone)]
struct TestContext {
    mp3_dir: Option<String>,
    dest_dir: Option<String>,
    mp3_files: Option<Vec<String>>,
    transcribed_files: Option<HashSet<String>>,
    untranscribed_files: Option<Vec<String>>,
    transcription_files: Option<Vec<String>>,
}

impl TestContext {
    fn new() -> Self {
        Self { mp3_dir: None, dest_dir: None, mp3_files: None, transcribed_files: None, untranscribed_files: None, transcription_files: None }
    }

    fn with_mp3_dir(mut self, mp3_dir: String) -> Self {
        self.mp3_dir = Some(mp3_dir);
        self
    }

    fn with_dest_dir(mut self, dest_dir: String) -> Self {
        self.dest_dir = Some(dest_dir);
        self
    }

    fn with_mp3_files(mut self, mp3_files: Vec<String>) -> Self {
        self.mp3_files = Some(mp3_files);
        self
    }

    fn with_transcribed_files(mut self, transcribed_files: HashSet<String>) -> Self {
        self.transcribed_files = Some(transcribed_files);
        self
    }

    fn with_untranscribed_files(mut self, untranscribed_files: Vec<String>) -> Self {
        self.untranscribed_files = Some(untranscribed_files);
        self
    }

    fn with_transcription_files(mut self, transcription_files: Vec<String>) -> Self {
        self.transcription_files = Some(transcription_files);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;
    use std::fs::File;
    use std::io::Write;

    fn given_mp3_files_in_directory() -> TestContext {
        let temp_dir = tempfile::tempdir().unwrap();
        let mp3_file = temp_dir.path().join("test.mp3");
        fs::write(&mp3_file, "dummy content").unwrap();
        let mp3_dir = temp_dir.into_path().to_str().unwrap().to_string();
        TestContext::new().with_mp3_dir(mp3_dir)
    }

    fn when_extracting_mp3_files(context: TestContext) -> TestContext {
        let mp3_files = FileManager::extract_mp3_files(context.mp3_dir.as_ref().unwrap());
        context.with_mp3_files(mp3_files)
    }

    fn then_mp3_files_are_extracted(context: TestContext) {
        assert_eq!(context.mp3_files.as_ref().unwrap().len(), 1);
        assert!(context.mp3_files.as_ref().unwrap()[0].ends_with("test.mp3"));
    }

    #[test]
    fn test_extract_mp3_files() {
        let context = given_mp3_files_in_directory();
        let context = when_extracting_mp3_files(context);
        then_mp3_files_are_extracted(context);
    }

    fn given_mock_atrain_core() -> tempfile::TempDir {
        let temp_dir = tempfile::tempdir().unwrap();
        let mock_atrain_core = temp_dir.path().join("aTrain_core");
        let mut file = File::create(&mock_atrain_core).unwrap();
        writeln!(file, "#!/bin/sh").unwrap();
        writeln!(file, "set -e").unwrap();
        writeln!(file, "TRANS_DIR=\"$HOME/Documents/aTrain/transcriptions/$(date +%Y-%m-%d_%H-%M-%S)\"").unwrap();
        writeln!(file, "mkdir -p \"$TRANS_DIR\"").unwrap();
        writeln!(file, "echo \"path_to_audio_file: $2\" > \"$TRANS_DIR/metadata.txt\"").unwrap();
        writeln!(file, "echo \"test transcription\" > \"$TRANS_DIR/transcription.txt\"").unwrap();
        fs::set_permissions(&mock_atrain_core, fs::Permissions::from_mode(0o755)).unwrap();

        // Create Documents directory structure
        let home_dir = dirs::home_dir().expect("Failed to get home directory");
        let doc_dir = home_dir.join("Documents/aTrain/transcriptions");
        fs::create_dir_all(&doc_dir).unwrap();
        
        temp_dir
    }

    fn when_transcribing_mp3_files(context: TestContext, mock_dir: &tempfile::TempDir) -> TestContext {
        let old_path = std::env::var("PATH").unwrap();
        let new_path = format!("{}:{}", mock_dir.path().to_str().unwrap(), old_path);
        std::env::set_var("PATH", new_path);
        let transcription_files = TranscriptionManager::transcribe_mp3_files(context.mp3_files.clone().unwrap());
        std::env::set_var("PATH", old_path);
        context.with_transcription_files(transcription_files)
    }

    fn then_transcription_files_are_generated(context: TestContext) {
        println!("Transcription files: {:?}", context.transcription_files); // Debug print
        assert_eq!(context.transcription_files.as_ref().unwrap().len(), 1);
        assert!(context.transcription_files.as_ref().unwrap()[0].ends_with(".md"));
    }

    #[test]
    fn test_transcribe_mp3_files() {
        let context = given_mp3_files_in_directory();
        let context = when_extracting_mp3_files(context);
        let mock_dir = given_mock_atrain_core();
        let context = when_transcribing_mp3_files(context, &mock_dir);
        then_transcription_files_are_generated(context);
        // Clean up
        mock_dir.close().unwrap();
    }

    fn given_transcription_files() -> (TestContext, tempfile::TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let transcription_file = temp_dir.path().join("test.mp3.md");
        let mut file = File::create(&transcription_file).unwrap();
        writeln!(file, "dummy transcription").unwrap();
        let transcription_files = vec![transcription_file.to_str().unwrap().to_string()];
        (TestContext::new().with_transcription_files(transcription_files), temp_dir)
    }

    fn when_copying_transcription_files(context: TestContext, dest_dir: &str) -> TestContext {
        FileManager::copy_transcription_files(context.transcription_files.clone().unwrap(), dest_dir);
        context.with_dest_dir(dest_dir.to_string())
    }

    fn then_transcription_files_are_copied(context: TestContext) {
        let dest_path = Path::new(context.dest_dir.as_ref().unwrap()).join("test.mp3.md");
        assert!(dest_path.exists());
    }

    #[test]
    fn test_copy_transcription_files() {
        let (context, temp_dir) = given_transcription_files();
        let dest_dir = tempfile::tempdir().unwrap();
        let context = when_copying_transcription_files(context, dest_dir.path().to_str().unwrap());
        then_transcription_files_are_copied(context);
        // Clean up
        temp_dir.close().unwrap();
        dest_dir.close().unwrap();
    }

    fn given_transcribed_files(dest_dir: &str) -> (TestContext, tempfile::TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let transcriptions_dir = Path::new(dest_dir);
        fs::create_dir_all(&transcriptions_dir).unwrap();
        let transcription_file = transcriptions_dir.join("transcribed_test.md");
        let mut file = File::create(&transcription_file).unwrap();
        writeln!(file, "dummy transcription").unwrap();
        let transcribed_files = TranscriptionManager::get_transcribed_files(dest_dir);
        println!("Transcribed files: {:?}", transcribed_files); // Debug print
        (TestContext::new().with_transcribed_files(transcribed_files), temp_dir)
    }

    fn when_filtering_untranscribed_files(context: TestContext) -> TestContext {
        let untranscribed_files = TranscriptionManager::filter_untranscribed_files(context.mp3_files.clone().unwrap(), context.transcribed_files.as_ref().unwrap());
        println!("Untranscribed files: {:?}", untranscribed_files); // Debug print
        context.with_untranscribed_files(untranscribed_files)
    }

    fn then_untranscribed_files_are_filtered(context: TestContext) {
        assert_eq!(context.untranscribed_files.as_ref().unwrap().len(), 1);
        assert!(context.untranscribed_files.as_ref().unwrap()[0].ends_with("test.mp3"));
    }

    #[test]
    fn test_filter_untranscribed_files() {
        let context = given_mp3_files_in_directory();
        let context = when_extracting_mp3_files(context);
        let dest_dir = tempfile::tempdir().unwrap();
        let (mut context_with_transcribed, transcribed_dir) = given_transcribed_files(dest_dir.path().to_str().unwrap());
        context_with_transcribed.mp3_files = context.mp3_files;
        context_with_transcribed.dest_dir = Some(dest_dir.path().to_str().unwrap().to_string());
        
        // Debug prints
        println!("MP3 files: {:?}", context_with_transcribed.mp3_files);
        println!("Transcribed files: {:?}", context_with_transcribed.transcribed_files);
        
        let context = when_filtering_untranscribed_files(context_with_transcribed);
        
        // Debug print
        println!("Untranscribed files: {:?}", context.untranscribed_files);
        
        then_untranscribed_files_are_filtered(context);
        
        // Clean up
        transcribed_dir.close().unwrap();
        dest_dir.close().unwrap();
    }
}
