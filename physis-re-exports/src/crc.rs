// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later
// Modified for `scree` by augenfrosch,
// starting 2026-05-16, from commit d9ab40fc2405bae3822986bd57691b7702c3b80f

// CRC used for filepath hashes in index file

const TABLE_LEN_U32: u32 = 256;

const TABLE: [u32; 256] = {
	let mut table: [u32; 256] = [0u32; 256];

	let polynomial: u32 = 0xEDB8_8320;
	let mut i = 0;
	while i < TABLE_LEN_U32 {
		let mut c = i;
		let mut j = 0;
		while j < 8 {
			if (c & 1) == 1 {
				c = polynomial ^ (c >> 1);
			} else {
				c >>= 1;
			}
			j += 1;
		}

		table[i as usize] = c;
		i += 1;
	}

	table
};

pub(crate) fn checksum(bytes: &[u8]) -> u32 {
	let mut c: u32 = 0xFFFF_FFFF;
	for byte in bytes {
		c = TABLE[((c ^ u32::from(*byte)) & 0xFF) as usize] ^ (c >> 8);
	}

	c // I have no clue why this originally had a double inversion
}
