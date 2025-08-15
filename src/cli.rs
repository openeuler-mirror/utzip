/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

use chrono::NaiveDate;
use clap::{ArgAction, Args, CommandFactory, Parser};
use std::path::PathBuf;

#[derive(Debug, Clone, Args, Default)]
#[group(id = "basic_mode_options", multiple = false)]
#[command(next_help_heading = "Basic modes")]
pub struct BasicModeOptions {
    /// Update: add new files/update existing files only if later date
    #[arg(short = 'u', long = "update", action = ArgAction::SetTrue)]
    pub update: bool,
    /// Freshen: update existing files only (no files added)
    #[arg(short = 'f', long = "freshen", action = ArgAction::SetTrue)]
    pub freshen: bool,

    /// Filesync: update if date or size changed, delete if no OS match
    #[arg(long = "FS", action = ArgAction::SetTrue)]
    pub filesync: bool,

    /// Delete files from archive (see below)
    #[arg(short = 'd', long = "delete", action = ArgAction::SetTrue)]
    pub delete: bool,

    /// Select files in archive to copy (use with --out)
    #[arg(short = 'U', long = "copy", requires = "out", action = ArgAction::SetTrue)]
    pub copy: bool,
}

#[derive(Debug, Clone, Args, Default)]
#[group(id = "basic_options")]
#[command(next_help_heading = "Basic options")]
pub struct BasicOptions {
    /// Recurse into directories (see Recursion below)
    #[arg(short = 'r', long = "recurse", action = ArgAction::SetTrue, conflicts_with="recurse_patterns")]
    pub recurse: bool,

    /// After archive created, delete original files (move into archive)
    #[arg(short = 'm', long = "move", action = ArgAction::SetTrue)]
    pub move_files: bool,
    /// Junk directory names (store just file names)
    #[arg(short = 'j', long = "junk-paths", action = ArgAction::SetTrue)]
    pub junk_paths: bool,
    /// Quiet operation
    #[arg(short = 'q', long = "quiet", action = ArgAction::SetTrue, conflicts_with="verbose")]
    pub quiet: bool,
    /// Verbose operation (just \"utzip -v\" shows version information)
    #[arg(short = 'v', long = "verbose", action = ArgAction::SetTrue, conflicts_with="quiet")]
    pub verbose: bool,

    /// Prompt for one-line comment for each entry
    #[arg(short = 'c', long = "comments", action = ArgAction::SetTrue)]
    pub add_comments: bool,

    /// Prompt for comment for archive (end with just \".\" line or EOF)
    #[arg(short = 'z', long = "archive-comment", action = ArgAction::SetTrue)]
    pub add_archive_comment: bool,

    /// Read names to zip from stdin (one path per line)
    #[arg(short = '@', action = ArgAction::SetTrue)]
    pub read_names_from_stdin: bool,

    /// Make zipfile as old as latest entry
    #[arg(short = 'o', long = "latest-time", action = ArgAction::SetTrue)]
    pub latest_time: bool,
}

#[derive(Debug, Clone, Args, Default)]
#[group(id = "display")]
#[command(next_help_heading = "Display Options")]
pub struct DisplayOptions {
    /// Display running count of bytes processed and bytes to go
    #[arg(long = "db", action = ArgAction::SetTrue)]
    pub display_bytes: bool,
    /// Display running count of entries done and entries to go
    #[arg(long = "dc", action = ArgAction::SetTrue)]
    pub display_count: bool,
    /// Display dots every 10 MB (or dot size) while processing files
    #[arg(long = "dd", action = ArgAction::SetTrue, conflicts_with="display_dots_global")]
    pub display_dots: bool,
    /// Display dots globally for archive instead of for each file
    #[arg(long = "dg", action = ArgAction::SetTrue, conflicts_with="display_dots")]
    pub display_dots_global: bool,
    /// Each dot is siz processed where siz is nm as splits (0 no dots)
    #[arg(long = "ds", value_name = "size", value_parser = |s: &str| parse_split_size_arg(s, 32 * 1024))]
    pub display_dots_size: Option<u64>,
    /// Display original uncompressed size for each entry as added
    #[arg(long = "du", action = ArgAction::SetTrue)]
    pub display_uncompressed: bool,
    /// Display volume (disk) number in format in_disk>out_disk
    #[arg(long = "dv", action = ArgAction::SetTrue)]
    pub display_volume: bool,
}

