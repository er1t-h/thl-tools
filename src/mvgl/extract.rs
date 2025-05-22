use std::{collections::HashSet, io, path::Path};

use bitvec::{order::Lsb0, vec::BitVec};
use indicatif::{MultiProgress, ProgressBar, ProgressIterator};
use regex::Regex;

use crate::helpers::{
    indicatif::{IndicatifProgressExt, byte_bar_style_with_message_header},
    traits::ReadSeek,
};

use super::ContentIterator;

pub struct Extractor<'a> {
    multi_progress: Option<&'a MultiProgress>,
    name_matcher: Option<Regex>,
    rename_images: bool,
}

impl Default for Extractor<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Extractor<'a> {
    pub const fn new() -> Self {
        Self {
            multi_progress: None,
            name_matcher: None,
            rename_images: false,
        }
    }

    pub fn with_rename_images(self, rename_images: bool) -> Self {
        Self {
            rename_images,
            ..self
        }
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

    pub fn extract(&self, reader: &mut dyn ReadSeek, destination: &Path) -> io::Result<()> {
        std::fs::create_dir_all(destination)?;
        let mut content_iterator = ContentIterator::new(reader)?;
        let mut total_compressed_size = 0;
        let mut entry_skip_status =
            BitVec::<u8, Lsb0>::with_capacity(content_iterator.file_infos().len());

        for file in content_iterator.file_infos() {
            let should_skip = if let Some(name_matcher) = &self.name_matcher {
                !name_matcher.is_match(&file.associated_struct.name)
            } else {
                false
            };

            entry_skip_status.push(should_skip);
            if !should_skip {
                total_compressed_size += file.compressed_size;
            }
        }

        let mut created_dirnames = HashSet::new();
        let progress_bar = ProgressBar::new(total_compressed_size)
            .with_style(byte_bar_style_with_message_header("extracting files"));

        let mut nth = 0;
        for should_skip in entry_skip_status
            .into_iter()
            .progress_with(progress_bar.clone())
            .in_optional_multi_progress(self.multi_progress)
        {
            if should_skip {
                nth += 1;
                continue;
            }
            progress_bar.set_message(
                content_iterator.file_infos()[nth]
                    .associated_struct
                    .name
                    .clone(),
            );
            let (info, content) = content_iterator.nth(nth).unwrap()?;
            nth = 0;

            if let Some((dirname, _)) = info.associated_struct.name.rsplit_once('/') {
                let format = format!("{}/{}", destination.display(), dirname);
                if !created_dirnames.contains(&format) {
                    std::fs::create_dir_all(&format)?;
                    created_dirnames.insert(format);
                }
            }

            let file_name = if self.rename_images {
                if let Some(x) = info.associated_struct.name.strip_suffix(".img") {
                    format!("{}/{x}.dds", destination.display())
                } else {
                    format!("{}/{}", destination.display(), info.associated_struct.name)
                }
            } else {
                format!("{}/{}", destination.display(), info.associated_struct.name)
            };

            progress_bar.inc(info.compressed_size);
            std::fs::write(file_name, &content)?;
        }

        Ok(())
    }
}
