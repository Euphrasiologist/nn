use clap::{Parser, Subcommand};
use dirs::home_dir;
use jiff::Zoned;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf, process::Command};

/// Load config from ~/.notes_cli/config.toml or create a default one
fn load_or_init_config() -> Config {
    let config_dir = home_dir().unwrap().join(".notes_cli");
    let config_file = config_dir.join("config.toml");
    let default = Config {
        notes_dir: config_dir.join("notes"),
        editor: "nano".into(),
    };

    if !config_file.exists() {
        fs::create_dir_all(&config_dir).unwrap();
        let toml_str = toml::to_string(&default).unwrap();
        fs::write(&config_file, toml_str).unwrap();
        fs::create_dir_all(&default.notes_dir).unwrap();
        default
    } else {
        let toml_str = fs::read_to_string(&config_file).unwrap();
        // check against toml keys
        match toml::from_str(&toml_str) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Error loading nn config: {}", e);
                // ask to continue [y/n]
                let mut input = String::new();
                println!("Continue with default config? [y/n]");
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read line");
                if input.trim().to_lowercase() != "y" {
                    // just exit here
                    std::process::exit(1);
                } else {
                    default
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    notes_dir: PathBuf,
    editor: String,
}

#[derive(Parser)]
#[command(name = "nn", version, about = "A normal notes tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Delete a note
    Delete { date: String },
    /// List all notes
    List,
    /// Search all notes for a string
    Search { query: String },
    /// Show all tags used in notes
    Tags,
}

/// Get the path to the note file for a given date
fn get_note_path(config: &Config, date: &str) -> PathBuf {
    config.notes_dir.join(format!("{}.md", date))
}

/// Create a note file with a basic heading if it does not already exist
fn create_note_if_missing(path: &PathBuf) -> io::Result<()> {
    if !path.exists() {
        fs::write(
            path,
            format!("# {}\n\n", path.file_name().unwrap().to_string_lossy()),
        )?;
    }
    Ok(())
}

/// Open the given file in the configured text editor
fn open_editor(path: &PathBuf, config: &Config) {
    let editor = &config.editor;
    Command::new(editor).arg(path).status().unwrap();
}

/// Print a list of all notes in the configured notes directory
fn list_notes(config: &Config) {
    let entries = fs::read_dir(&config.notes_dir).unwrap();
    for entry in entries.flatten() {
        if let Some(name) = entry.path().to_str() {
            println!("{}", name);
        }
    }
}

/// Delete the note file corresponding to the given date, if it exists
fn delete_note(config: &Config, date: &str) {
    let path = get_note_path(config, date);
    if path.exists() {
        fs::remove_file(path).unwrap();
        eprintln!("Deleted note for {}", date);
    } else {
        eprintln!("No note found for {}", date);
    }
}

/// Search all notes for a query string and print matching notes with content
fn search_notes(config: &Config, query: &str) {
    let entries = fs::read_dir(&config.notes_dir).unwrap();
    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(contents) = fs::read_to_string(&path) {
            if contents.contains(query) {
                eprintln!("{}:\n{}", path.display(), contents);
            }
        }
    }
}

/// Extract and print all unique tags (e.g. #rust, #todo) used in notes
fn extract_tags(config: &Config) {
    let tag_re = Regex::new(r"#\w+").unwrap();
    let mut tags = std::collections::HashSet::new();
    let entries = fs::read_dir(&config.notes_dir).unwrap();

    for entry in entries.flatten() {
        if let Ok(contents) = fs::read_to_string(entry.path()) {
            for tag in tag_re.find_iter(&contents) {
                tags.insert(tag.as_str().to_string());
            }
        }
    }

    for tag in tags {
        println!("{}", tag);
    }
}

fn main() {
    let cli = Cli::parse();
    let config = load_or_init_config();

    match cli.command {
        Some(Commands::Delete { date }) => delete_note(&config, &date),
        Some(Commands::List) => list_notes(&config),
        Some(Commands::Search { query }) => search_notes(&config, &query),
        Some(Commands::Tags) => extract_tags(&config),
        None => {
            // Default: edit today's note
            let date = Zoned::now().strftime("%Y-%m-%d").to_string();
            let path = get_note_path(&config, &date);
            create_note_if_missing(&path).unwrap();
            open_editor(&path, &config);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_get_note_path() {
        let config = Config {
            notes_dir: PathBuf::from("/tmp/my-notes"),
            editor: "nano".to_string(),
        };
        let date = "2025-04-09";
        let path = get_note_path(&config, date);
        assert_eq!(path, PathBuf::from("/tmp/my-notes/2025-04-09.md"));
    }

    #[test]
    fn test_create_note_if_missing_creates_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test-note.md");

        create_note_if_missing(&file_path).unwrap();
        assert!(file_path.exists());

        let content = fs::read_to_string(file_path).unwrap();
        assert!(content.contains("# test-note.md"));
    }

    #[test]
    fn test_create_note_if_missing_does_not_overwrite() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test-note.md");
        fs::write(&file_path, "original content").unwrap();

        create_note_if_missing(&file_path).unwrap();
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "original content"); // File should remain unchanged
    }

    #[test]
    fn test_extract_tags_finds_all_tags() {
        let dir = tempdir().unwrap();
        let config = Config {
            notes_dir: dir.path().to_path_buf(),
            editor: "nano".into(),
        };

        let mut file1 = File::create(dir.path().join("note1.md")).unwrap();
        writeln!(file1, "Today I worked on #rust and #cli").unwrap();

        let mut file2 = File::create(dir.path().join("note2.md")).unwrap();
        writeln!(file2, "This is #rust again and also #dev").unwrap();

        let mut captured_tags = std::collections::HashSet::new();
        let tag_re = Regex::new(r"#\w+").unwrap();
        let entries = fs::read_dir(&config.notes_dir).unwrap();
        for entry in entries.flatten() {
            if let Ok(contents) = fs::read_to_string(entry.path()) {
                for tag in tag_re.find_iter(&contents) {
                    captured_tags.insert(tag.as_str().to_string());
                }
            }
        }

        let expected_tags: std::collections::HashSet<_> = ["#rust", "#cli", "#dev"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(captured_tags, expected_tags);
    }
}