#[derive(Debug, Clone, Args, Default)]
#[group(id = "logging")]
#[command(next_help_heading = "Logging")]
pub struct LoggingOptions {
    /// Open file at path as logfile (overwrite existing file)
    #[arg(long = "lf", value_name = "LOGFILE")]
    pub logfile: Option<PathBuf>,

    /// Append to existing logfile
    #[arg(long = "la", action = ArgAction::SetTrue, requires = "logfile")]
    pub logfile_append: bool,

    /// Include info messages (default just warnings and errors)
    #[arg(long = "li", action = ArgAction::SetTrue, requires = "logfile")]
    pub logfile_info: bool,
}

#[derive(Debug, Clone, Args, Default)]
#[group(id = "split_options")]
#[command(next_help_heading = "Splits (archives created as a set of split files)")]
pub struct SplitOptions {
    /// Create split archive with splits of size ssize, where ssize nm n number and m multiplier (kmgt, default m), 100k -> 100 kB
    #[arg(short = 's', long = "ssize", value_name = "ssize", value_parser = |s: &str| parse_split_size_arg(s, 64 * 1024))]
    pub split_size: Option<u64>,

    /// Use after each split closed to allow changing disks
    #[arg(long = "sp", action = ArgAction::SetTrue, requires="split_size")]
    pub split_pause: bool,

    /// Ring bell when pause
    #[arg(long = "sb", action = ArgAction::SetTrue,requires="split_size")]
    pub split_beep: bool,

    /// Be verbose about creating splits
    #[arg(long = "sv", action = ArgAction::SetTrue,requires="split_size")]
    pub split_verbose: bool,
}

#[derive(Debug, Clone, Args, Default)]
#[group(id = "show_files_options")]
#[command(next_help_heading = "Show files")]
pub struct ShowOptions {
    /// List archive contents
    #[arg(long = "sf", action = ArgAction::SetTrue, requires = "zipfile")]
    pub list: bool,
    /// As --sf but show escaped UTF-8 Unicode names also if exist
    #[arg(long = "su", action = ArgAction::SetTrue, requires = "zipfile")]
    pub show_unicode: bool,
    /// As --sf but show escaped UTF-8 Unicode names instead
    #[arg(long = "sU", action = ArgAction::SetTrue, requires = "zipfile")]
    pub show_unicode_only: bool,
}

#[derive(Debug, Clone, Args, Default)]
#[group(id = "compression_options")]
#[command(next_help_heading = "Compression")]
pub struct CompressionOptions {
    /// Store files with no compression
    #[arg(short = '0', action = ArgAction::SetTrue)]
    pub store_only: bool,

    /// Compress faster
    #[arg(short = '1', action = ArgAction::SetTrue)]
    pub compress_faster: bool,

    /// Compress better
    #[arg(short = '9', action = ArgAction::SetTrue)]
    pub compress_better: bool,

    /// Compress level (0-9)
    #[arg(short = '2', action = ArgAction::SetTrue, hide=true)]
    pub level_2: bool,
    #[arg(short = '3', action = ArgAction::SetTrue, hide=true)]
    pub level_3: bool,
    #[arg(short = '4', action = ArgAction::SetTrue, hide=true)]
    pub level_4: bool,
    #[arg(short = '5', action = ArgAction::SetTrue, hide=true)]
    pub level_5: bool,
    #[arg(short = '6', action = ArgAction::SetTrue, hide=true)]
    pub level_6: bool,
    #[arg(short = '7', action = ArgAction::SetTrue, hide=true)]
    pub level_7: bool,
    #[arg(short = '8', action = ArgAction::SetTrue, hide=true)]
    pub level_8: bool,

