/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

// -lf 参数生成的日志文件
use chrono::Local;
use std::io::Write;
use std::path::PathBuf;

pub struct LogFile {
    log_file: std::fs::File,
    log_file_info: bool,
}

impl LogFile {
    pub fn new(log_file_path: PathBuf, append: bool, log_file_info: bool) -> Self {
        let log_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(log_file_path)
            .expect("Failed to open log file");

        LogFile {
            log_file,
            log_file_info,
        }
    }

    pub fn log_command(&mut self, args: &[String]) -> anyhow::Result<()> {
        // 跳过第一个参数（命令本身）
        let filtered_args = if args.len() > 1 {
            args[1..].join(" ")
        } else {
            String::new()
        };

        writeln!(
            self.log_file,
            "---------\nUtzip log opened {}",
            Local::now().format("%a %b %d %H:%M:%S %Y")
        )?;
        writeln!(
            self.log_file,
            "command line arguments:\n {}\n",
            filtered_args
        )?;

        Ok(())
    }

    // 写入日志，enter 为 None 时不换行
    pub fn write_log(&mut self, message: &str, enter: Option<()>) -> anyhow::Result<()> {
        if self.log_file_info {
            if enter.is_some() {
                writeln!(self.log_file, "{}", message)?;
            } else {
                write!(self.log_file, "{}", message)?;
            }
        }
        Ok(())
    }

    pub fn log_summary(
        &mut self,
        total_files: usize,
        total_original_size: u64,
    ) -> anyhow::Result<()> {
        let format_size = |size: u64| {
            if size < 1024 {
                format!("{}B", size)
            } else if size < 1024 * 1024 {
                format!("{:.0}K", size as f64 / 1024.0)
            } else if size < 1024 * 1024 * 1024 {
                format!("{:.0}M", size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.0}G", size as f64 / (1024.0 * 1024.0 * 1024.0))
            }
        };
        let total_original_size = format_size(total_original_size);
        writeln!(
            self.log_file,
            "\nTotal {} entries ({} bytes)",
            total_files, total_original_size
        )?;

        writeln!(
            self.log_file,
            "Done {}",
            Local::now().format("%a %b %d %H:%M:%S %Y")
        )?;

        Ok(())
    }

    pub fn close(&mut self) -> anyhow::Result<()> {
        self.log_file.flush()?;
        Ok(())
    }
}
