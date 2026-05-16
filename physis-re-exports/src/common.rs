// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::{Endian, binrw};
/// Platform used for game data.
#[binrw]
#[brw(repr = u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Platform {
	/// Windows and macOS.
	Win32 = 0x0,
	/// Playstation 3.
	PS3 = 0x1,
	/// Playstation 4.
	PS4 = 0x2,
	/// Playstation 5.
	PS5 = 0x3,
	/// Xbox One.
	Xbox = 0x4,
}

impl Platform {
	/// Returns the endianness for this platform.
	pub(crate) fn endianness(&self) -> Endian {
		match self {
			Platform::PS3 => Endian::Big,
			_ => Endian::Little,
		}
	}
}
