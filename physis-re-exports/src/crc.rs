// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

/// CRC used for filepath hashes in index file

const TABLE: [u32; 256] = {
	let mut table: [u32; 256] = [0u32; 256];

	let polynomial: u32 = 0xEDB88320;
	let mut i = 0;
	while i < table.len() {
		let mut c = i as u32;
		let mut j = 0;
		while j < 8 {
			if (c & 1) == 1 {
				c = polynomial ^ (c >> 1);
			} else {
				c >>= 1;
			}
			j += 1;
		}

		table[i] = c;
		i += 1;
	}

	table
};

pub(crate) fn checksum(bytes: &[u8]) -> u32 {
	let mut c: u32 = 0xFFFFFFFF;
	for byte in bytes {
		c = TABLE[((c ^ *byte as u32) & 0xFF) as usize] ^ (c >> 8);
	}

	c // I have no clue why this originally had a double inversion
}
