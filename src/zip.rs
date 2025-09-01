/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

use bzip2::write::BzEncoder;
use chrono::{Datelike, Local, TimeZone, Timelike};
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

use crate::utils::common::get_file_modification_time;

pub const ZIP_CRYPTO_FLAG: u16 = 0x1;
pub const VERSION_MADE: u16 = 0x031E; // 3.0 (Unix)
pub const VERSION_NEEDED: u16 = 0x0A; // 1.0
pub const VERSION_NEEDED_ZIP64: u16 = 0x2D; // 4.5 for ZIP64

// ZIP64常量
pub const ZIP64_VERSION_MADE: u16 = 0x032D; // 4.5 (Unix)
pub const ZIP64_EXTRA_FIELD_ID: u16 = 0x0001; // ZIP64扩展信息额外字段标识符
#[allow(dead_code)]
pub const ZIP64_END_OF_CENTRAL_DIR_SIZE: usize = 56; // ZIP64结束目录记录大小
pub const ZIP64_END_OF_CENTRAL_DIR_LOCATOR_SIZE: usize = 20; // ZIP64结束目录定位器大小
pub const MAX_ZIP_SIZE: u32 = 0xFFFFFFFF; // 4GB - 1 (ZIP格式32位限制)
pub const MAX_ZIP_ENTRIES: u16 = 0xFFFF; // 65535 (ZIP格式16位限制)

// 压缩方法枚举
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum CompressionMethod {
    #[default]
    Stored = 0,
    Deflated = 8,
    Bzip2 = 12,
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

impl std::fmt::Display for CompressionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionMethod::Stored => write!(f, "stored"),
            CompressionMethod::Deflated => write!(f, "deflated"),
            CompressionMethod::Bzip2 => write!(f, "bzipped"),
        }
    }
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

// ZIP64扩展信息结构
#[derive(Debug, Clone, Default)]
pub struct Zip64ExtendedInfo {
    pub uncompressed_size: Option<u64>,
    pub compressed_size: Option<u64>,
    pub local_header_offset: Option<u64>,
    pub disk_start_number: Option<u32>,
}

impl Zip64ExtendedInfo {
    pub fn new() -> Self {
        Self::default()
    }

    // 根据ZIP64规范，extra field的字段顺序必须是固定的
    // 并且只有当标准字段为最大值时，对应的ZIP64字段才会被写入
    pub fn to_bytes(
        &self,
        uncompressed_max: bool,
        compressed_max: bool,
        offset_max: bool,
    ) -> Vec<u8> {
        let mut data = Vec::new();

        // 按照ZIP64规范的固定顺序写入字段
        if uncompressed_max {
            if let Some(size) = self.uncompressed_size {
                data.extend_from_slice(&size.to_le_bytes());
            }
        }
        if compressed_max {
            if let Some(size) = self.compressed_size {
                data.extend_from_slice(&size.to_le_bytes());
            }
        }
        if offset_max {
            if let Some(offset) = self.local_header_offset {
                data.extend_from_slice(&offset.to_le_bytes());
            }
        }
        // 磁盘编号字段（通常不用于单个文件）
        if let Some(disk) = self.disk_start_number {
            data.extend_from_slice(&disk.to_le_bytes());
        }

        data
    }

    // 向后兼容的方法
    #[allow(dead_code)]
    pub fn to_bytes_compat(&self) -> Vec<u8> {
        let mut data = Vec::new();

        if let Some(size) = self.uncompressed_size {
            data.extend_from_slice(&size.to_le_bytes());
        }
        if let Some(size) = self.compressed_size {
            data.extend_from_slice(&size.to_le_bytes());
        }
        if let Some(offset) = self.local_header_offset {
            data.extend_from_slice(&offset.to_le_bytes());
        }
        if let Some(disk) = self.disk_start_number {
            data.extend_from_slice(&disk.to_le_bytes());
        }

        data
    }

