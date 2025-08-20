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

#[derive(Debug, Parser, Clone, Default)]
// #[command(disable_help_flag = true)] // 禁用默认的-h/--help
#[command(name = "utzip")]
#[command(about = "A ZIP file archiver written in Rust")]

pub struct ZipArgs {
    /// Input zip file
    #[arg(value_name = "ZIPFILE")]
    pub zipfile: Option<PathBuf>,

    /// Files to process
    #[arg(value_name = "FILES")]
    pub files: Vec<PathBuf>,

    /// Basic Mode Options
    #[command(flatten)]
    pub basic_mode_options: BasicModeOptions,

    /// basic options
    #[command(flatten)]
    pub basic_options: BasicOptions,

    /// compression options
    #[command(flatten)]
    pub compression: CompressionOptions,

    /// encryption options
    #[command(flatten)]
    pub encryption: EncryptionOptions,

    /// filter options
    #[command(flatten)]
    pub filter: FilterOptions,

    /// translation options
    #[command(flatten)]
    pub translation: TranslationOptions,

    /// data filter options
    #[command(flatten)]
    pub data_filter: DataFilterOptions,

    /// display options
    #[command(flatten)]
    pub display: DisplayOptions,
    /// logging options
    #[command(flatten)]
    pub logging: LoggingOptions,
    /// test options
    #[command(flatten)]
    pub test: TestOptions,

    /// split options
    #[command(flatten)]
    pub split: SplitOptions,
    /// show files options
    #[command(flatten)]
    pub show: ShowOptions,

    /// extractor options
    #[command(flatten)]
    pub extractor: ExtractorOptions,
    /// fix options
    #[command(flatten)]
    pub fix: FixOptions,
    /// other options
    #[command(flatten)]
    pub other: OtherOptions,

    /// Command to execute (internal use)
    #[arg(skip)]
    pub command: Command,
}

#[derive(Debug, Parser, Clone, Default)]
#[command(name = "utzipnote")]
#[command(
    about = r#"Copyright (c) 1990-2008 Info-ZIP - Type 'utzipnote "-L"' for software license."#
)]
pub struct ZipNoteArgs {
    /// Input zip file
    #[arg(value_name = "ZIPFILE")]
    pub zipfile: Option<PathBuf>,

    /// Write the zipfile comments from stdin
    #[arg(short = 'w', long = "write", action = ArgAction::SetTrue)]
    pub write: bool,

    /// Use "path" for the temporary zip file
    #[arg(short = 'b', long = "temp-path", value_name = "PATH")]
    pub temp_path: Option<PathBuf>,

    /// Quiet operation, suppress some informational messages
    #[arg(short = 'q', long = "quiet", action = ArgAction::SetTrue)]
    pub quiet: bool,

    /// Show version info
    #[arg(short = 'v', long = "version", action = ArgAction::SetTrue)]
    pub version: bool,

    /// Show software license
    #[arg(short = 'L', long = "license", action = ArgAction::SetTrue)]
    pub license: bool,
}

#[derive(Debug, Parser, Clone, Default)]
#[command(name = "utzipcloak")]
#[command(
    about = r#"Copyright (c) 1990-2008 Info-ZIP - Type 'utzipcloak "-L"' for software license."#
)]
pub struct ZipCloakArgs {
    /// Input zip file
    #[arg(value_name = "ZIPFILE")]
    pub zipfile: Option<PathBuf>,

    /// Decrypt encrypted entries (copy if given wrong password)
    #[arg(short = 'd', long = "decrypt", action = ArgAction::SetTrue)]
    pub decrypt: bool,

    /// Use "path" for the temporary zip file
    #[arg(short = 'b', long = "temp-path", value_name = "PATH")]
    pub temp_path: Option<PathBuf>,

    /// Write output to new zip file
    #[arg(short = 'O', long = "output-file", value_name = "OUTPUT")]
    pub out: Option<PathBuf>,

    /// Quiet operation, suppress some informational messages
    #[arg(short = 'q', long = "quiet", action = ArgAction::SetTrue)]
    pub quiet: bool,

    /// Show version info
    #[arg(short = 'v', long = "version", action = ArgAction::SetTrue)]
    pub version: bool,

