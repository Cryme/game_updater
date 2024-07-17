use shared::admin_panel::{FileInfo, FolderInfo};
use std::ops::Not;
use std::slice::Iter;
use strum::{Display, EnumIter};

#[derive(Eq, PartialEq, Default, Copy, Clone)]
pub(crate) enum SortDir {
    Asc,
    #[default]
    Desc,
}

impl Not for SortDir {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            SortDir::Asc => SortDir::Desc,
            SortDir::Desc => SortDir::Asc,
        }
    }
}

#[derive(Copy, Clone, Default, EnumIter, Display, Eq, PartialEq)]
pub(crate) enum FileSortBy {
    #[default]
    Name,
    Size,
    CreatedAt,
    ModifiedAt,
}

#[derive(Default)]
pub(crate) struct FileInfoHolder {
    folders: Vec<FolderInfo>,
    files: Vec<FileInfo>,
    pub(crate) sort_by: FileSortBy,
    pub(crate) sort_dir: SortDir,
}

impl FileInfoHolder {
    pub fn files(&self) -> Iter<FileInfo> {
        self.files.iter()
    }

    pub fn folders(&self) -> Iter<FolderInfo> {
        self.folders.iter()
    }

    pub fn files_count(&self) -> usize {
        self.files.len()
    }

    pub fn folders_count(&self) -> usize {
        self.folders.len()
    }

    pub fn set(&mut self, files: Vec<FileInfo>, folders: Vec<FolderInfo>) {
        self.files = files;
        self.folders = folders;
        self.sort();
    }

    pub(crate) fn sort(&mut self) {
        self.files.sort_by(|a, b| {
            let ord = match self.sort_by {
                FileSortBy::Name => a.name.cmp(&b.name),
                FileSortBy::Size => a.size.cmp(&b.size),
                FileSortBy::CreatedAt => a.created.cmp(&b.created),
                FileSortBy::ModifiedAt => a.modified_at.cmp(&b.modified_at),
            };

            if self.sort_dir == SortDir::Asc {
                ord.reverse()
            } else {
                ord
            }
        });
        self.folders.sort_by(|a, b| {
            let ord = match self.sort_by {
                FileSortBy::Name => a.name.cmp(&b.name),
                FileSortBy::Size => a.size.cmp(&b.size),
                FileSortBy::CreatedAt => a.created.cmp(&b.created),
                FileSortBy::ModifiedAt => a.modified_at.cmp(&b.modified_at),
            };

            if self.sort_dir == SortDir::Asc {
                ord.reverse()
            } else {
                ord
            }
        });
    }
}
