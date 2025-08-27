/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::cli;
use crate::error::ZipSplitError;
use crate::zip::ZipArchive;
use anyhow::{Ok, Result};

pub struct ZipSplitter<'a> {
    archive: ZipArchive,
    args: &'a cli::ZipSplitArgs,
}

impl<'a> ZipSplitter<'a> {
    pub fn new(args: &'a cli::ZipSplitArgs) -> Result<Self> {
        let zip_path = args.zipfile.clone().unwrap();
        if !zip_path.exists() {
            log::error!("Zip file not found: {}", zip_path.display());
            return Err(ZipSplitError::ArchiveNotFound(zip_path.display().to_string()).into());
        }
        let archive = ZipArchive::new(zip_path.to_str().unwrap())?;
        Ok(Self { archive, args })
    }
}

fn main() {
    println!("Hello, world!");
}
