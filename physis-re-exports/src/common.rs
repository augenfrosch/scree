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
    /// Returns the short-hand codename for this platform.
    ///
    /// For example, `Platform::Win32` becomes "win32".
    pub fn shortname(&self) -> &'static str {
        match self {
            Platform::Win32 => "win32",
            Platform::PS3 => "ps3",
            Platform::PS4 => "ps4",
            Platform::PS5 => "ps5",
            Platform::Xbox => "lys",
        }
    }

    /// Returns the endianness for this platform.
    pub(crate) fn endianness(&self) -> Endian {
        match self {
            Platform::PS3 => Endian::Big,
            _ => Endian::Little,
        }
    }
}
