/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ZipError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Archive not found: {0}")]
    ArchiveNotFound(PathBuf),

    #[error("Entry not found: {0}")]
    EntryNotFound(String),

    #[error("Password required for encrypted archive")]
    PasswordRequired,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Invalid command line arguments: {0}")]
    InvalidArguments(String),

    #[error("utzip error: Nothing to do! ({0})")]
    NothingToDo(String),

    #[error("Pattern error: {0}")]
    PatternError(String),

    #[error("Operation not permitted: {0}")]
    OperationNotPermitted(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Invalid date/time: {0}")]
    InvalidDateTime(String),

    #[error("utzip error: Invalid command arguments ({0})")]
    DuplicateFileName(String),

    #[error("utzip error: {0}")]
    UnzipError(String),

    #[error("utzip error: Interrupted ({0})")]
    Interrupted(String),

    #[error("utzip error: Zip file structure invalid ({0})")]
    InvalidArchive(String),
}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ZipNoteError {
    #[error("utzipnote error: Invalid command arguments ({0})")]
    InvalidArguments(String),
    #[error("utzipnote error: Invalid comment format ({0})")]
    InvalidCommentFormat(String),
    #[error("utzipnote error: Not found ({0})")]
    ArchiveNotFound(String),

    #[error("utzip error: Nothing to do! ({0})")]
    NothingToDo(String),
    #[error("Pattern error: {0}")]
    PatternError(String),
}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ZipCloakError {
    #[error("utzipcloak error: Invalid command arguments ({0})")]
    InvalidArguments(String),
    #[error("utzipcloak error: Not found ({0})")]
    ArchiveNotFound(String),
    #[error("utzipcloak error: Nothing to do! ({0})")]
    NothingToDo(String),
    #[error("Pattern error: {0}")]
    PatternError(String),
}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ZipSplitError {
    #[error("utzipsplit error: Invalid command arguments ({0})")]
    InvalidArguments(String),
    #[error("utzipsplit error: Not found ({0})")]
    ArchiveNotFound(String),
    #[error("utzipsplit error: Nothing to do! ({0})")]
    NothingToDo(String),
    #[error("utzipsplit error: Entry too big to split, read, or write ({0})")]
    EntryTooLarge(String),
}
