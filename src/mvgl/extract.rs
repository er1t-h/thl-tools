use std::{borrow::Cow, collections::HashSet, io, path::Path, sync::Mutex};

use bitvec::{order::Lsb0, vec::BitVec};
use indicatif::{MultiProgress, ProgressBar, ProgressFinish};
use rayon::prelude::*;
use regex::Regex;

use crate::helpers::{
    indicatif::{IndicatifProgressExt, byte_bar_style_with_message_header},
    traits::ReadSeekSendSync,
};

use super::{CompressedFileHandle, DecompressedFile, MVGLArchive};

pub struct Extractor<'a> {
    multi_progress: Option<&'a MultiProgress>,
    name_matcher: Option<Regex>,
    rename_images: bool,
    overwrite: bool,
    multi_threading: bool,
}

impl Default for Extractor<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Extractor<'a> {
    fn handle_path_renaming<'p>(&self, path: &'p Path) -> Cow<'p, Path> {
        match path.extension().and_then(|x| x.to_str()) {
            Some("img") if self.rename_images => Cow::Owned(path.with_extension("dds")),
            _ => Cow::Borrowed(path),
        }
    }

    pub const fn new() -> Self {
        Self {
            multi_progress: None,
            name_matcher: None,
            rename_images: false,
            overwrite: false,
            multi_threading: true,
        }
    }

    pub fn with_rename_images(self, rename_images: bool) -> Self {
        Self {
            rename_images,
            ..self
        }
    }

    pub fn with_multi_threading(self, multi_threading: bool) -> Self {
        Self {
            multi_threading,
            ..self
        }
    }

    pub fn with_overwrite(self, overwrite: bool) -> Self {
        Self { overwrite, ..self }
    }

    pub fn with_multi_progress(self, multi_progress: Option<&'a MultiProgress>) -> Self {
        Self {
            multi_progress,
            ..self
        }
    }

    pub fn with_name_matcher(self, name_matcher: Option<Regex>) -> Self {
        Self {
            name_matcher,
            ..self
        }
    }

    pub fn extract(&self, reader: &mut dyn ReadSeekSendSync, destination: &Path) -> io::Result<()> {
        std::fs::create_dir_all(destination)?;
        let archive = MVGLArchive::from_reader(reader)?;
        let mut total_compressed_size = 0;
        let mut entry_skip_status = BitVec::<u8, Lsb0>::with_capacity(archive.len());

        for file in archive.iter() {
            let mut should_skip = if let Some(name_matcher) = &self.name_matcher {
                !name_matcher.is_match(&file.info.name)
            } else {
                false
            };

            if !self.overwrite {
                let path = self.handle_path_renaming(Path::new(&file.info.name));
                should_skip = should_skip || (!self.overwrite && destination.join(&path).exists());
            }

            entry_skip_status.push(should_skip);
            if !should_skip {
                total_compressed_size += file.info.compressed_size;
            }
        }

        let created_dirnames = Mutex::new(HashSet::new());
        let progress_bar = ProgressBar::new(total_compressed_size)
            .with_style(byte_bar_style_with_message_header("extracting files"))
            .with_finish(ProgressFinish::WithMessage(Cow::Borrowed(
                "finished extracting all files",
            )))
            .in_optional_multi_progress(self.multi_progress);

        let closure =
            |(should_skip, handle): (bool, CompressedFileHandle<'_, &mut dyn ReadSeekSendSync>)| {
                if should_skip {
                    return Ok(());
                }
                progress_bar.set_message(handle.info.name.clone());
                let path = Path::new(&handle.info.name);

                if let Some(dirname) = path.parent() {
                    let mut lock = created_dirnames.lock().unwrap();
                    if lock.insert(dirname.to_path_buf()) {
                        std::fs::create_dir_all(destination.join(dirname))?;
                    }
                }

                let path = self.handle_path_renaming(Path::new(&handle.info.name));
                let file_name = destination.join(path);

                let compressed_file = handle.info.compressed_size;
                let content = handle.read()?;
                let decompressed = content
                    .decompress()
                    .map_or_else(|| content.into_inner(), DecompressedFile::into_inner);
                std::fs::write(file_name, &decompressed)?;
                progress_bar.inc(compressed_file);
                Ok(())
            };

        let mut iter = entry_skip_status.into_iter().zip(archive.iter());
        if self.multi_threading {
            iter.par_bridge().try_for_each(closure)
        } else {
            iter.try_for_each(closure)
        }
    }
}
