/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

// 压缩方法枚举
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum CompressionMethod {
    #[default]
    Stored = 0,
    Deflated = 8,
    Bzip2 = 12,
}
