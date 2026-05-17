// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later
// Modified for `scree` by augenfrosch,
// starting 2026-05-16, from commit d9ab40fc2405bae3822986bd57691b7702c3b80f

use binrw::binrw;

use crate::common::Platform;

mod index;
pub use index::{FolderEntryInfo, Hash, IndexEntry, IndexType, SqPackIndex};

/// The type of this `SqPack` file.
#[binrw]
#[brw(repr = u32)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum SqPackFileType {
	/// FFXIV Explorer says "SQDB", whatever that is.
	Sqdb = 0x0,
	/// Dat files.
	Data = 0x1,
	/// Index/Index2 files.
	Index = 0x2,
}

#[binrw]
#[brw(magic = b"SqPack\0\0")]
#[derive(Debug, Clone)]
pub(crate) struct SqPackHeader {
	#[brw(pad_size_to = 4)]
	platform: Platform,
	/// The size of this header in bytes.
	pub size: u32,
	/// Have only seen version 1.
	version: u32,
	file_type: SqPackFileType,

	// TODO: some unknown value, zeroed out for index files
	// XivAlexandar says date/time, where does that come from?
	unk1: u32,
	unk2: u32,

	// TODO: this is possibly region, but CN users reported this as 0 so maybe not?
	region: u8,

	#[brw(pad_before = 924)]
	#[brw(pad_after = 44)]
	/// The SHA1 of the bytes immediately before this.
	sha1_hash: [u8; 20],
}
