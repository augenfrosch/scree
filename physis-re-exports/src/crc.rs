// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

/// CRC used for filepath hashes in index file
pub(crate) struct Jamcrc {
    table: [u32; 256],
}

impl Jamcrc {
    pub(crate) const fn new() -> Self {
        let mut table: [u32; 256] = [0u32; 256];

        let polynomial: u32 = 0xEDB88320;
        let mut i = 0;
        while i < table.len() {
            let mut c: u32 = i as u32;
            let mut j = 0;
            while j < 8 {
                if (c & 1u32) == 1u32 {
                    c = polynomial ^ (c >> 1);
                } else {
                    c >>= 1;
                }
                j += 1;
            }

            table[i] = c;
            i += 1;
        }

        Self { table }
    }

    pub(crate) fn checksum(&self, bytes: &[u8]) -> u32 {
        let mut c: u32 = 0xFFFFFFFF;
        for byte in bytes {
            c = self.table[((c ^ *byte as u32) & 0xFF) as usize] ^ (c >> 8);
        }

        !(c ^ 0xFFFFFFFF)
    }
}
