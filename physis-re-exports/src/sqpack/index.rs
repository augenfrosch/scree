// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
	io::{Read, Seek, SeekFrom, Write},
	path::Path,
};

use binrw::{BinRead, BinResult, BinWrite, Endian, Error, binrw};

use crate::{common::Platform, crc::checksum, sqpack::SqPackHeader};

#[binrw]
#[derive(Debug, Clone)]
pub struct SegementDescriptor {
	count: u32,
	offset: u32,
	size: u32,
	#[brw(pad_after = 40)]
	sha1_hash: [u8; 20],
}

#[binrw]
#[brw(repr = u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexType {
	/// `.index` files.
	Index1 = 0,
	/// `.index2` files.
	Index2 = 2,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct SqPackIndexHeader {
	/// The size of this header in bytes.
	size: u32,

	#[brw(pad_after = 4)]
	file_descriptor: SegementDescriptor,

	// Count in this descriptor correlates to the number of dat files.
	data_descriptor: SegementDescriptor,

	unknown_descriptor: SegementDescriptor,

	#[brw(pad_after = 4)]
	folder_descriptor: SegementDescriptor,

	pub(crate) index_type: IndexType,

	#[brw(pad_before = 652)]
	#[brw(pad_after = 44)]
	// The SHA1 of the bytes immediately before this
	sha1_hash: [u8; 20],
}

#[derive(PartialEq, Debug, Clone, Copy)]
#[repr(C)]
pub enum Hash {
	// TODO: on the PS3 these are flipped, but I don't have a good way to represent that yet...
	// MAYBE: resolved now with the new changes. I don't have any files to test
	SplitPath { name: u32, path: u32 },
	FullPath(u32),
}

impl BinRead for Hash {
	type Args<'a> = (&'a IndexType,);

	fn read_options<R: Read + Seek>(
		reader: &mut R,
		endian: Endian,
		args: Self::Args<'_>,
	) -> BinResult<Self> {
		let crc1 = <u32>::read_options(reader, endian, ())?;
		match args.0 {
			IndexType::Index1 => {
				let crc2 = <u32>::read_options(reader, endian, ())?;
				// CRCs are flipped on the PS3
				match endian {
					Endian::Big => Ok(Self::SplitPath {
						name: crc2,
						path: crc1,
					}),
					Endian::Little => Ok(Self::SplitPath {
						name: crc1,
						path: crc2,
					}),
				}
			},
			IndexType::Index2 => Ok(Self::FullPath(crc1)),
		}
	}
}

impl BinWrite for Hash {
	type Args<'a> = ();

	fn write_options<W: Write + Seek>(
		&self,
		writer: &mut W,
		endian: Endian,
		(): Self::Args<'_>,
	) -> BinResult<()> {
		match self {
			// CRCs are flipped on the PS3
			Hash::SplitPath { name, path } => match endian {
				Endian::Big => {
					path.write_options(writer, endian, ())?;
					name.write_options(writer, endian, ())
				},
				Endian::Little => {
					name.write_options(writer, endian, ())?;
					path.write_options(writer, endian, ())
				},
			},
			Hash::FullPath(crc) => crc.write_options(writer, endian, ()),
		}
	}
}

#[derive(Debug, Clone)]
pub struct FileEntryData {
	pub is_synonym: bool,
	pub data_file_id: u8,
	pub offset: u64,
}

impl BinRead for FileEntryData {
	type Args<'a> = ();

	fn read_options<R: Read + Seek>(
		reader: &mut R,
		endian: Endian,
		(): Self::Args<'_>,
	) -> BinResult<Self> {
		let mut data = <u32>::read_options(reader, endian, ())?;
		if endian == Endian::Big {
			// Taken from Lumina, fixes reading from PS3 somehow?
			data = (data << 4) | ((data & 0x70000000) >> 27) | ((data & 0x80000000) >> 31);
		}

		Ok(Self {
			is_synonym: (data & 0b1) == 0b1,
			data_file_id: ((data & 0b1110) >> 1) as u8,
			offset: (data & !0xF) as u64 * 0x08,
		})
	}
}

impl BinWrite for FileEntryData {
	type Args<'a> = ();

	fn write_options<W: Write + Seek>(
		&self,
		writer: &mut W,
		endian: Endian,
		(): Self::Args<'_>,
	) -> Result<(), Error> {
		// TODO: support synonym and data_file_id
		let data: u32 = self.offset.wrapping_div(0x08) as u32;

		// TODO: support big endian?

		data.write_options(writer, endian, ())
	}
}

#[binrw]
#[brw(import(index_type: &IndexType))]
#[derive(Debug, Clone)]
pub struct FileEntry {
	#[br(args(index_type))]
	pub hash: Hash,

	pub data: FileEntryData,

