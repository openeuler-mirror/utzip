/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

use bzip2::write::BzEncoder;
use crc32fast::Hasher;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::collections::HashSet;
use std::fs::{metadata, File};
use std::io::Seek;
use std::io::SeekFrom;
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use crate::encryption::zipcrypt::{ZipCryptoDecryptor, ZipCryptoEncryptor};

// // 压缩方法枚举
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum CompressionMethod {
    #[default]
    Stored = 0,
    Deflated = 8,
    Bzip2 = 12,
}

// 压缩编码器枚举
pub enum CompressionEncoder<W: Write + 'static> {
    Stored(W),
    Deflate(DeflateEncoder<W>),
    Bzip2(BzEncoder<W>),
    // 仅加密（无压缩）
    Encrypted(ZipCryptoEncryptor<W>),
    // 压缩+加密
    DeflateEncrypted(DeflateEncoder<ZipCryptoEncryptor<W>>),
    Bzip2Encrypted(BzEncoder<ZipCryptoEncryptor<W>>),
}
impl CompressionMethod {
    pub fn to_le_bytes(self) -> [u8; 2] {
        (self as u16).to_le_bytes()
    }

    pub fn from(num: u16) -> Self {
        match num {
            0 => Self::Stored,
            8 => Self::Deflated,
            12 => Self::Bzip2,
            _ => Self::Stored,
        }
    }
}

// ZIP64扩展信息结构
#[derive(Debug, Clone, Default)]
pub struct Zip64ExtendedInfo {
    pub uncompressed_size: Option<u64>,
    pub compressed_size: Option<u64>,
    pub local_header_offset: Option<u64>,
    pub disk_start_number: Option<u32>,
}

// 归档文件基本信息结构体
#[derive(Debug, Default)]
pub struct ArchiveFileInfo {
    pub num_entries: u16,
    pub size: u32,
    pub offset: u32,
    pub comment: String,
    // ZIP64支持
    pub is_zip64: bool,
    pub zip64_num_entries: Option<u64>,
    pub zip64_size: Option<u64>,
    pub zip64_offset: Option<u64>,
}

// 中央目录结构
#[derive(Default, Debug, Clone)]
pub struct CentralDirectoryHeader {
    pub version_made: u16,
    pub version_needed: u16,
    pub flags: u16,
    pub compression: CompressionMethod,
    pub mod_time: u16,
    pub mod_date: u16,
    pub crc32: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub filename: Vec<u8>,
    pub extra_field: Vec<u8>,
    pub file_comment: Vec<u8>,
    pub disk_num: u16,
    pub internal_attr: u16,
    pub external_attr: u32,
    pub local_header_offset: u32,
    // ZIP64支持
    pub zip64_extended_info: Option<Zip64ExtendedInfo>,
}

struct CurrentFile<W: Write + Seek + 'static> {
    name: String,
    header_start: u64,
    data_start: u64,
    compression: CompressionMethod,
    flags: u16,
    password: Option<String>,
    hasher: Hasher,
    bytes_written: u64,
    encoder: Option<CompressionEncoder<W>>,

    mod_time: u16,
    mod_date: u16,
    external_attr: u32,
    disk_num: u16,
    extra_field: Vec<u8>,

    skip_compression: bool, // 是否跳过压缩,跳过后，下面的三个字段才有用
    compress_size: u32,     // 压缩后的大小
    uncompress_size: u64,   // 原始大小
    crc32: u32,             // 原始的crc32

    // 新增：标记压缩级别是否由外部显式指定，用于控制自动Store模式切换
    compression_level_specified: bool,

    // 用于自动切换到Store模式的原始数据缓冲区
    original_data_buffer: Vec<u8>,
    original_compression: CompressionMethod, // 保存原始压缩方法
}

pub struct ZipWriter<'a> {
    file: File,
    cd_headers: Vec<CentralDirectoryHeader>,
    current_file: Option<CurrentFile<File>>,
    output_path: String,
    archive_info: ArchiveFileInfo,

    // 新增分卷支持
    split_size: Option<u64>,  // 分卷大小
    current_split_index: u16, // 当前分卷索引
    base_name: String,        // 基础文件名
    // 回调函数，用于在每个分卷完成后调用
    split_callback: Option<Box<dyn FnMut(u16) -> anyhow::Result<PathBuf> + 'a>>,
    split_bell: bool,    // 是否响铃
    split_verbose: bool, // 是否显示分卷的详细输出
}

#[derive(Debug)]
pub struct ZipArchive {
    file: File,
    cd_headers: Vec<CentralDirectoryHeader>,
    arhive_info: ArchiveFileInfo,
    // 分割文件支持
    #[allow(dead_code)]
    split_files: Option<Vec<String>>, // 分割文件路径列表
    #[allow(dead_code)]
    base_name: Option<String>, // 基础文件名
}

impl ZipArchive {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        Ok(ZipArchive {
            file,
            cd_headers: Vec::new(),
            arhive_info: ArchiveFileInfo::default(),
            split_files: None,
            base_name: None,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct FileOptions {
    pub compression_method: CompressionMethod, // 压缩方法，8表示DEFLATE
    pub password: Option<String>,              // 可选密码
    pub compression_level: u32,                // 新增压缩级别字段
    pub modification_time: Option<(u16, u16)>, // 新增修改时间字段

    pub convert_lf_to_crlf: bool, // 是否将LF转换为CRLF
    pub convert_crlf_to_lf: bool, // 是否将CRLF转换为LF

    pub external_attr: u32,   // 文件属性
    pub extra_field: Vec<u8>, // 额外字段

    pub no_extra_field: bool, // 是否不使用额外字段
    pub store_symlinks: bool, // 是否存储符号链接

    pub no_compress_extensions: HashSet<String>, // 不压缩的文件扩展名列表

    pub skip_compression: bool, // 是否跳过压缩,跳过后，下面的三个字段才有用
    pub compress_size: u32,     // 压缩后的大小
    pub uncompress_size: u64,   // 原始大小
    pub crc32: u32,             // 原始的crc32

    // 新增：标记压缩级别是否由外部显式指定
    pub compression_level_specified: bool, // 压缩级别是否由外部指定
}