    /// Set compression method to cm
    #[arg(short = 'Z', long = "compression-method", value_name = "CM",
        value_parser = clap::builder::PossibleValuesParser::new(["store", "deflate", "bzip2"]))]
    pub compression_method: Option<String>,
}

#[derive(Debug, Clone, Args, Default)]
#[group(id = "encryption_options")]
#[command(next_help_heading = "Encryption")]
pub struct EncryptionOptions {
    /// Use standard (weak) PKZip 2.0 encryption, prompt for password
    #[arg(short = 'e', long = "encrypt", action = ArgAction::SetTrue)]
    pub encrypt: bool,
    /// Use standard encryption, password is pswd
    #[arg(short = 'P', long = "password")]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Args, Default)]
#[group(id = "filter_options")]
#[command(next_help_heading = "Include and Exclude")]
pub struct FilterOptions {
    /// Include files that match a pattern
    #[arg(short = 'i', long = "include", conflicts_with = "delete")]
    pub include: Vec<String>,

    /// Exclude files that match a pattern
    #[arg(short = 'x', long = "exclude")]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Args, Default)]
#[group(id = "data_filter_options")]
#[command(next_help_heading = "Date filtering")]
pub struct DataFilterOptions {
    /// Exclude before (include files modified on this date and later) (mmddyyyy or yyyy-mm-dd)
    #[arg(short='t', value_name = "data", value_parser = parse_date)]
    pub after_date: Option<NaiveDate>,
    /// Include before (include files modified before date) (mmddyyyy or yyyy-mm-dd)
    #[arg(long = "tt", value_name = "data", value_parser = parse_date)]
    pub before_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Args, Default)]
#[group(id = "test_options")]
#[command(next_help_heading = "Testing archives")]
pub struct TestOptions {
    /// Test completed temp archive with unzip before updating archive
    #[arg(short = 'T', action = ArgAction::SetTrue)]
    pub test: bool,

    /// Use command cmd instead of 'unzip -tqq' to test archive
    #[arg(long = "TT", value_name = "CMD")]
    pub test_cmd: Option<String>,
}

#[derive(Debug, Clone, Args, Default)]
#[group(id = "extractor_options")]
#[command(next_help_heading = "Self extractor")]
pub struct ExtractorOptions {
    /// Adjust self-extracting exe
    #[arg(short = 'A', long = "adjust-sfx", action = ArgAction::SetTrue)]
    pub adjust_sfx: bool,

    /// Junk zipfile prefix (unzipsfx)
    #[arg(short = 'J', long = "junk-sfx", action = ArgAction::SetTrue)]
    pub junk_sfx: bool,
}

#[derive(Debug, Clone, Args, Default)]
#[group(id = "fix_options")]
#[command(next_help_heading = "Fixing archives")]
pub struct FixOptions {
    /// Attempt to fix a mostly intact archive (try this first)
    #[arg(short = 'F', action = ArgAction::SetTrue, requires="out")]
    pub fix_normal: bool,

    /// Try to salvage what can (may get more but less reliable)
    #[arg(long = "FF",action = ArgAction::SetTrue, requires="out")]
    pub fix_full: bool,
}

#[derive(Debug, Clone, Args, Default)]
#[group(id = "translation_options")]
#[command(next_help_heading = "End Of Line Translation (text files only)")]
pub struct TranslationOptions {
    /// Change CR or LF (depending on OS) line end to CR LF (Unix->Win)
    #[arg(short = 'l', action = ArgAction::SetTrue)]
    pub convert_lf_to_crlf: bool,

    /// Change CR LF to CR or LF (depending on OS) line end (Win->Unix)
    #[arg(long = "ll",action = ArgAction::SetTrue)]
    pub convert_crlf_to_lf: bool,
}

#[derive(Debug, Clone, Args, Default)]
#[group(id = "other_options")]
#[command(next_help_heading = "More options")]
pub struct OtherOptions {
    /// Display extended help information
    #[arg(long = "h2", action = ArgAction::SetTrue)]
    pub extended_help: bool,

    /// Output to archive
    #[arg(short = 'O', long = "out", value_name = "OUTPUT")]
    pub out: Option<PathBuf>,

