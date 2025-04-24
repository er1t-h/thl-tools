use std::{
    borrow::Cow,
    fs::{self, File},
    io::{BufWriter, Seek, SeekFrom, Write},
    ops::Index,
    time::{Duration, Instant},
};

use byteorder::{LittleEndian, WriteBytesExt};
use glob::glob;
use indicatif::{ProgressBar, ProgressFinish, ProgressIterator, ProgressStyle};
use lz4::block::CompressionMode;

#[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord, Clone)]
struct SlicedPath {
    extension: [u8; 4],
    file: String,
}
impl Index<u16> for SlicedPath {
    type Output = u8;
    fn index(&self, index: u16) -> &Self::Output {
        self.extension
            .iter()
            .chain(self.file.as_bytes().iter())
            .chain(std::iter::once(&0))
            .nth(index as usize)
            .unwrap()
    }
}

const EMPTY_SLICED_PATH: &SlicedPath = &SlicedPath {
    file: String::new(),
    extension: [b' '; 4],
};

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

    let progress = ProgressBar::new_spinner().with_style(
        ProgressStyle::default_spinner()
            .template("[{elapsed_precise}] {spinner} {msg}")
            .unwrap(),
    );
    progress.enable_steady_tick(Duration::from_millis(500));

    let mut i = 0;
    while let Some(entry) = queue.pop() {
        let intro = format!("building tree, iteration {i}:");
        progress.set_message(format!("{intro} separating nodeless and with_nodes"));
        i += 1;

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

        progress.set_message(format!("{intro} finding child"));

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

        progress.set_message(format!("{intro} differentiating left and right childs"));

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
    progress.finish();

    nodes
}

pub fn pack(source_dir: &str, target_file: &str) -> std::io::Result<()> {
    let mut file = BufWriter::new(File::create_new(target_file)?);
    let bar_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {wide_bar} {pos:>7}/{len:7} {msg}")
        .unwrap();

    let progress = ProgressBar::new_spinner()
        .with_elapsed(Duration::from_secs(0))
        .with_message("collecting all files...")
        .with_style(
            ProgressStyle::default_spinner()
                .template("[{elapsed_precise}] {spinner} {msg}")
                .unwrap(),
        );
    progress.enable_steady_tick(Duration::from_millis(300));

    let mut all_paths = glob(&format!("{source_dir}/**/*"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|x| x.is_file())
        .map(|path| {
            let path = path.strip_prefix(source_dir).unwrap();
            let extension = path.extension().unwrap().to_str().unwrap();
            let path = path.with_extension("").to_str().unwrap().to_string();
            SlicedPath {
                file: path,
                extension: std::array::from_fn(|i| {
                    if let Some(&x) = extension.as_bytes().get(i) {
                        x
                    } else {
                        b' '
                    }
                }),
            }
        })
        .collect::<Vec<_>>();
    for path in &all_paths {
        println!(
            "{:4}{}",
            path.extension
                .iter()
                .copied()
                .map(|x| x as char)
                .collect::<String>(),
            path.file
        );
    }

    progress.finish_with_message("finished collecting all files!");

    write!(file, "MDB1")?;
    file.write_u32::<LittleEndian>(all_paths.len() as u32 + 1)?;
    file.write_u32::<LittleEndian>(all_paths.len() as u32 + 1)?;
    file.write_u32::<LittleEndian>(all_paths.len() as u32)?;

    let data_start_offset = all_paths.len() * (40 + 0x80) + 48 + 0x80;
    // This is the data start offset and the total file size, but we don't know that yet
    file.write_u64::<LittleEndian>(data_start_offset as u64)?;
    file.write_u64::<LittleEndian>(0)?;

    let tree = generate_tree(&all_paths);

    file.write_u64::<LittleEndian>(u64::MAX)?;
    file.write_u32::<LittleEndian>(0)?;
    file.write_u32::<LittleEndian>(1)?;

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
        file.write_u32::<LittleEndian>(entry.compare_bit)?;
        file.write_u32::<LittleEndian>(entry.id)?;
        file.write_u32::<LittleEndian>(entry.left)?;
        file.write_u32::<LittleEndian>(entry.right)?;
    }

    const EMPTY_BUFFER: [u8; 0x80] = [0; 0x80];

    file.write_all(&EMPTY_BUFFER)?;

    for &(_, entry) in header_1s
        .iter()
        .progress()
        .with_style(bar_style.clone())
        .with_message("writing file names")
        .with_finish(ProgressFinish::WithMessage(Cow::Borrowed(
            "finished writing all names",
        )))
    {
        file.write_all(&entry.extension)?;
        file.write_all(entry.file.replace('/', "\\").as_bytes())?;
        file.write_all(&EMPTY_BUFFER[..0x80 - entry.extension.len() - entry.file.len()])?;
    }

    for _ in tree[1..].iter() {
        file.write_u64::<LittleEndian>(0)?;
        file.write_u64::<LittleEndian>(0)?;
        file.write_u64::<LittleEndian>(0)?;
    }

    struct FileEntry {
        offset: u64,
        uncompressed_size: u64,
        compressed_size: u64,
    }

    let mut offset = 0;
    let mut entries = Vec::new();

    header_1s.sort_unstable_by_key(|(x, _)| x.id);

    for (_, entry) in header_1s
        .into_iter()
        .progress_with_style(bar_style)
        .with_message("compressing and writing files to archive")
        .with_finish(ProgressFinish::AndLeave)
    {
        let ext = entry
            .extension
            .into_iter()
            .map(|x| x as char)
            .take_while(|&x| x != ' ')
            .collect::<String>();
        let file_content = fs::read(format!("{}/{}.{}", source_dir, entry.file, ext))?;
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
        file.write_all(&compressed)?;
    }

    file.seek(SeekFrom::Start(0x18))?;
    file.write_u64::<LittleEndian>(data_start_offset as u64 + offset)?;

    file.seek(SeekFrom::Start(
        data_start_offset as u64 - all_paths.len() as u64 * 24,
    ))?;
    for entry in entries {
        file.write_u64::<LittleEndian>(entry.offset)?;
        file.write_u64::<LittleEndian>(entry.uncompressed_size)?;
        file.write_u64::<LittleEndian>(entry.compressed_size)?;
    }

    Ok(())
}