    /// Show software license
    #[arg(short = 'L', long = "license", action = ArgAction::SetTrue)]
    pub license: bool,
}

#[derive(Debug, Parser, Clone, Default)]
#[command(name = "utzipsplit")]
#[command(
    about = r#"Copyright (c) 1990-2008 Info-ZIP - Type 'utzipsplit "-L"' for software license."#
)]
pub struct ZipSplitArgs {
    /// Input zip file
    #[arg(value_name = "ZIPFILE")]
    pub zipfile: Option<PathBuf>,

    /// Report how many files it will take, but don't make them
    #[arg(short = 't', long = "test", action = ArgAction::SetTrue)]
    pub test: bool,
    /// Make index (zipsplit.idx) and count its size against first zip file
    #[arg(short = 'i', long = "index", action = ArgAction::SetTrue)]
    pub index: bool,
    /// Make zip files no larger than "size" (default = 36000)
    #[arg(
        short = 'n',
        long = "max_size",
        value_name = "SIZE",
        default_value = "36000"
    )]
    pub max_size: u32,
    /// Leave room for "room" bytes on the first disk (default = 0)
    #[arg(short = 'r', long = "room", value_name = "ROOM", default_value = "0")]
    pub room: u32,

    /// Use "path" for the temporary zip file
    #[arg(short = 'b', long = "temp-path", value_name = "PATH")]
    pub temp_path: Option<PathBuf>,

    /// Quiet operation, suppress some informational messages
    #[arg(short = 'q', long = "quiet", action = ArgAction::SetTrue)]
    pub quiet: bool,
    /// Pause between output zip files
    #[arg(short = 'p', long = "pause", action = ArgAction::SetTrue)]
    pub pause: bool,
    /// Do a sequential split even if it takes more zip files
    #[arg(short = 's', long = "sequential", action = ArgAction::SetTrue)]
    pub sequential: bool,

    /// Show version info
    #[arg(short = 'v', long = "version", action = ArgAction::SetTrue)]
    pub version: bool,

    /// Show software license
    #[arg(short = 'L', long = "license", action = ArgAction::SetTrue)]
    pub license: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Command {
    #[default]
    Add,
    Delete,
    Update,
    Copy,
    List,
    Test,
    Fix,
    Adjust,
}

