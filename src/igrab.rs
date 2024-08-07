/*
 * idGrab: A header generator for ID-engine (Keen: Galaxy) games.
 *
 * Copyright (C) 2024 David Gow <david@davidgow.net>
 *
 * This software is provided 'as-is', without any express or implied warranty.
 * In no event will the authors be held liable for any damages arising from
 * the use of this software.
 *
 * Permission is granted to anyone to use this software for any purpose, including
 * commercial applications, and to alter it and redistribute it freely, subject
 * to the following restrictions.
 *   1. The origin of this software must not be misrepresented; you must not
 *      claim that you wrote the original software. If you use this software in
 *      a product, an acknowledgment in the product documentation would be
 *      appreciated but is not required.
 *   2. Altered source versions must be plainly marked as such, and must not be
 *      misrepresented as being the original software.
 *   3. This notice may not be removed or altered from any source distribution.
 */

// The tab width used in outputting IGRAB files. Mostly used by 0.24
const IGRAB_TAB_WIDTH: usize = 8;

#[derive(PartialEq, Clone, Copy)]
pub enum IGrabVersion {
	ZeroPointTwoFour,
	ZeroPointFour,
}

impl Default for IGrabVersion {
	fn default() -> IGrabVersion { IGrabVersion::ZeroPointFour }
}

impl std::fmt::Display for IGrabVersion {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			IGrabVersion::ZeroPointTwoFour => write!(f, "0.24"),
			IGrabVersion::ZeroPointFour => write!(f, "0.4"),
		}
	}
}

#[derive(Default)]
pub struct IGrabOptions {
	pub version: IGrabVersion,
	pub append_underscores: bool,
}

impl IGrabOptions {
	pub fn write_chunk_line(
		&self,
		f: &mut dyn std::io::Write,
		chunk_name: &str,
		chunk_suffix: Option<&str>,
		chunk_num: u32,
		first: bool,
	) -> std::io::Result<()> {
		match self.version {
			IGrabVersion::ZeroPointTwoFour => {
				let num_chars = 8
					+ chunk_name.len() + if self.append_underscores
					&& chunk_suffix != None
				{
					1
				} else {
					0
				} + if chunk_suffix != None {
					chunk_suffix.unwrap().len()
				} else {
					0
				};
				let desired_column = 41; /* "#define ".len() */
				let num_tabs = (desired_column - num_chars) / IGRAB_TAB_WIDTH;
				write!(f, "#define {}", chunk_name)?;
				if self.append_underscores && chunk_suffix != None {
					write!(f, "_")?;
				}
				if chunk_suffix != None {
					write!(f, "{}", chunk_suffix.unwrap())?;
				}
				for _ in 0..num_tabs {
					write!(f, "\t")?;
				}
				writeln!(f, "{}", chunk_num)
			}
			IGrabVersion::ZeroPointFour => {
				if first {
					writeln!(
						f,
						"\t\t{}{}{} = {},",
						chunk_name,
						if self.append_underscores && chunk_suffix != None {
							"_"
						} else {
							""
						},
						chunk_suffix.unwrap_or(""),
						chunk_num
					)
				} else {
					let num_chars = chunk_name.len()
						+ if self.append_underscores && chunk_suffix != None
						{
							1
						} else {
							0
						} + if chunk_suffix != None {
						chunk_suffix.unwrap().len()
					} else {
						0
					} + 1; // ','
					let desired_column = 32 + 5; /* NAMELEN + 5 */
					let num_spaces = desired_column - num_chars;
					write!(
						f,
						"\t\t{}{}{},",
						chunk_name,
						if self.append_underscores && chunk_suffix != None {
							"_"
						} else {
							""
						},
						chunk_suffix.unwrap_or("")
					)?;
					for _ in 0..num_spaces {
						write!(f, " ")?;
					}
					writeln!(f, "// {}", chunk_num)
				}
			}
		}
	}

	pub fn write_asm_chunk_line(
		&self,
		f: &mut dyn std::io::Write,
		chunk_name: &str,
		chunk_suffix: Option<&str>,
		chunk_num: u32,
	) -> std::io::Result<()> {
		let num_chars = chunk_name.len()
			+ if chunk_suffix != None {
				chunk_suffix.unwrap().len()
					+ if self.append_underscores { 1 } else { 0 }
			} else {
				0
			};
		let desired_column = 33;
		let num_tabs = (desired_column - num_chars + IGRAB_TAB_WIDTH - 2) / IGRAB_TAB_WIDTH;
		write!(
			f,
			"{}{}{}",
			chunk_name,
			if self.append_underscores && chunk_suffix != None {
				"_"
			} else {
				""
			},
			chunk_suffix.unwrap_or("")
		)?;
		for _ in 0..num_tabs {
			write!(f, "\t")?;
		}
		writeln!(f, "\t=\t{}", chunk_num)
	}
}