    pub fn from_bytes(data: &[u8]) -> anyhow::Result<Self> {
        let mut info = Self::new();
        let mut offset = 0;
        // 根据数据长度确定包含哪些字段
        if data.len() >= 8 {
            info.uncompressed_size = Some(u64::from_le_bytes(data[offset..offset + 8].try_into()?));
            offset += 8;
        }
        if data.len() >= 16 {
            info.compressed_size = Some(u64::from_le_bytes(data[offset..offset + 8].try_into()?));
            offset += 8;
        }
        if data.len() >= 24 {
            info.local_header_offset =
                Some(u64::from_le_bytes(data[offset..offset + 8].try_into()?));
            offset += 8;
        }
        if data.len() >= 28 {
            info.disk_start_number = Some(u32::from_le_bytes(data[offset..offset + 4].try_into()?));
        }

        Ok(info)
    }
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

impl CentralDirectoryHeader {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            version_made: VERSION_MADE,
            version_needed: VERSION_NEEDED,
            zip64_extended_info: None,
            ..Default::default()
        }
    }

    pub fn needs_zip64(&self) -> bool {
        let uncompressed = self.get_uncompressed_size();
        let compressed = self.get_compressed_size();
        let offset = self.get_local_header_offset();

        uncompressed > MAX_ZIP_SIZE as u64
            || compressed > MAX_ZIP_SIZE as u64
            || offset > MAX_ZIP_SIZE as u64
    }

    pub fn get_uncompressed_size(&self) -> u64 {
        self.zip64_extended_info
            .as_ref()
            .and_then(|info| info.uncompressed_size)
            .unwrap_or(self.uncompressed_size as u64)
    }

    pub fn get_compressed_size(&self) -> u64 {
        self.zip64_extended_info
            .as_ref()
            .and_then(|info| info.compressed_size)
            .unwrap_or(self.compressed_size as u64)
    }

    pub fn get_local_header_offset(&self) -> u64 {
        self.zip64_extended_info
            .as_ref()
            .and_then(|info| info.local_header_offset)
            .unwrap_or(self.local_header_offset as u64)
    }
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

// 新增枚举定义转换类型
#[derive(Debug, Clone, Copy)]
pub enum LineEndingConversion {
    None,
    LfToCrlf, // Unix -> Windows
    CrlfToLf, // Windows -> Unix
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

impl FileOptions {
    pub fn new() -> Self {
        Self {
            compression_method: CompressionMethod::Deflated, // DEFLATE压缩
            password: None,
            compression_level: 6, // 默认压缩级别为6
            modification_time: None,
            no_compress_extensions: HashSet::from([
                ".zip".to_string(),
                ".Z".to_string(),
                ".zoo".to_string(),
                ".arc".to_string(),
                ".arj".to_string(),
            ]),
            compression_level_specified: false, // 默认为未指定
            ..Default::default()
        }
    }

    pub fn set_file_path(&mut self, file_path: &PathBuf) -> anyhow::Result<()> {
        // 根据传入文件路径设置文件属性
        // 获取文件修改时间并转换为ZIP格式时间戳
        let (time, date) = get_file_modification_time(file_path)?;
        self.with_modification_time((time, date));

        if !self.no_extra_field {
            self.set_ut_extra_field(file_path)?;
        }
        self.with_file_attrs(file_path)?;

        if file_path.is_dir() {
            self.with_compression(CompressionMethod::Stored);
        }

        // 检查文件扩展名是否在不压缩列表中
        let extension = file_path.extension().map_or(String::new(), |e| {
            let ext = e.to_string_lossy();
            if ext.starts_with('.') {
                ext.to_string()
            } else {
                format!(".{}", ext)
            }
        });
        log::debug!(
            "extension: {:?}, no_compress_extensions: {:?}",
            extension,
            self.no_compress_extensions
        );
        if self.no_compress_extensions.contains(&extension) {
            self.with_compression(CompressionMethod::Stored);
        }

        if file_path.is_file() {
            let metadata = metadata(file_path)?;
            let file_size = metadata.len();
            self.uncompress_size = file_size;

            let crc32 = self.caculate_crc32(file_path)?;
            self.crc32 = crc32;

            // 根据文件大小动态优化压缩级别
            if self.compression_method == CompressionMethod::Deflated {
                self.optimize_compression_level_for_size(file_size);
            }

            log::debug!("File '{}' size: {} bytes", file_path.display(), file_size);
        }

        Ok(())
    }