pub fn parse_args() -> ZipArgs {
    let mut args = ZipArgs::parse();

    // 如果设置了show_options参数，显示帮助信息并退出
    if args.other.show_options {
        ZipArgs::command().print_help().unwrap();
        std::process::exit(0);
    }

    // 如果设置了--h2参数，显示帮助信息并退出（等价于-h）
    if args.other.extended_help {
        ZipArgs::command().print_help().unwrap();
        std::process::exit(0);
    }

    // 确定要执行的命令
    if args.basic_mode_options.delete {
        args.command = Command::Delete;
    } else if args.show.list || args.show.show_unicode || args.show.show_unicode_only {
        args.command = Command::List;
    } else if args.basic_mode_options.update
        || args.basic_mode_options.filesync
        || args.basic_mode_options.freshen
    {
        //刷新操作是更新的特例 - 只更新已存在于归档中的文件
        args.command = Command::Update;
    } else if args.basic_mode_options.copy {
        args.command = Command::Copy;
    } else if args.test.test || args.test.test_cmd.is_some() {
        // -T 参数的特殊逻辑：
        // 1. 如果有文件参数，则先执行压缩操作，然后执行test
        // 2. 如果没有文件参数，则只执行test（如果zip文件不存在则报错）
        if !args.files.is_empty() {
            // 有文件参数，执行压缩操作 (Add命令会处理-T参数)
            args.command = Command::Add;
        } else {
            // 没有文件参数，只执行test
            args.command = Command::Test;
        }
    } else if args.fix.fix_normal || args.fix.fix_full {
        args.command = Command::Fix;
    } else if args.extractor.adjust_sfx || args.extractor.junk_sfx {
        args.command = Command::Adjust;
    } else {
        args.command = Command::Add; // 默认命令
    }

    // 动态验证 -s 参数的要求
    if args.split.split_size.is_some() {
        // 检查ZIP文件是否存在
        if let Some(zipfile) = &args.zipfile {
            if zipfile.exists() {
                // ZIP文件存在，必须提供 --out 参数
                if args.other.out.is_none() {
                    eprintln!("zip error: when zip file exists, split option (-s) requires output file (--out)");
                    std::process::exit(1);
                }
            }
            // ZIP文件不存在时，不要求 --out 参数，可以直接使用原文件名
        } else {
            eprintln!("zip error: split option (-s) requires zip file name");
            std::process::exit(1);
        }
    }

    args
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

// 解析zipnote命令行参数
#[allow(dead_code)]
pub fn parse_args_note() -> ZipNoteArgs {
    ZipNoteArgs::parse()
}

// 解析zipcloak命令行参数
#[allow(dead_code)]
pub fn parse_args_cloak() -> ZipCloakArgs {
    ZipCloakArgs::parse()
}

// 解析zipsplit命令行参数
#[allow(dead_code)]
pub fn parse_args_split() -> ZipSplitArgs {
    ZipSplitArgs::parse()
}

#[allow(dead_code)]
pub fn show_help() {
    println!("utzip [-options] [-b path] [-t mmddyyyy] [-n suffixes] [zipfile list] [-xi list]");
    println!("  The default action is to add or replace zipfile entries from list, which");
    println!("  can include the special name - to compress standard input.");
    println!("  If zipfile and list are omitted, zip compresses stdin to stdout.");
    println!("  -f   freshen: only changed files  -u   update: only changed or new files");
    println!("  -d   delete entries in zipfile    -m   move into zipfile (delete OS files)");
    println!("  -r   recurse into directories     -j   junk (don't record) directory names");
    println!("  -0   store only                   -l   convert LF to CR LF (-ll CR LF to LF)");
    println!("  -1   compress faster              -9   compress better");
    println!("  -q   quiet operation              -v   verbose operation/print version info");
    println!("  -c   add one-line comments        -z   add zipfile comment");
    println!("  -@   read names from stdin        -o   make zipfile as old as latest entry");
    println!("  -x   exclude the following names  -i   include only the following names");
    println!("  -F   fix zipfile (-FF try harder) -D   do not add directory entries");
    println!("  -A   adjust self-extracting exe   -J   junk zipfile prefix (unzipsfx)");
    println!("  -T   test zipfile integrity       -X   eXclude eXtra file attributes");
    println!("  -y   store symbolic links as the link instead of the referenced file");
    println!("  -e   encrypt                      -n   don't compress these suffixes");
}

#[allow(dead_code)]
pub fn show_extended_help() {
    show_help();
    println!(
        r#"
Extended Help for Zip

See the Zip Manual for more detailed help


Zip stores files in zip archives.  The default action is to add or replace
zipfile entries.

Basic command line:
  zip options archive_name file file ...

Some examples:
  Add file.txt to z.zip (create z if needed):      zip z file.txt
  Zip all files in current dir:                    zip z *
  Zip files in current dir and subdirs also:       zip -r z .

Basic modes:
 External modes (selects files from file system):
        add      - add new files/update existing files in archive (default)
  -u    update   - add new files/update existing files only if later date
  -f    freshen  - update existing files only (no files added)
  -FS   filesync - update if date or size changed, delete if no OS match
 Internal modes (selects entries in archive):
  -d    delete   - delete files from archive (see below)
  -U    copy     - select files in archive to copy (use with --out)

Basic options:
  -r        recurse into directories (see Recursion below)
  -m        after archive created, delete original files (move into archive)
  -j        junk directory names (store just file names)
  -q        quiet operation
  -v        verbose operation (just "zip -v" shows version information)
  -c        prompt for one-line comment for each entry
  -z        prompt for comment for archive (end with just "." line or EOF)
  -@        read names to zip from stdin (one path per line)
  -o        make zipfile as old as latest entry


Syntax:
  The full command line syntax is:

    zip [-shortopts ...] [--longopt ...] [zipfile [path path ...]] [-xi list]

  Any number of short option and long option arguments are allowed
  (within limits) as well as any number of path arguments for files
  to zip up.  If zipfile exists, the archive is read in.  If zipfile
  is "-", stream to stdout.  If any path is "-", zip stdin.

Options and Values:
  For short options that take values, use -ovalue or -o value or -o=value
  For long option values, use either --longoption=value or --longoption value
  For example:
    zip -ds 10 --temp-dir=path zipfile path1 path2 --exclude pattern pattern
  Avoid -ovalue (no space between) to avoid confusion
  In particular, be aware of 2-character options.  For example:
    -d -s is (delete, split size) while -ds is (dot size)
  Usually better to break short options across multiple arguments by function
    zip -r -dbdcds 10m -lilalf logfile archive input_directory -ll

  All args after just "--" arg are read verbatim as paths and not options.
    zip zipfile path path ... -- verbatimpath verbatimpath ...
  Use -nw to also disable wildcards, so paths are read literally:
    zip zipfile -nw -- "-leadingdashpath" "a[path].c" "path*withwildcard"
  You may still have to escape or quote arguments to avoid shell expansion

Wildcards:
  Internally zip supports the following wildcards:
    ?       (or %% or #, depending on OS) matches any single character
    *       matches any number of characters, including zero
    [list]  matches char in list (regex), can do range [ac-f], all but [!bf]
  If port supports [], must escape [ as [[] or use -nw to turn off wildcards
  For shells that expand wildcards, escape (\* or "*") so zip can recurse
    zip zipfile -r . -i "*.h"

  Normally * crosses dir bounds in path, e.g. 'a*b' can match 'ac/db'.  If
   -ws option used, * does not cross dir bounds but ** does

  For DOS and Windows, [list] is now disabled unless the new option
  -RE       enable [list] (regular expression) matching
  is used to avoid problems with file paths containing "[" and "]":
    zip files_ending_with_number -RE foo[0-9].c

Include and Exclude:
  -i pattern pattern ...   include files that match a pattern
  -x pattern pattern ...   exclude files that match a pattern
  Patterns are paths with optional wildcards and match paths as stored in
  archive.  Exclude and include lists end at next option, @, or end of line.
    zip -x pattern pattern @ zipfile path path ...

Case matching:
  On most OS the case of patterns must match the case in the archive, unless
  the -ic option is used.
  -ic       ignore case of archive entries
  This option not available on case-sensitive file systems.  On others, case
  ignored when matching files on file system but matching against archive
  entries remains case sensitive for modes -f (freshen), -U (archive copy),
  and -d (delete) because archive paths are always case sensitive.  With
  -ic, all matching ignores case, but it's then possible multiple archive
  entries that differ only in case will match.

End Of Line Translation (text files only):
  -l        change CR or LF (depending on OS) line end to CR LF (Unix->Win)
  -ll       change CR LF to CR or LF (depending on OS) line end (Win->Unix)
  If first buffer read from file contains binary the translation is skipped

Recursion:
  -r        recurse paths, include files in subdirs:  zip -r a path path ...
  -R        recurse current dir and match patterns:   zip -R a ptn ptn ...
  Use -i and -x with either to include or exclude paths
  Path root in archive starts at current dir, so if /a/b/c/file and
   current dir is /a/b, 'zip -r archive .' puts c/file in archive

Date filtering:
  -t date   exclude before (include files modified on this date and later)
  -tt date  include before (include files modified before date)
  Can use both at same time to set a date range
  Dates are mmddyyyy or yyyy-mm-dd

Deletion, File Sync:
  -d        delete files
  Delete archive entries matching internal archive paths in list
    zip archive -d pattern pattern ...
  Can use -t and -tt to select files in archive, but NOT -x or -i, so
    zip archive -d "*" -t 2005-12-27
  deletes all files from archive.zip with date of 27 Dec 2005 and later
  Note the * (escape as "*" on Unix) to select all files in archive

  -FS       file sync
  Similar to update, but files updated if date or size of entry does not
  match file on OS.  Also deletes entry from archive if no matching file
  on OS.
    zip archive_to_update -FS -r dir_used_before
  Result generally same as creating new archive, but unchanged entries
  are copied instead of being read and compressed so can be faster.
      WARNING:  -FS deletes entries so make backup copy of archive first

Compression:
  -0        store files (no compression)
  -1 to -9  compress fastest to compress best (default is 6)
  -Z cm     set compression method to cm:
              store   - store without compression, same as option -0
              deflate - original zip deflate, same as -1 to -9 (default)
            if bzip2 is enabled:
              bzip2 - use bzip2 compression (need modern unzip)

Encryption:
  -e        Use standard (weak) PKZip 2.0 encryption, prompt for password
  -P pswd   use standard encryption, password is pswd

Splits (archives created as a set of split files):
  -s ssize  create split archive with splits of size ssize, where ssize nm
              n number and m multiplier (kmgt, default m), 100k -> 100 kB
  -sp       pause after each split closed to allow changing disks
      WARNING:  Archives created with -sp use data descriptors and should
                work with most unzips but may not work with some
  -sb       ring bell when pause
  -sv       be verbose about creating splits
      Split archives CANNOT be updated, but see --out and Copy Mode below

Using --out (output to new archive):
  --out oa  output to new archive oa
  Instead of updating input archive, create new output archive oa.
  Result is same as without --out but in new archive.  Input archive
  unchanged.
      WARNING:  --out ALWAYS overwrites any existing output file
  For example, to create new_archive like old_archive but add newfile1
  and newfile2:
    zip old_archive newfile1 newfile2 --out new_archive
  Cannot update split archive, so use --out to out new archive:
    zip in_split_archive newfile1 newfile2 --out out_split_archive
  If input is split, output will default to same split size
  Use -s=0 or -s- to turn off splitting to convert split to single file:
    zip in_split_archive -s 0 --out out_single_file_archive
      WARNING:  If overwriting old split archive but need less splits,
                old splits not overwritten are not needed but remain

Copy Mode (copying from archive to archive):
  -U        (also --copy) select entries in archive to copy (reverse delete)
  Copy Mode copies entries from old to new archive with --out and is used by
  zip when either no input files on command line or -U (--copy) used.
    zip inarchive --copy pattern pattern ... --out outarchive
  To copy only files matching *.c into new archive, excluding foo.c:
    zip old_archive --copy "*.c" --out new_archive -x foo.c
  If no input files and --out, copy all entries in old archive:
    zip old_archive --out new_archive

Streaming and FIFOs:
  prog1 | zip -ll z -      zip output of prog1 to zipfile z, converting CR LF
  zip - -R "*.c" | prog2   zip *.c files in current dir and stream to prog2 
  prog1 | zip | prog2      zip in pipe with no in or out acts like zip - -
  If Zip is Zip64 enabled, streaming stdin creates Zip64 archives by default
   that need PKZip 4.5 unzipper like UnZip 6.0
  WARNING:  Some archives created with streaming use data descriptors and
            should work with most unzips but may not work with some
  Can use -fz- to turn off Zip64 if input not large (< 4 GB):
    prog_with_small_output | zip archive -fz-

  Zip now can read Unix FIFO (named pipes).  Off by default to prevent zip
  from stopping unexpectedly on unfed pipe, use -FI to enable:
    zip -FI archive fifo

Dots, counts:
  -db       display running count of bytes processed and bytes to go
              (uncompressed size, except delete and copy show stored size)
  -dc       display running count of entries done and entries to go
  -dd       display dots every 10 MB (or dot size) while processing files
  -dg       display dots globally for archive instead of for each file
    zip -qdgds 10m   will turn off most output except dots every 10 MB
  -ds siz   each dot is siz processed where siz is nm as splits (0 no dots)
  -du       display original uncompressed size for each entry as added
  -dv       display volume (disk) number in format in_disk>out_disk
  Dot size is approximate, especially for dot sizes less than 1 MB
  Dot options don't apply to Scanning files dots (dot/2sec) (-q turns off)

Logging:
  -lf path  open file at path as logfile (overwrite existing file)
  -la       append to existing logfile
  -li       include info messages (default just warnings and errors)

Testing archives:
  -T        test completed temp archive with unzip before updating archive
  -TT cmd   use command cmd instead of 'unzip -tqq' to test archive
             On Unix, to use unzip in current directory, could use:
               zip archive file1 file2 -T -TT "./unzip -tqq"
             In cmd, {{}} replaced by temp archive path, else temp appended.
             The return code is checked for success (0 on Unix)

Fixing archives:
  -F        attempt to fix a mostly intact archive (try this first)
  -FF       try to salvage what can (may get more but less reliable)
  Fix options copy entries from potentially bad archive to new archive.
  -F tries to read archive normally and copy only intact entries, while
  -FF tries to salvage what can and may result in incomplete entries.
  Must use --out option to specify output archive:
    zip -F bad.zip --out fixed.zip
  Use -v (verbose) with -FF to see details:
    zip reallybad.zip -FF -v --out fixed.zip
  Currently neither option fixes bad entries, as from text mode ftp get.

Difference mode:
  -DF       (also --dif) only include files that have changed or are
             new as compared to the input archive
  Difference mode can be used to create incremental backups.  For example:
    zip --dif full_backup.zip -r somedir --out diff.zip
  will store all new files, as well as any files in full_backup.zip where
  either file time or size have changed from that in full_backup.zip,
  in new diff.zip.  Output archive not excluded automatically if exists,
  so either use -x to exclude it or put outside what is being zipped.

DOS Archive bit (Windows only):
  -AS       include only files with the DOS Archive bit set
  -AC       after archive created, clear archive bit of included files
      WARNING: Once the archive bits are cleared they are cleared
               Use -T to test the archive before the bits are cleared
               Can also use -sf to save file list before zipping files

Show files:
  -sf       show files to operate on and exit (-sf- logfile only)
  -su       as -sf but show escaped UTF-8 Unicode names also if exist
  -sU       as -sf but show escaped UTF-8 Unicode names instead
  Any character not in the current locale is escaped as #Uxxxx, where x
  is hex digit, if 16-bit code is sufficient, or #Lxxxxxx if 24-bits
  are needed.  If add -UN=e, Zip escapes all non-ASCII characters.

Unicode:
  If compiled with Unicode support, Zip stores UTF-8 path of entries.
  This is backward compatible.  Unicode paths allow better conversion
  of entry names between different character sets.

  New Unicode extra field includes checksum to verify Unicode path
  goes with standard path for that entry (as utilities like ZipNote
  can rename entries).  If these do not match, use below options to
  set what Zip does:
      -UN=Quit     - if mismatch, exit with error
      -UN=Warn     - if mismatch, warn, ignore UTF-8 (default)
      -UN=Ignore   - if mismatch, quietly ignore UTF-8
      -UN=No       - ignore any UTF-8 paths, use standard paths for all
  An exception to -UN=N are entries with new UTF-8 bit set (instead
  of using extra fields).  These are always handled as Unicode.

  Normally Zip escapes all chars outside current char set, but leaves
  as is supported chars, which may not be OK in path names.  -UN=Escape
  escapes any character not ASCII:
    zip -sU -UN=e archive
  Can use either normal path or escaped Unicode path on command line
  to match files in archive.

  Zip now stores UTF-8 in entry path and comment fields on systems
  where UTF-8 char set is default, such as most modern Unix, and
  and on other systems in new extra fields with escaped versions in
  entry path and comment fields for backward compatibility.
  Option -UN=UTF8 will force storing UTF-8 in entry path and comment
  fields:
      -UN=UTF8     - store UTF-8 in entry path and comment fields
  This option can be useful for multi-byte char sets on Windows where
  escaped paths and comments can be too long to be valid as the UTF-8
  versions tend to be shorter.

  Only UTF-8 comments on UTF-8 native systems supported.  UTF-8 comments
  for other systems planned in next release.

Self extractor:
  -A        Adjust offsets - a self extractor is created by prepending
             the extractor executable to archive, but internal offsets
             are then off.  Use -A to fix offsets.
  -J        Junk sfx - removes prepended extractor executable from
             self extractor, leaving a plain zip archive.

More option highlights (see manual for additional options and details):
  -b dir    when creating or updating archive, create the temp archive in
             dir, which allows using seekable temp file when writing to a
             write once CD, such archives compatible with more unzips
             (could require additional file copy if on another device)
  -MM       input patterns must match at least one file and matched files
             must be readable or exit with OPEN error and abort archive
             (without -MM, both are warnings only, and if unreadable files
             are skipped OPEN error (18) returned after archive created)
  -nw       no wildcards (wildcards are like any other character)
  -sc       show command line arguments as processed and exit
  -sd       show debugging as Zip does each step
  -so       show all available options on this system
  -X        default=strip old extra fields, -X- keep old, -X strip most
  -ws       wildcards don't span directory boundaries in paths

    
    "#
    );
}
