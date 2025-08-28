/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

use crate::cli;
use crate::utils::logfile::LogFile;
use crate::zip::{CompressionMethod, FileOptions, ZipArchive, ZipWriter};
use anyhow::{Context, Result};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

// 跨文件系统安全的文件移动函数
pub fn safe_move_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    // 首先尝试快速的 rename 操作
    match fs::rename(from, to) {
        Ok(()) => {
            log::debug!(
                "Successfully renamed {} to {}",
                from.display(),
                to.display()
            );
            Ok(())
        }
        Err(e) => {
            // 如果是跨设备错误（EXDEV），使用复制+删除的方式
            if e.raw_os_error() == Some(18) {
                // EXDEV: Invalid cross-device link
                log::info!(
                    "Cross-device operation detected, falling back to copy+delete: {} -> {}",
                    from.display(),
                    to.display()
                );

                // 使用复制+删除的方式
                match fs::copy(from, to) {
                    Ok(_) => {
                        // 复制成功，删除原文件
                        match fs::remove_file(from) {
                            Ok(()) => {
                                log::debug!(
                                    "Successfully copied and removed: {} -> {}",
                                    from.display(),
                                    to.display()
                                );
                                Ok(())
                            }
                            Err(remove_err) => {
                                // 删除失败，清理已复制的文件
                                let _ = fs::remove_file(to);
                                Err(anyhow::anyhow!(
                                    "Failed to remove source file after copy: {} ({})",
                                    from.display(),
                                    remove_err
                                ))
                            }
                        }
                    }
                    Err(copy_err) => Err(anyhow::anyhow!(
                        "Failed to copy file {} to {}: {}",
                        from.display(),
                        to.display(),
                        copy_err
                    )),
                }
            } else {
                // 其他类型的错误，直接返回
                Err(anyhow::anyhow!(
                    "Failed to move file {} to {}: {}",
                    from.display(),
                    to.display(),
                    e
                ))
            }
        }
    }
}

// 生成类似标准zip工具的随机临时文件名
fn generate_temp_filename() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    // 使用时间戳和进程ID来生成更加随机的文件名
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let pid = std::process::id();
    let timestamp_part = (now.as_nanos() % 0xFFFFFF) as u32;

    // 生成类似 ziABC123 这样的文件名，类似标准zip工具
    format!("zi{:06X}", (pid ^ timestamp_part) & 0xFFFFFF)
}

// 添加新的结构体来跟踪压缩信息
pub struct FileCompressionTracker {
    pub original_size: u64,
    pub compressed_size: u64,
    pub ratio: f64,
    pub method: CompressionMethod,
    #[allow(dead_code)]
    pub disk_num: u16,
}

#[derive(Default)]
pub struct RunState<'a> {
    pub zip_file: Option<PathBuf>,
    pub zip_file_tmp: Option<PathBuf>,
    pub writer: Option<ZipWriter<'a>>,
    pub archive: Option<ZipArchive>,
    pub file_options: FileOptions,
    pub dirs_to_remove: std::collections::HashSet<PathBuf>, // 待删除的目录

    pub total_original_size: u64,
    pub total_compressed_size: u64,
    pub total_entries: usize, // 统计文件数量

    pub changed_files: Vec<String>, // 保存已修改的文件列表

    pub update_modify_time: bool, // 是否更新修改时间

    pub testing: bool, // 启用测试模式

    pub verbose: bool,    // 启用详细输出
    pub quiet: bool,      // 启用安静模式
    pub show_debug: bool, // 启用调试模式 (--sd)

    output: Option<PathBuf>, // 输出文件路径

    pub log_file: Option<LogFile>, // 日志文件

    // 显示输出控制
    pub display_bytes: bool,        // --db
    pub display_count: bool,        // --dc
    pub display_dots: bool,         // --dd
    pub display_global_dots: bool,  // --dg
    pub dot_size: u64,              // --ds
    pub display_uncompressed: bool, // --du
    pub display_volume: bool,       // --dv

    pub disk_num: u16,
    pub changed_files_count: u16,
    pub changed_files_size: u64,
    last_changed_file_size: u64,
    pub changed_files_total_size: u64,
    pub changed_files_total_count: u16,

    pub args: cli::ZipArgs,

    global_bytes_processed: u64,
    #[allow(dead_code)]
    global_dots_shown: u64,
}

// 手动实现Debug，跳过writer和archive字段
impl<'a> std::fmt::Debug for RunState<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunState")
            .field("zip_file", &self.zip_file)
            .field("zip_file_tmp", &self.zip_file_tmp)
            .field("file_options", &self.file_options)
            .field("dirs_to_remove", &self.dirs_to_remove)
            .field("total_original_size", &self.total_original_size)
            .field("total_compressed_size", &self.total_compressed_size)
            .field("changed_files", &self.changed_files)
            .field("update_modify_time", &self.update_modify_time)
            .field("testing", &self.testing)
            .field("verbose", &self.verbose)
            .field("quiet", &self.quiet)
            .finish()
    }
}

impl<'a> RunState<'a> {
    pub fn new(zipfile: Option<PathBuf>) -> Self {
        // 初始化RunState
        let zip_file = zipfile;
        let zip_file_tmp = None; // 临时文件路径将在init_run_state中根据-b参数设置
        Self {
            zip_file,
            zip_file_tmp,
            writer: None,
            archive: None,
            file_options: FileOptions::new(),
            dirs_to_remove: std::collections::HashSet::new(),
            changed_files: Vec::new(),
            output: None,
            log_file: None,
            args: cli::ZipArgs::default(),
            disk_num: 1,
            ..Default::default()
        }
    }
}

// 定义 trait 来统一不同数据类型的接口
pub trait SizeProvider {
    fn get_size(&self) -> u64;
}

// 为 PathBuf 实现 SizeProvider
impl SizeProvider for &PathBuf {
    fn get_size(&self) -> u64 {
        self.metadata()
            .map(|m| if self.is_dir() { 0 } else { m.len() })
            .unwrap_or(0)
    }
}
// 为 u64 实现 SizeProvider
impl SizeProvider for u32 {
    fn get_size(&self) -> u64 {
        *self as u64
    }
}
// 为 u64 实现 SizeProvider
impl SizeProvider for u64 {
    fn get_size(&self) -> u64 {
        *self
    }
}
