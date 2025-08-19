/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

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