    /// Recurse current dir and match patterns
    #[arg(short = 'R', long = "recurse-patterns", action = ArgAction::SetTrue, conflicts_with = "recurse")]
    pub recurse_patterns: bool,

    /// Do not add directory entries
    #[arg(short = 'D', long = "no-dir-entries", action = ArgAction::SetTrue)]
    pub no_dir_entries: bool,

    /// Exclude extra file attributes
    #[arg(short = 'X', long = "no-extra", action = ArgAction::SetTrue)]
    pub no_extra: bool,

    /// Store symbolic links as links
    #[arg(short = 'y', long = "symlinks", action = ArgAction::SetTrue)]
    pub store_symlinks: bool,

    /// Don't compress these suffixes
    #[arg(short = 'n', long = "suffixes")]
    pub dont_compress_suffixes: Option<String>,

    /// Use temporary file path
    #[arg(short = 'b', long = "temp-path", value_name = "PATH")]
    pub temp_path: Option<PathBuf>,

    /// Only include files that have changed or are new as compared to the input archive
    #[arg(long = "dif", action = ArgAction::SetTrue, requires_all = ["zipfile", "files","out"])]
    pub dif: bool,

    /// Unicode paths allow better conversion of entry names between different character sets
    #[arg(long = "UN", value_name = "ENCODING", value_parser = clap::builder::PossibleValuesParser::new(["Quit", "Warn", "Ignore","No","Escape","UTF8"]))]
    pub encode: Option<String>,

    /// Show command line arguments as processed and exit
    #[arg(long = "sc", action = ArgAction::SetTrue)]
    pub show_command: bool,

    /// Show debugging as Zip does each step
    #[arg(long = "sd", action = ArgAction::SetTrue)]
    pub show_debug: bool,
    /// Show all available options on this system
    #[arg(long = "so", action = ArgAction::SetTrue)]
    pub show_options: bool,

    /// No wildcards (wildcards are like any other character)
    #[arg(long = "nw", action = ArgAction::SetTrue)]
    pub no_wildcards: bool,
    /// Wildcards don't span directory boundaries in paths
    #[arg(long = "ws", action = ArgAction::SetTrue)]
    pub no_wildcards_boundary: bool,

    /// Show software license
    #[arg(short = 'L', long = "license", action = ArgAction::SetTrue)]
    pub license: bool,
}

// 解析日期字符串为 NaiveDate 类型, 支持 MMDDYYYY 和 YYYY-MM-DD 格式
fn parse_date(date_str: &str) -> Result<NaiveDate, String> {
    if date_str.len() == 8 {
        // MMDDYYYY 格式
        NaiveDate::parse_from_str(date_str, "%m%d%Y")
            .map_err(|e| format!("Invalid MMDDYYYY date format: {}", e))
    } else if date_str.len() == 10 && date_str.contains('-') {
        // YYYY-MM-DD 格式
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|e| format!("Invalid YYYY-MM-DD date format: {}", e))
    } else {
        Err("Date must be in MMDDYYYY or YYYY-MM-DD format".to_string())
    }
}

// 解析分割大小参数, 支持 100m, 1g 等格式
fn parse_split_size_arg(s: &str, min_size: u64) -> Result<u64, String> {
    let re = regex::Regex::new(r"^(?i)(\d+)([kmgt]?)$").unwrap();
    let caps = re
        .captures(s)
        .ok_or_else(|| format!("Invalid size format '{}', must be like 100m, 100k etc.", s))?;

    let num = caps[1].parse::<u64>().map_err(|e| e.to_string())?;
    let unit = caps.get(2).map(|m| m.as_str().to_lowercase());

    let size = match unit.as_deref() {
        Some("k") => num * 1024,
        Some("m") | None => num * 1024 * 1024,
        Some("g") => num * 1024 * 1024 * 1024,
        Some("t") => num * 1024 * 1024 * 1024 * 1024,
        _ => unreachable!(),
    };
    // 使用传入的min_size参数进行校验
    if size < min_size {
        return Err(format!(
            "Split size must be at least {}, got {} bytes",
            min_size, size
        ));
    }

    Ok(size)
}