    fn caculate_crc32(&self, file_path: &PathBuf) -> anyhow::Result<u32> {
        let mut hasher = Hasher::new();
        let mut file = File::open(file_path)?;
        let mut buffer = Box::new([0u8; 32 * 1024]); // 32KB buffer
        loop {
            let bytes_read = file.read(&mut *buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        Ok(hasher.finalize())
    }

    pub fn get_line_ending_conversion(&mut self, is_text: bool) -> LineEndingConversion {
        if !is_text {
            return LineEndingConversion::None;
        }
        if self.convert_lf_to_crlf {
            LineEndingConversion::LfToCrlf
        } else if self.convert_crlf_to_lf {
            LineEndingConversion::CrlfToLf
        } else {
            LineEndingConversion::None
        }
    }

    pub fn with_password(&mut self, password: &str) {
        self.password = Some(password.to_string());
    }

    #[allow(dead_code)]
    pub fn with_skip_compression(&mut self, skip: bool) -> &mut Self {
        self.skip_compression = skip;
        self
    }

    pub fn with_compression(&mut self, method: CompressionMethod) {
        self.compression_method = method;

        // 只有在压缩级别未被外部指定时，才设置默认值
        if !self.compression_level_specified {
            if method == CompressionMethod::Stored {
                self.compression_level = 0; // 如果使用存储方法，则不需要压缩级别
            } else if method == CompressionMethod::Deflated {
                self.compression_level = 6; // 默认使用优化的压缩级别
            } else if method == CompressionMethod::Bzip2 {
                self.compression_level = 9; // Bzip2默认压缩级别
            }
        }
    }
    pub fn with_compression_level(&mut self, level: u32) {
        self.compression_level = level;
        self.compression_level_specified = true; // 标记为外部指定
    }

    // 根据文件大小优化压缩级别（仅在未外部指定时）
    pub fn optimize_compression_level_for_size(&mut self, file_size: u64) {
        // 如果压缩级别已经由外部指定，则不进行自动优化
        if self.compression_level_specified {
            log::debug!(
                "Compression level {} was externally specified, skipping optimization for {} bytes file",
                self.compression_level,
                file_size
            );
            return;
        }

        let original_level = self.compression_level;
        self.compression_level = match file_size {
            0..=100 => 1,        // 极小文件：最低级别，速度优先
            101..=1024 => 1,     // 小文件：级别1，最佳平衡点
            1025..=10240 => 2,   // 中小文件：级别2，稍好压缩
            10241..=102400 => 3, // 中等文件：级别3
            _ => 6,              // 大文件：标准级别，压缩比优先
        };

        log::debug!(
            "Auto-optimized compression level for {} bytes file: {} -> {}",
            file_size,
            original_level,
            self.compression_level
        );
    }

    fn with_modification_time(&mut self, time: (u16, u16)) {
        self.modification_time = Some(time);
    }

    //从实际文件获取权限
    fn with_file_attrs(&mut self, path: &Path) -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let metadata = metadata(path)?;
        let mode = metadata.permissions().mode();

        // 高16位: Unix属性 (文件类型+权限)
        // 低16位: DOS属性 (兼容Windows)
        self.external_attr =
            ((mode as u32 & 0xFFFF) << 16) | if metadata.is_dir() { 0x10 } else { 0x20 };

        Ok(())
    }

    // 获取utime时间戳
    fn set_ut_extra_field(&mut self, file_path: &Path) -> anyhow::Result<()> {
        let metadata = metadata(file_path)?;
        let mod_time = metadata
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as u32;

        let mut field = Vec::with_capacity(7);
        field.extend_from_slice(&0x5455u16.to_le_bytes()); // Header ID
        field.extend_from_slice(&5u16.to_le_bytes()); // Data Size
        field.push(0x01); // Flags: modtime present
        field.extend_from_slice(&(mod_time as u32).to_le_bytes()); // modtime (UTC, u32)

        self.extra_field = field.clone();
        Ok(())
    }
}

// 新增 ZipFile 结构体
#[derive(Debug, Clone)]
pub struct ZipFile {
    header: CentralDirectoryHeader,
    #[allow(dead_code)]
    data_start: u64,
    data_end: u64,
    file: Arc<File>,
}

impl ZipFile {
    pub fn name(&self) -> String {
        String::from_utf8_lossy(&self.header.filename).to_string()
    }

