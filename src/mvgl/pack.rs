use std::{borrow::Cow, ffi::OsStr, fs, io::SeekFrom, path::Path, time::Duration};

use byteorder::{LittleEndian, WriteBytesExt};
use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressIterator};
use lz4::block::CompressionMode;
use walkdir::WalkDir;

use crate::helpers::{
    indicatif::{
        IndicatifProgressExt, default_bar_style_with_message_header, default_spinner_style,
    },
    traits::WriteSeek,
};

use super::{EMPTY_SLICED_PATH, SlicedPath};

#[derive(Debug)]
struct TreeNode<'a> {
    compare_bit: u16,
    left: u16,
    right: u16,
    name: &'a SlicedPath,
}

#[derive(Debug, Clone, Copy, Default)]
struct Header1 {
    compare_bit: u32,
    id: u32,
    left: u32,
    right: u32,
}

fn find_first_bit_mismatch<'a>(
    first: u16,
    nodeless: &[&'a SlicedPath],
    with_node: &[&'a SlicedPath],
) -> TreeNode<'a> {
    if with_node.is_empty() {
        return TreeNode {
            compare_bit: first,
            left: 0,
            right: 0,
            name: nodeless[0],
        };
    }
    for i in first.. {
        let mut set = false;
        let mut unset = false;
        for file in with_node {
            if ((file[i >> 3] >> (i & 7)) & 1) != 0 {
                set = true;
            } else {
                unset = true;
            }
            if set && unset {
                return TreeNode {
                    compare_bit: i,
                    left: 0,
                    right: 0,
                    name: nodeless[0],
                };
            }
        }

        if let Some(node) = nodeless.iter().find(|&file| {
            let val = (file[i >> 3] >> (i & 7)) & 1 != 0;
            val && unset || !val && set
        }) {
            return TreeNode {
                compare_bit: i,
                left: 0,
                right: 0,
                name: node,
            };
        }
    }
    TreeNode {
        compare_bit: u16::MAX,
        left: u16::MAX,
        right: 0,
        name: EMPTY_SLICED_PATH,
    }
}

fn generate_tree(all_paths: &'_ [SlicedPath]) -> Vec<TreeNode<'_>> {
    #[derive(Debug)]
    struct QueueEntry<'a> {
        parent: u16,
        val: u16,
        list: Vec<&'a SlicedPath>,
        node_list: Vec<&'a SlicedPath>,
        is_left: bool,
    }

    let mut nodes = vec![TreeNode {
        compare_bit: 0xffff,
        left: 0,
        right: 0,
        name: EMPTY_SLICED_PATH,
    }];
    let mut queue = Vec::from([QueueEntry {
        parent: 0,
        val: 0xffff,
        list: all_paths.iter().collect::<Vec<_>>(),
        node_list: Vec::new(),
        is_left: false,
    }]);

    while let Some(entry) = queue.pop() {
        let mut nodeless = vec![];
        let mut with_node = vec![];

        for &file in &entry.list {
            if entry.node_list.contains(&file) {
                with_node.push(file);
            } else {
                nodeless.push(file);
            }
        }

        if nodeless.is_empty() {
            let first = entry.list[0];
            let position = nodes.iter().position(|node| node.name == first).unwrap();
            let parent = &mut nodes[entry.parent as usize];
            if entry.is_left {
                parent.left = position as u16;
            } else {
                parent.right = position as u16;
            }
            continue;
        }

        let child = find_first_bit_mismatch(entry.val.wrapping_add(1), &nodeless, &with_node);

        let len = nodes.len() as u16;
        let parent = &mut nodes[entry.parent as usize];
        if entry.is_left {
            parent.left = len;
        } else {
            parent.right = len;
        }

        let mut left = Vec::new();
        let mut right = Vec::new();

        for file in entry.list {
            if (file[child.compare_bit >> 3] >> (child.compare_bit & 7)) & 1 != 0 {
                right.push(file);
            } else {
                left.push(file);
            }
        }

        let mut new_node_list = entry.node_list;
        new_node_list.push(child.name);

        if !left.is_empty() {
            queue.push(QueueEntry {
                parent: nodes.len() as u16,
                val: child.compare_bit,
                list: left,
                node_list: new_node_list.clone(),
                is_left: true,
            });
        }
        if !right.is_empty() {
            queue.push(QueueEntry {
                parent: nodes.len() as u16,
                val: child.compare_bit,
                list: right,
                node_list: new_node_list,
                is_left: false,
            });
        }
        nodes.push(child);
    }

    nodes
}

pub struct Packer<'a> {
    rename_images: bool,
    multi_progress: Option<&'a MultiProgress>,
}