	#[br(temp)]
	#[bw(calc = 0)]
	#[br(if(*index_type == IndexType::Index1))]
	padding: u32,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct DataEntry {
	// A bunch of 0xFFFFFFFF
	unk: [u8; 256],
}

#[binrw]
#[derive(Debug, Clone)]
pub struct FolderEntry {
	// Hash of the full folder/directory path. Matches the `Hash::SplitPath` variant's `path` field
	pub hash: u32,
	// Offset into the index file to a `total_files_size` bytes long section containing `FileEntry`s that are in the  folder
	pub files_offset: u32,
	// Divide by 0x10 (the size of a `FileEntry` for `IndexType::Index1`, I think) to get the number of files
	#[brw(pad_after = 4)]
	pub total_files_size: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct IndexEntry {
	pub data_file_id: u8,
	pub offset: u64,
}

impl From<&FileEntry> for IndexEntry {
	fn from(entry: &FileEntry) -> Self {
		let FileEntryData {
			is_synonym: _,
			data_file_id,
			offset,
		} = entry.data;
		Self {
			data_file_id,
			offset,
		}
	}
}

#[derive(Debug)]
pub struct FolderEntryInfo {
	pub files_starting_index: usize,
	pub file_count: usize,
}

impl FolderEntryInfo {
	pub fn new(index: &SqPackIndex, folder_entry: &FolderEntry) -> Self {
		const FILE_ENTRY_SIZE: usize = 0x10;
		let files_starting_index = usize::try_from(
			folder_entry
				.files_offset
				.checked_sub(index.index_header.file_descriptor.offset)
				.expect(&format!(
					"malformed index: base offset ({offset}) is greater than folder's files offset ({files_offset})",
					offset = index.index_header.file_descriptor.offset,
					files_offset = folder_entry.files_offset
				)),
		)
		.expect("16-bit D:")
			/ FILE_ENTRY_SIZE;
		Self {
			files_starting_index,
			file_count: usize::try_from(folder_entry.total_files_size).expect("16-bit :(")
				/ FILE_ENTRY_SIZE,
		}
	}
}

#[binrw]
#[derive(Debug, Clone)]
pub struct SqPackIndex {
	sqpack_header: SqPackHeader,

	#[br(seek_before = SeekFrom::Start(sqpack_header.size.into()))]
	index_header: SqPackIndexHeader,

	#[br(seek_before = SeekFrom::Start(index_header.file_descriptor.offset.into()), count = index_header.file_descriptor.size / 16, args { inner: (&index_header.index_type,) })]
	#[bw(args(&index_header.index_type,))]
	pub entries: Vec<FileEntry>,

	#[br(seek_before = SeekFrom::Start(index_header.data_descriptor.offset.into()))]
	#[br(count = index_header.data_descriptor.size / 256)]
	pub data_entries: Vec<DataEntry>,

	/*#[br(seek_before = SeekFrom::Start(index_header.unknown_descriptor.offset.into()))]
	 *    #[br(count = index_header.unknown_descriptor.size / 16)]
	 *    pub unknown_entries: Vec<IndexHashTableEntry>,*/
	#[br(seek_before = SeekFrom::Start(index_header.folder_descriptor.offset.into()))]
	#[br(count = index_header.folder_descriptor.size / 16)]
	pub folder_entries: Vec<FolderEntry>,
}

impl SqPackIndex {
	/// Creates a new reference to an existing index file.
	pub fn from_existing(platform: Platform, path: &Path) -> Option<Self> {
		// Index files are individually small, so we can easily load them entirely to memory.
		// Our current index-reading code uses seeking, and that's *very* slow when reading from a disk.
		let buf = std::fs::read(path).ok()?;
		Self::read_options(&mut std::io::Cursor::new(buf), platform.endianness(), ()).ok()
	}

	/// Calculates a partial hash for a given path
	pub fn calculate_partial_hash(path: &str) -> u32 {
		let lowercase = path.to_lowercase();

		checksum(lowercase.as_bytes())
	}

	/// Calculates a hash for `index` files from a game path.
	pub fn calculate_hash(index_type: IndexType, path: &str) -> Hash {
		let lowercase = path.to_lowercase();

		match index_type {
			IndexType::Index1 => {
				if let Some(pos) = lowercase.rfind('/') {
					let (directory, filename) = lowercase.split_at(pos);

					let path = checksum(directory.as_bytes());
					let name = checksum(&filename.as_bytes()[1..]);

					Hash::SplitPath { name, path }
				} else {
					// TODO: is this ever hit?
					panic!("This is unexpected, why is the file sitting outside of a folder?");
				}
			},
			IndexType::Index2 => Hash::FullPath(checksum(lowercase.as_bytes())),
		}
	}

	pub fn exists(&self, path: &str) -> bool {
		let hash = Self::calculate_hash(self.index_header.index_type, path);
		self.entries.iter().any(|s| s.hash == hash)
	}

	pub fn find_entry(&self, path: &str) -> Option<IndexEntry> {
		let hash = Self::calculate_hash(self.index_header.index_type, path);
		self.find_entry_from_hash(hash)
	}

	pub fn find_folder_entry(&self, path: &str) -> Option<FolderEntryInfo> {
		// Stripping a terminating '/' might not be "correct", but it does make the function more fogiving.
		// MAYBE: remove this, if performace becomes a concern.
		let path = if path.ends_with('/') {
			&path[..path.len() - 1]
		} else {
			path
		};
		let hash = Self::calculate_partial_hash(path);
		self.find_folder_entry_info_from_hash(hash)
	}

	pub fn find_entry_from_hash(&self, hash: Hash) -> Option<IndexEntry> {
		self.entries
			.iter()
			.find(|s| s.hash == hash)
			.map(IndexEntry::from)
	}

	pub fn find_folder_entry_info_from_hash(&self, hash: u32) -> Option<FolderEntryInfo> {
		self.folder_entries
			.iter()
			.find(|s| s.hash == hash)
			.map(|entry| FolderEntryInfo::new(self, entry))
	}

	pub fn find_entry_from_offset(&self, offset: u64) -> Option<Hash> {
		if let Some(entry) = self.entries.iter().find(|s| s.data.offset == offset) {
			return Some(entry.hash);
		}

		None
	}
}