    #[allow(dead_code)]
    pub fn extra_field(&self) -> &[u8] {
        &self.header.extra_field
    }

    #[allow(dead_code)]
    pub fn header(&self) -> &CentralDirectoryHeader {
        &self.header
    }
    #[allow(dead_code)]
    pub fn header_mut(&mut self) -> &mut CentralDirectoryHeader {
        &mut self.header
    }

    #[allow(dead_code)]
    pub fn comments(&self) -> String {
        String::from_utf8_lossy(&self.header.file_comment).to_string()
    }

    #[allow(dead_code)]
    pub fn set_comments(&mut self, comment: &str) {
        self.header.file_comment = comment.as_bytes().to_vec();
    }

    #[allow(dead_code)]
    pub fn options(&self) -> FileOptions {
        let mut file_options = FileOptions::new();
        file_options.compression_method = self.header.compression;
        file_options.password = None;
        file_options.compression_level = match self.header.compression {
            CompressionMethod::Stored => 0,
            CompressionMethod::Deflated => 6, // 默认压缩级别
            CompressionMethod::Bzip2 => 9,    // Bzip2默认压缩级别
        };
        file_options.modification_time = Some((self.header.mod_time, self.header.mod_date));
        file_options.external_attr = self.header.external_attr;
        file_options.extra_field = self.header.extra_field.clone();

        file_options.compress_size = self.header.compressed_size;
        file_options.uncompress_size = self.header.uncompressed_size as u64;
        file_options.crc32 = self.header.crc32;

        file_options
    }

    #[allow(dead_code)]
    pub fn is_dir(&self) -> bool {
        self.header.external_attr & 0x10 != 0
            || (!self.header.filename.is_empty() && *self.header.filename.last().unwrap() == b'/')
    }

    pub fn encrypted(&self) -> bool {
        self.header.flags & ZIP_CRYPTO_FLAG != 0
    }

    pub fn last_modified(&self) -> anyhow::Result<chrono::DateTime<Local>> {
        // 解析时间字段 (MS-DOS 时间格式)
        let time = self.header.mod_time;
        let hour = ((time >> 11) & 0x1F) as u32;
        let minute = ((time >> 5) & 0x3F) as u32;
        let second = (time & 0x1F) as u32 * 2; // MS-DOS 时间存储秒/2

        // 解析日期字段 (MS-DOS 日期格式)
        let date = self.header.mod_date;
        let day = (date & 0x1F) as u32;
        let month = ((date >> 5) & 0xF) as u32;
        let year = (date >> 9) as u32 + 1980; // MS-DOS 日期从1980年开始

        // 创建日期时间对象
        Local
            .with_ymd_and_hms(year as i32, month, day, hour, minute, second)
            .single()
            .ok_or_else(|| anyhow::anyhow!("Invalid date time in zip header"))
    }

    pub fn origin_size(&self) -> u64 {
        self.header.get_uncompressed_size()
    }

    pub fn compressed_size(&self) -> u64 {
        self.header.get_compressed_size()
    }
}