impl Default for Packer<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Packer<'a> {
    pub const fn new() -> Self {
        Self {
            rename_images: false,
            multi_progress: None,
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

    pub fn pack(&self, source_dir: &Path, target_file: &mut dyn WriteSeek) -> std::io::Result<()> {
        let collecting_files_progress = ProgressBar::new_spinner()
            .with_elapsed(Duration::from_secs(0))
            .with_message("collecting all files...")
            .with_style(default_spinner_style())
            .in_optional_multi_progress(self.multi_progress);
        collecting_files_progress.enable_steady_tick(Duration::from_millis(200));

        let all_paths = WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|x| x.file_type().is_file())
            .map(|entry| {
                let path =
                    if self.rename_images && entry.path().extension() == Some(OsStr::new("dds")) {
                        Cow::Owned(entry.path().with_extension("img"))
                    } else {
                        Cow::Borrowed(entry.path())
                    };
                SlicedPath::new(path.strip_prefix(source_dir).unwrap()).unwrap()
            })
            .collect::<Vec<_>>();

        write!(target_file, "MDB1")?;
        target_file.write_u32::<LittleEndian>(all_paths.len() as u32 + 1)?;
        target_file.write_u32::<LittleEndian>(all_paths.len() as u32 + 1)?;
        target_file.write_u32::<LittleEndian>(all_paths.len() as u32)?;

        let data_start_offset = all_paths.len() * (40 + 0x80) + 48 + 0x80;
        // This is the data start offset and the total file size, but we don't know that yet
        target_file.write_u64::<LittleEndian>(data_start_offset as u64)?;
        target_file.write_u64::<LittleEndian>(0)?;

        let tree = generate_tree(&all_paths);

        target_file.write_u64::<LittleEndian>(u64::MAX)?;
        target_file.write_u32::<LittleEndian>(0)?;
        target_file.write_u32::<LittleEndian>(1)?;

        let def_slice = SlicedPath::default();

        let mut header_1s = vec![(Header1::default(), &def_slice); all_paths.len()];

        for (i, path) in all_paths.iter().enumerate() {
            let position = tree[1..].iter().position(|x| path == x.name).unwrap();
            let entry = &tree[1..][position];

            header_1s[position] = (
                Header1 {
                    id: i as u32,
                    left: entry.left as u32,
                    right: entry.right as u32,
                    compare_bit: entry.compare_bit as u32,
                },
                path,
            );
        }

        for (entry, _) in &header_1s {
            target_file.write_u32::<LittleEndian>(entry.compare_bit)?;
            target_file.write_u32::<LittleEndian>(entry.id)?;
            target_file.write_u32::<LittleEndian>(entry.left)?;
            target_file.write_u32::<LittleEndian>(entry.right)?;
        }

        const EMPTY_BUFFER: [u8; 0x80] = [0; 0x80];

        target_file.write_all(&EMPTY_BUFFER)?;

        for &(_, entry) in &header_1s {
            target_file.write_all(&entry.extension)?;
            target_file.write_all(entry.file.replace('/', "\\").as_bytes())?;
            target_file
                .write_all(&EMPTY_BUFFER[..0x80 - entry.extension.len() - entry.file.len()])?;
        }

        for _ in tree[1..].iter() {
            target_file.write_u64::<LittleEndian>(0)?;
            target_file.write_u64::<LittleEndian>(0)?;
            target_file.write_u64::<LittleEndian>(0)?;
        }

        struct FileEntry {
            offset: u64,
            uncompressed_size: u64,
            compressed_size: u64,
        }

        let mut offset = 0;
        let mut entries = Vec::new();

        header_1s.sort_unstable_by_key(|(x, _)| x.id);

        collecting_files_progress.finish_with_message("finished collecting all files!");

        let compression_progress = ProgressBar::new(header_1s.len() as u64)
            .with_style(default_bar_style_with_message_header("compressing file"))
            .with_finish(ProgressFinish::WithMessage(Cow::Borrowed(
                "finished compressing all files",
            )))
            .in_optional_multi_progress(self.multi_progress);

        for (_, entry) in header_1s
            .into_iter()
            .progress_with(compression_progress.clone())
        {
            compression_progress.set_message(Cow::Owned(entry.to_string()));
            let file_content = if self.rename_images && entry.extension == *b"img " {
                fs::read(format!("{}/{}.dds", source_dir.display(), entry.file))?
            } else {
                fs::read(format!("{}/{}", source_dir.display(), entry))?
            };
            let compressed = lz4::block::compress(
                &file_content,
                //None,
                Some(CompressionMode::HIGHCOMPRESSION(12)),
                false,
            )?;
            entries.push(FileEntry {
                offset,
                uncompressed_size: file_content.len() as u64,
                compressed_size: compressed.len() as u64,
            });
            offset += compressed.len() as u64;
            target_file.write_all(&compressed)?;
        }

        target_file.seek(SeekFrom::Start(0x18))?;
        target_file.write_u64::<LittleEndian>(data_start_offset as u64 + offset)?;

        target_file.seek(SeekFrom::Start(
            data_start_offset as u64 - all_paths.len() as u64 * 24,
        ))?;
        for entry in entries {
            target_file.write_u64::<LittleEndian>(entry.offset)?;
            target_file.write_u64::<LittleEndian>(entry.uncompressed_size)?;
            target_file.write_u64::<LittleEndian>(entry.compressed_size)?;
        }

        Ok(())
    }
}
