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
#![allow(dead_code)]

pub use std::{convert::TryInto, fs::File, io::Write, path::Path};

mod binary_io;
use binary_io::*;
mod igrab;
mod parser;
use igrab::*;

#[derive(Debug)]
struct Lump {
	name: String,
	start_chunk: u32,
	end_chunk: u32,
}

#[derive(Debug)]
enum MiscChunk {
	Chunk(String),
	Article(String),
	B8000Text(String),
	Terminator(String),
	Demo(u32),
}

#[derive(Default, Debug)]
struct GfxHeaders {
	extension: Option<String>,
	header_chunk_count: u32,
	fonts: Vec<String>,
	fonts_masked: Vec<String>,
	bitmaps: Vec<String>,
	bitmaps_masked: Vec<String>,
	sprites: Vec<String>,
	tile8_count: u32,
	tile8_masked_count: u32,
	tile16_count: u32,
	tile16_masked_count: u32,
	tile32_count: u32,
	tile32_masked_count: u32,
	misc_chunks: Vec<MiscChunk>,
	lumps: Vec<Lump>,
}

impl GfxHeaders {
	fn num_chunks(&self) -> u32 {
		self.header_chunk_count
			+ self.fonts.len() as u32
			+ self.fonts_masked.len() as u32
			+ self.bitmaps.len() as u32
			+ self.bitmaps_masked.len() as u32
			+ self.sprites.len() as u32
			+ if self.tile8_count != 0 { 1 } else { 0 }
			+ if self.tile8_masked_count != 0 { 1 } else { 0 }
			+ self.tile16_count + self.tile16_masked_count
			+ self.tile32_count + self.tile32_masked_count
			+ self.misc_chunks.len() as u32
	}

	fn fonts_start(&self) -> u32 {
		self.header_chunk_count
	}

	fn fonts_masked_start(&self) -> u32 {
		self.fonts_start() + self.fonts.len() as u32
	}

	fn bitmaps_start(&self) -> u32 {
		self.fonts_masked_start() + self.fonts_masked.len() as u32
	}

	fn bitmaps_masked_start(&self) -> u32 {
		self.bitmaps_start() + self.bitmaps.len() as u32
	}

	fn sprites_start(&self) -> u32 {
		self.bitmaps_masked_start() + self.bitmaps_masked.len() as u32
	}

	fn tile8_start(&self) -> u32 {
		self.sprites_start() + self.sprites.len() as u32
	}

	fn tile8_masked_start(&self) -> u32 {
		self.tile8_start() + if self.tile8_count != 0 { 1 } else { 0 }
	}

	fn tile16_start(&self) -> u32 {
		self.tile8_masked_start() + if self.tile8_masked_count != 0 { 1 } else { 0 }
	}

	fn tile16_masked_start(&self) -> u32 {
		self.tile16_start() + self.tile16_count
	}

	fn tile32_start(&self) -> u32 {
		self.tile16_masked_start() + self.tile16_masked_count
	}

	fn tile32_masked_start(&self) -> u32 {
		self.tile32_start() + self.tile32_count
	}

	fn misc_start(&self) -> u32 {
		self.tile32_masked_start() + self.tile32_masked_count
	}

	fn chunk_name(&self, chunk: u32) -> Option<String> {
		if chunk < self.fonts_start() {
			None
		} else if chunk < self.fonts_masked_start() {
			Some(self.fonts[(chunk - self.fonts_start()) as usize].clone())
		} else if chunk < self.bitmaps_start() {
			Some(
				self.fonts_masked[(chunk - self.fonts_masked_start()) as usize]
					.clone(),
			)
		} else if chunk < self.bitmaps_masked_start() {
			Some(self.bitmaps[(chunk - self.bitmaps_start()) as usize].clone())
		} else if chunk < self.sprites_start() {
			Some(
				self.bitmaps_masked[(chunk - self.bitmaps_masked_start()) as usize]
					.clone(),
			)
		} else if chunk < self.tile8_start() {
			Some(self.sprites[(chunk - self.sprites_start()) as usize].clone())
		} else {
			None
		}
	}

	fn omnispeak_chunk_name(&self, chunk: u32) -> Option<String> {
		if chunk < self.fonts_start() {
			None
		} else if chunk < self.fonts_masked_start() {
			Some(format!(
				"FON_{}",
				self.fonts[(chunk - self.fonts_start()) as usize]
			))
		} else if chunk < self.bitmaps_start() {
			Some(format!(
				"FONM_{}",
				self.fonts_masked[(chunk - self.fonts_masked_start()) as usize]
			))
		} else if chunk < self.bitmaps_masked_start() {
			Some(format!(
				"PIC_{}",
				self.bitmaps[(chunk - self.bitmaps_start()) as usize]
			))
		} else if chunk < self.sprites_start() {
			Some(format!(
				"PICM_{}",
				self.bitmaps_masked[(chunk - self.bitmaps_masked_start()) as usize]
			))
		} else if chunk < self.tile8_start() {
			Some(format!(
				"SPR_{}",
				self.sprites[(chunk - self.sprites_start()) as usize]
			))
		} else {
			None
		}
	}

	fn write_gfxinfoe(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
		// Tile counts
		write_le16(self.tile8_count as u16, writer)?;
		write_le16(self.tile8_masked_count as u16, writer)?;
		write_le16(self.tile16_count as u16, writer)?;
		write_le16(self.tile16_masked_count as u16, writer)?;
		write_le16(self.tile32_count as u16, writer)?;
		write_le16(self.tile32_masked_count as u16, writer)?;
		// Tile Starts
		write_le16(self.tile8_start() as u16, writer)?;
		write_le16(self.tile8_masked_start() as u16, writer)?;
		write_le16(self.tile16_start() as u16, writer)?;
		write_le16(self.tile16_masked_start() as u16, writer)?;
		write_le16(self.tile32_start() as u16, writer)?;
		write_le16(self.tile32_masked_start() as u16, writer)?;
		// Other Counts
		write_le16(self.bitmaps.len() as u16, writer)?;
		write_le16(self.bitmaps_masked.len() as u16, writer)?;
		write_le16(self.sprites.len() as u16, writer)?;
		// Other Starts
		write_le16(self.bitmaps_start() as u16, writer)?;
		write_le16(self.bitmaps_masked_start() as u16, writer)?;
		write_le16(self.sprites_start() as u16, writer)?;
		// Header chunks
		write_le16(0, writer)?;
		write_le16(1, writer)?;
		write_le16(2, writer)?;
		// Miscs
		write_le16(self.misc_chunks.len() as u16, writer)?;
		write_le16(self.misc_start() as u16, writer)?;
		Ok(())
	}

	fn save_gfxinfoe(&self, filename: &str) -> std::io::Result<()> {
		let gfxinfoe_file = std::fs::File::create(filename)?;
		let mut gfxinfoe_writer = std::io::BufWriter::new(gfxinfoe_file);
		self.write_gfxinfoe(&mut gfxinfoe_writer)
	}

	fn write_modid_script(&self, script: &mut dyn std::io::Write) -> std::io::Result<()> {
		writeln!(script, "# ModID Script: Automatically Generated")?;
		writeln!(script, "GALAXY")?;
		if let Some(ext) = &self.extension {
			writeln!(script, "\tGAMEEXT {}", ext)?;
		}
		writeln!(script, "\tGRSTARTS 3")?;
		//TODO: CKPATCH options
		//writeln!(script, "\tEXEINFO ajd.exe 0x3F630 0x259B0 0x36F4E 0x2C00")?;
		//writeln!(script, "\tCKPATCHVER 1.6")?;
		writeln!(script, "\tCHUNKS {}", self.num_chunks())?;

		let mut chunk_count = self.header_chunk_count;
		writeln!(script, "\t\tFONT\t\t{} {}", self.fonts.len(), chunk_count)?;
		chunk_count += self.fonts.len() as u32;
		writeln!(
			script,
			"\t\tFONTM\t\t{} {}",
			self.fonts_masked.len(),
			chunk_count
		)?;
		chunk_count += self.fonts_masked.len() as u32;
		writeln!(
			script,
			"\t\tPICS\t\t{} {} 0",
			self.bitmaps.len(),
			chunk_count
		)?;
		chunk_count += self.bitmaps.len() as u32;
		writeln!(
			script,
			"\t\tPICM\t\t{} {} 1",
			self.bitmaps_masked.len(),
			chunk_count
		)?;
		chunk_count += self.bitmaps_masked.len() as u32;
		writeln!(
			script,
			"\t\tSPRITES\t\t{} {} 2",
			self.sprites.len(),
			chunk_count
		)?;
		chunk_count += self.sprites.len() as u32;
		writeln!(script, "\t\tTILE8\t\t{} {}", self.tile8_count, chunk_count)?;
		chunk_count += if self.tile8_count != 0 { 1 } else { 0 }; /* Tile8s are stored in a single chunk. */
		writeln!(
			script,
			"\t\tTILE8M\t\t{} {}",
			self.tile8_masked_count, chunk_count
		)?;
		chunk_count += if self.tile8_masked_count != 0 { 1 } else { 0 }; /* …as are Tile8ms. */
		writeln!(
			script,
			"\t\tTILE16\t\t{} {}",
			self.tile16_count, chunk_count
		)?;
		chunk_count += self.tile16_count;
		writeln!(
			script,
			"\t\tTILE16M\t\t{} {}",
			self.tile16_masked_count, chunk_count
		)?;
		chunk_count += self.tile16_masked_count;
		writeln!(
			script,
			"\t\tTILE32\t\t{} {}",
			self.tile32_count, chunk_count
		)?;
		chunk_count += self.tile32_count;
		writeln!(
			script,
			"\t\tTILE32M\t\t{} {}",
			self.tile32_masked_count, chunk_count
		)?;
		chunk_count += self.tile32_masked_count;

		for chunk in &self.misc_chunks {
			match chunk {
				MiscChunk::Chunk(name) => {
					writeln!(script, "\t\tMISC {} {}", chunk_count, name)?;
				}
				MiscChunk::B8000Text(name) => {
					writeln!(script, "\t\tB800TEXT {} {}", chunk_count, name)?;
				}
				MiscChunk::Article(name) => {
					writeln!(script, "\t\tTEXT {} {}", chunk_count, name)?;
				}
				MiscChunk::Terminator(name) => {
					writeln!(
						script,
						"\t\tTERMINATOR {} {}",
						chunk_count, name
					)?;
				}
				MiscChunk::Demo(num) => {
					writeln!(script, "\t\tDEMO {} {}", chunk_count, num)?;
				}
			}
			chunk_count += 1;
		}
		Ok(())
	}

	fn save_modid_script(&self, filename: &str) -> std::io::Result<()> {
		let modid_file = std::fs::File::create(filename)?;
		let mut modid_writer = std::io::BufWriter::new(modid_file);
		self.write_modid_script(&mut modid_writer)
	}

	#[cfg(feature = "timestamps")]
	fn timestamp() -> String {
		// From man ctime_r: "stores the string in a user-supplied buffer which should have room for at least 26 bytes"
		let mut buf = vec![0u8; 26];

		let time = unsafe { libc::time(std::ptr::null_mut()) };
		unsafe { libc::ctime_r(&time, buf.as_mut_ptr() as *mut std::ffi::c_char) };
		let str_slice = std::ffi::CStr::from_bytes_until_nul(&buf).unwrap();

		str_slice.to_string_lossy().into_owned()
	}

	fn write_igrab_header(
		&self,
		f: &mut dyn std::io::Write,
		igrab_options: &IGrabOptions,
	) -> std::io::Result<()> {
		writeln!(f, "//////////////////////////////////////")?;
		writeln!(f, "//")?;
		if let Some(ext) = &self.extension {
			writeln!(f, "// Graphics .H file for {}", ext)?;
		}
		#[cfg(feature = "timestamps")]
		write!(f, "// idGrab-ed on {}", GfxHeaders::timestamp())?;
		writeln!(f, "// idGrab emulating IGRAB {}", igrab_options.version)?;
		writeln!(f, "//")?;
		writeln!(f, "//////////////////////////////////////\n")?;

		let mut chunk_id = self.bitmaps_start();

		/* If the IGRAB version is 0.24, we use defines. Otherwise, we use an enum. */
		if igrab_options.version == IGrabVersion::ZeroPointFour {
			writeln!(f, "typedef enum {{")?;
		}

		/* Fonts are not included, nor masked fonts. */

		for pic in &self.bitmaps {
			igrab_options.write_chunk_line(
				f,
				pic,
				Some("PIC"),
				chunk_id,
				chunk_id == self.bitmaps_start(),
			)?;
			chunk_id += 1;
		}

		writeln!(f, "")?;

		for picm in &self.bitmaps_masked {
			igrab_options.write_chunk_line(
				f,
				picm,
				Some("PICM"),
				chunk_id,
				chunk_id == self.bitmaps_masked_start(),
			)?;
			chunk_id += 1;
		}

		writeln!(f, "")?;

		for sprite in &self.sprites {
			igrab_options.write_chunk_line(
				f,
				sprite,
				Some("SPR"),
				chunk_id,
				chunk_id == self.sprites_start(),
			)?;
			chunk_id += 1;
		}

		if igrab_options.version == IGrabVersion::ZeroPointFour {
			//writeln!(f, "\n// Misc chunks (externs)")?;
			chunk_id = self.misc_start();
			for misc in &self.misc_chunks {
				match misc {
					MiscChunk::Chunk(name)
					| MiscChunk::B8000Text(name)
					| MiscChunk::Article(name)
					| MiscChunk::Terminator(name) => {
						igrab_options.write_chunk_line(
							f, name, None, chunk_id, true,
						)?;
					}
					MiscChunk::Demo(num) => {
						if igrab_options.version
							== IGrabVersion::ZeroPointFour
						{
							writeln!(
								f,
								"\t\tDEMO{}={},",
								num, chunk_id
							)?;
						} else {
							writeln!(
								f,
								"#define DEMO{} {}",
								num, chunk_id
							)?;
						}
					}
				}
				chunk_id += 1;
			}
		}
		if igrab_options.version == IGrabVersion::ZeroPointFour {
			writeln!(f, "\t\tENUMEND\n\t     }} graphicnums;\n")?;
		}

		writeln!(f, "//\n// Data LUMPs\n//")?;
		// Keen doesn't actually define this in the GFX header, so it's commented out.
		//writeln!(f, "//#define NUMLUMPS {}", self.lumps.len())?;
		for lump in &self.lumps {
			writeln!(f, "#define {}_LUMP_START {}", lump.name, lump.start_chunk)?;
			writeln!(f, "#define {}_LUMP_END {}", lump.name, lump.end_chunk)?;
		}

		writeln!(f, "//\n// Amount of each data item\n//")?;
		writeln!(f, "#define NUMCHUNKS    {}", self.num_chunks())?;
		writeln!(f, "#define NUMFONT      {}", self.fonts.len())?;
		writeln!(f, "#define NUMFONTM     {}", self.fonts_masked.len())?;
		writeln!(f, "#define NUMPICS      {}", self.bitmaps.len())?;
		writeln!(f, "#define NUMPICM      {}", self.bitmaps_masked.len())?;
		writeln!(f, "#define NUMSPRITES   {}", self.sprites.len())?;
		writeln!(f, "#define NUMTILE8     {}", self.tile8_count)?;
		writeln!(f, "#define NUMTILE8M    {}", self.tile8_masked_count)?;
		writeln!(f, "#define NUMTILE16    {}", self.tile16_count)?;
		writeln!(f, "#define NUMTILE16M   {}", self.tile16_masked_count)?;
		writeln!(f, "#define NUMTILE32    {}", self.tile32_count)?;
		writeln!(f, "#define NUMTILE32M   {}", self.tile32_masked_count)?;

		writeln!(f, "//\n// File offsets for data items\n//")?;
		writeln!(f, "#define STRUCTPIC    0")?;
		writeln!(f, "#define STRUCTPICM   1")?;
		writeln!(f, "#define STRUCTSPRITE 2")?;
		writeln!(f, "")?;
		writeln!(f, "#define STARTFONT    {}", self.fonts_start())?;
		writeln!(f, "#define STARTFONTM   {}", self.fonts_masked_start())?;
		writeln!(f, "#define STARTPICS    {}", self.bitmaps_start())?;
		writeln!(f, "#define STARTPICM    {}", self.bitmaps_masked_start())?;
		writeln!(f, "#define STARTSPRITES {}", self.sprites_start())?;
		writeln!(f, "#define STARTTILE8   {}", self.tile8_start())?;
		writeln!(f, "#define STARTTILE8M  {}", self.tile8_masked_start())?;
		writeln!(f, "#define STARTTILE16  {}", self.tile16_start())?;
		writeln!(f, "#define STARTTILE16M {}", self.tile16_masked_start())?;
		writeln!(f, "#define STARTTILE32  {}", self.tile32_start())?;
		writeln!(f, "#define STARTTILE32M {}", self.tile32_masked_start())?;
		writeln!(f, "#define STARTEXTERNS {}", self.misc_start())?;

		writeln!(f, "")?;
		writeln!(f, "//")?;
		writeln!(f, "// Thank you for using idGrab!")?;
		writeln!(f, "//")?;

		Ok(())
	}

	fn save_igrab_header(
		&self,
		filename: &str,
		igrab_options: &IGrabOptions,
	) -> std::io::Result<()> {
		let igrab_file = std::fs::File::create(filename)?;
		let mut igrab_writer = std::io::BufWriter::new(igrab_file);
		self.write_igrab_header(&mut igrab_writer, igrab_options)
	}
	fn write_igrab_asm_header(
		&self,
		f: &mut dyn std::io::Write,
		igrab_options: &IGrabOptions,
	) -> std::io::Result<()> {
		writeln!(f, ";=====================================")?;
		writeln!(f, ";")?;
		if let Some(ext) = &self.extension {
			writeln!(f, "; Graphics .H file for .{}", ext)?;
		}
		#[cfg(feature = "timestamps")]
		write!(f, "; idGrab-ed on {}", GfxHeaders::timestamp())?;
		writeln!(f, "; idGrab emulating IGRAB {}", igrab_options.version)?;
		writeln!(f, ";")?;
		writeln!(f, ";=====================================\n")?;

		let mut chunk_id = self.bitmaps_start();

		/* Fonts are not included, nor masked fonts. */

		for pic in &self.bitmaps {
			igrab_options.write_asm_chunk_line(f, pic, Some("PIC"), chunk_id)?;
			chunk_id += 1;
		}

		writeln!(f, "")?;

		for picm in &self.bitmaps_masked {
			igrab_options.write_asm_chunk_line(f, picm, Some("PICM"), chunk_id)?;
			chunk_id += 1;
		}

		writeln!(f, "")?;

		for sprite in &self.sprites {
			igrab_options.write_asm_chunk_line(f, sprite, Some("SPR"), chunk_id)?;
			chunk_id += 1;
		}

		if igrab_options.version == IGrabVersion::ZeroPointFour {
			//writeln!(f, "\n// Misc chunks (externs)")?;
			chunk_id = self.misc_start();
			for misc in &self.misc_chunks {
				match misc {
					MiscChunk::Chunk(name)
					| MiscChunk::B8000Text(name)
					| MiscChunk::Article(name)
					| MiscChunk::Terminator(name) => {
						igrab_options.write_asm_chunk_line(
							f, name, None, chunk_id,
						)?;
					}
					MiscChunk::Demo(num) => {
						writeln!(f, "DEMO{}  \t=\t{}", num, chunk_id)?;
					}
				}
				chunk_id += 1;
			}
		}

		writeln!(f, "")?;
		// Keen doesn't actually define this in the GFX header, so it's commented out.
		//writeln!(f, "//#define NUMLUMPS {}", self.lumps.len())?;
		for lump in &self.lumps {
			writeln!(f, "{}_LUMP_START  \t=\t{}", lump.name, lump.start_chunk)?;
			writeln!(f, "{}_LUMP_END  \t=\t{}", lump.name, lump.end_chunk)?;
		}

		writeln!(f, ";\n; Amount of each data item\n;")?;
		writeln!(f, "NUMCHUNKS\t=\t{}", self.num_chunks())?;
		writeln!(f, "NUMFONT  \t=\t{}", self.fonts.len())?;
		writeln!(f, "NUMFONTM  \t=\t{}", self.fonts_masked.len())?;
		writeln!(f, "NUMPICS  \t=\t{}", self.bitmaps.len())?;
		writeln!(f, "NUMPICM  \t=\t{}", self.bitmaps_masked.len())?;
		writeln!(f, "NUMSPRITES  \t=\t{}", self.sprites.len())?;
		writeln!(f, "NUMTILE8  \t=\t{}", self.tile8_count)?;
		writeln!(f, "NUMTILE8M  \t=\t{}", self.tile8_masked_count)?;
		writeln!(f, "NUMTILE16  \t=\t{}", self.tile16_count)?;
		writeln!(f, "NUMTILE16M  \t=\t{}", self.tile16_masked_count)?;
		writeln!(f, "NUMTILE32  \t=\t{}", self.tile32_count)?;
		writeln!(f, "NUMTILE32M  \t=\t{}", self.tile32_masked_count)?;

		writeln!(f, ";\n; File offsets for data items\n;")?;
		writeln!(f, "STRUCTPIC  \t=\t0")?;
		writeln!(f, "STRUCTPICM  \t=\t1")?;
		writeln!(f, "STRUCTSPRITE  \t=\t2")?;
		writeln!(f, "")?;
		writeln!(f, "STARTFONT  \t=\t{}", self.fonts_start())?;
		writeln!(f, "STARTFONTM  \t=\t{}", self.fonts_masked_start())?;
		writeln!(f, "STARTPICS  \t=\t{}", self.bitmaps_start())?;
		writeln!(f, "STARTPICM  \t=\t{}", self.bitmaps_masked_start())?;
		writeln!(f, "STARTSPRITES  \t=\t{}", self.sprites_start())?;
		writeln!(f, "STARTTILE8  \t=\t{}", self.tile8_start())?;
		writeln!(f, "STARTTILE8M  \t=\t{}", self.tile8_masked_start())?;
		writeln!(f, "STARTTILE16  \t=\t{}", self.tile16_start())?;
		writeln!(f, "STARTTILE16M  \t=\t{}", self.tile16_masked_start())?;
		writeln!(f, "STARTTILE32  \t=\t{}", self.tile32_start())?;
		writeln!(f, "STARTTILE32M  \t=\t{}", self.tile32_masked_start())?;
		writeln!(f, "STARTEXTERNS  \t=\t{}", self.misc_start())?;

		writeln!(f, "")?;
		writeln!(f, ";")?;
		writeln!(f, "; Thank you for using idGrab!")?;
		writeln!(f, ";")?;

		Ok(())
	}

	fn save_igrab_asm_header(
		&self,
		filename: &str,
		igrab_options: &IGrabOptions,
	) -> std::io::Result<()> {
		let igrab_file = std::fs::File::create(filename)?;
		let mut igrab_writer = std::io::BufWriter::new(igrab_file);
		self.write_igrab_asm_header(&mut igrab_writer, igrab_options)
	}

	fn write_omnispeak_cfg(&self, f: &mut dyn std::io::Write) -> std::io::Result<()> {
		writeln!(f, "# GFX Header (Omnispeak)\n")?;
		let mut chunk_id = self.header_chunk_count;

		writeln!(f, "# Fonts")?;
		for font in &self.fonts {
			writeln!(f, "%int FON_{} {}", font, chunk_id)?;
			chunk_id += 1;
		}

		writeln!(f, "# Masked Fonts")?;
		for font in &self.fonts_masked {
			writeln!(f, "%int FONM_{} {}", font, chunk_id)?;
			chunk_id += 1;
		}

		writeln!(f, "# Bitmaps")?;
		for pic in &self.bitmaps {
			writeln!(f, "%int PIC_{} {}", pic, chunk_id)?;
			chunk_id += 1;
		}

		writeln!(f, "# Masked Bitmaps")?;
		for picm in &self.bitmaps_masked {
			writeln!(f, "%int MPIC_{} {}", picm, chunk_id)?;
			chunk_id += 1;
		}

		writeln!(f, "# Sprites")?;
		for sprite in &self.sprites {
			writeln!(f, "%int SPR_{} {}", sprite, chunk_id)?;
			chunk_id += 1;
		}

		/* Tile8 and Tile8m are stored in a single chunk each. */
		chunk_id += if self.tile8_count != 0 { 1 } else { 0 };

		chunk_id += if self.tile8_masked_count != 0 { 1 } else { 0 };

		chunk_id += self.tile16_count;

		chunk_id += self.tile16_masked_count;

		chunk_id += self.tile32_count;

		chunk_id += self.tile32_masked_count;

		let mut demostart: Option<u32> = None;
		for chunk in &self.misc_chunks {
			match chunk {
				MiscChunk::Chunk(name) => {
					writeln!(f, "%int EXTERN_{} {}", name, chunk_id)?;
				}
				MiscChunk::B8000Text(name) => {
					writeln!(f, "%int EXTERN_{} {}", name, chunk_id)?;
				}
				MiscChunk::Article(name) => {
					writeln!(f, "%int TEXT_{} {}", name, chunk_id)?;
				}
				MiscChunk::Terminator(name) => {
					writeln!(f, "%int EXTERN_{} {}", name, chunk_id)?;
				}
				MiscChunk::Demo(num) => {
					if demostart.is_none() {
						demostart = Some(chunk_id);
					}
					writeln!(f, "# Demo {} = {}", num, chunk_id)?;
				}
			}
			chunk_id += 1;
		}
		writeln!(f, "%int DEMOSTART {}", demostart.unwrap())?;

		writeln!(f, "#\n# Lumps\n#")?;
		writeln!(f, "%int NUMLUMPS {}", self.lumps.len())?;
		writeln!(f, "%intarray lumpStarts")?;
		let mut lump_start_iterator = self.lumps.iter().peekable();
		while let Some(lump) = lump_start_iterator.next() {
			let start_chunk_name = self.omnispeak_chunk_name(lump.start_chunk);
			let comma = if lump_start_iterator.peek().is_none() {
				""
			} else {
				","
			};
			if start_chunk_name.is_some() {
				writeln!(f, "\t@{}{}", start_chunk_name.unwrap(), comma)?;
			} else {
				writeln!(f, "\t{}{}", lump.start_chunk, comma)?;
			}
		}
		writeln!(f, "%intarray lumpEnds")?;
		let mut lump_end_iterator = self.lumps.iter().peekable();
		while let Some(lump) = lump_end_iterator.next() {
			let end_chunk_name = self.omnispeak_chunk_name(lump.end_chunk);
			let comma = if lump_end_iterator.peek().is_none() {
				""
			} else {
				","
			};
			if end_chunk_name.is_some() {
				writeln!(f, "\t@{}{}", end_chunk_name.unwrap(), comma)?;
			} else {
				writeln!(f, "\t{}{}", lump.end_chunk, comma)?;
			}
		}
		writeln!(f, "# Lump names")?;
		for (i, lump) in self.lumps.iter().enumerate() {
			writeln!(f, "%int LUMP_{} {}", lump.name, i)?;
		}
		Ok(())
	}

	fn save_omnispeak_cfg(&self, filename: &str) -> std::io::Result<()> {
		let omnispeak_file = std::fs::File::create(filename)?;
		let mut omnispeak_writer = std::io::BufWriter::new(omnispeak_file);
		self.write_omnispeak_cfg(&mut omnispeak_writer)
	}
}

fn parse_gfx_script(filename: &str) -> std::io::Result<GfxHeaders> {
	let script_data = std::fs::read_to_string(filename)?;
	let mut lexer = parser::Lexer::from_str(script_data.as_str());
	let mut current_lump: Option<Lump> = None;

	let mut headers = GfxHeaders::default();
	headers.header_chunk_count = 3;

	loop {
		let entry_type = lexer.next_token();
		match entry_type {
			None => {
				break;
			}
			Some(parser::Token::Ident("Extension")) => {
				headers.extension = Some(lexer.get_string_literal());
			}
			Some(parser::Token::Ident("Fonts")) => {
				lexer.expect_symbol('{');
				loop {
					let font_tok = lexer.next_token();
					match font_tok {
						Some(parser::Token::Symbol('}')) => {
							if current_lump.is_some() {
								let mut lump = current_lump
									.take()
									.unwrap();
								lump.end_chunk = headers
									.fonts_start()
									+ headers.fonts.len()
										as u32 - 1;
								headers.lumps.push(lump);
							} else {
								break;
							}
						}
						Some(parser::Token::Ident("Lump")) => {
							if current_lump.is_some() {
								panic!("Tried to nest a lump!");
							}
							current_lump = Some(Lump {
								name: lexer.get_string_literal(),
								start_chunk: headers.fonts_start()
									+ headers.fonts.len()
										as u32,
								end_chunk: 0,
							});
							lexer.expect_symbol('{');
						}
						Some(parser::Token::StringLiteral(font_name)) => {
							headers.fonts.push(font_name);
						}
						None => {
							break;
						}
						_ => {
							panic!("Unknown token");
						}
					}
				}
			}
			Some(parser::Token::Ident("FontsMasked")) => {
				lexer.expect_symbol('{');
				loop {
					let font_tok = lexer.next_token();
					match font_tok {
						Some(parser::Token::Symbol('}')) => {
							if current_lump.is_some() {
								let mut lump = current_lump
									.take()
									.unwrap();
								lump.end_chunk = headers
									.fonts_masked_start()
									+ headers.fonts_masked.len()
										as u32 - 1;
								headers.lumps.push(lump);
							} else {
								break;
							}
						}
						Some(parser::Token::Ident("Lump")) => {
							if current_lump.is_some() {
								panic!("Tried to nest a lump!");
							}
							current_lump = Some(Lump {
								name: lexer.get_string_literal(),
								start_chunk: headers
									.fonts_masked_start()
									+ headers.fonts_masked.len()
										as u32,
								end_chunk: 0,
							});
							lexer.expect_symbol('{');
						}
						Some(parser::Token::StringLiteral(font_name)) => {
							headers.fonts_masked.push(font_name);
						}
						None => {
							break;
						}
						_ => {
							panic!("Unknown token");
						}
					}
				}
			}
			Some(parser::Token::Ident("Bitmaps")) => {
				lexer.expect_symbol('{');
				loop {
					let bmp_tok = lexer.next_token();
					match bmp_tok {
						Some(parser::Token::Symbol('}')) => {
							if current_lump.is_some() {
								let mut lump = current_lump
									.take()
									.unwrap();
								lump.end_chunk = headers
									.bitmaps_start()
									+ headers.bitmaps.len()
										as u32 - 1;
								headers.lumps.push(lump);
							} else {
								break;
							}
						}
						Some(parser::Token::Ident("Lump")) => {
							if current_lump.is_some() {
								panic!("Tried to nest a lump!");
							}
							current_lump = Some(Lump {
								name: lexer.get_string_literal(),
								start_chunk: headers
									.bitmaps_start()
									+ headers.bitmaps.len()
										as u32,
								end_chunk: 0,
							});
							lexer.expect_symbol('{');
						}
						Some(parser::Token::StringLiteral(bmp_name)) => {
							headers.bitmaps.push(bmp_name);
						}
						None => {
							break;
						}
						_ => {
							panic!("Unknown token");
						}
					}
				}
			}
			Some(parser::Token::Ident("BitmapsMasked")) => {
				lexer.expect_symbol('{');
				loop {
					let bmp_tok = lexer.next_token();
					match bmp_tok {
						Some(parser::Token::Symbol('}')) => {
							if current_lump.is_some() {
								let mut lump = current_lump
									.take()
									.unwrap();
								lump.end_chunk = headers
									.bitmaps_masked_start()
									+ headers
										.bitmaps_masked
										.len() as u32 - 1;
								headers.lumps.push(lump);
							} else {
								break;
							}
						}
						Some(parser::Token::Ident("Lump")) => {
							if current_lump.is_some() {
								panic!("Tried to nest a lump!");
							}
							current_lump = Some(Lump {
								name: lexer.get_string_literal(),
								start_chunk: headers
									.bitmaps_masked_start()
									+ headers
										.bitmaps_masked
										.len() as u32,
								end_chunk: 0,
							});
							lexer.expect_symbol('{');
						}
						Some(parser::Token::StringLiteral(bmp_name)) => {
							headers.bitmaps_masked.push(bmp_name);
						}
						None => {
							break;
						}
						_ => {
							panic!("Unknown token");
						}
					}
				}
			}
			Some(parser::Token::Ident("Sprites")) => {
				lexer.expect_symbol('{');
				loop {
					let sprite_tok = lexer.next_token();
					match sprite_tok {
						Some(parser::Token::Symbol('}')) => {
							if current_lump.is_some() {
								let mut lump = current_lump
									.take()
									.unwrap();
								lump.end_chunk = headers
									.sprites_start()
									+ headers.sprites.len()
										as u32 - 1;
								headers.lumps.push(lump);
							} else {
								break;
							}
						}
						Some(parser::Token::Ident("Lump")) => {
							if current_lump.is_some() {
								panic!("Tried to nest a lump!");
							}
							current_lump = Some(Lump {
								name: lexer.get_string_literal(),
								start_chunk: headers
									.sprites_start()
									+ headers.sprites.len()
										as u32,
								end_chunk: 0,
							});
							lexer.expect_symbol('{');
						}
						Some(parser::Token::StringLiteral(spr_name)) => {
							headers.sprites.push(spr_name);
						}
						None => {
							break;
						}
						_ => {
							panic!("Unknown token");
						}
					}
				}
			}
			Some(parser::Token::Ident("Tiles8")) => {
				let num_tiles8 = lexer.get_int_literal() as u32;
				headers.tile8_count = num_tiles8;
			}
			Some(parser::Token::Ident("Tiles8Masked")) => {
				let num_tiles8m = lexer.get_int_literal() as u32;
				headers.tile8_masked_count = num_tiles8m;
			}
			Some(parser::Token::Ident("Tiles16")) => {
				let num_tiles16 = lexer.get_int_literal() as u32;
				headers.tile16_count = num_tiles16;
			}
			Some(parser::Token::Ident("Tiles16Masked")) => {
				let num_tiles16m = lexer.get_int_literal() as u32;
				headers.tile16_masked_count = num_tiles16m;
			}
			Some(parser::Token::Ident("Tiles32")) => {
				let num_tiles32 = lexer.get_int_literal() as u32;
				headers.tile32_count = num_tiles32;
			}
			Some(parser::Token::Ident("Tiles32Masked")) => {
				let num_tiles32m = lexer.get_int_literal() as u32;
				headers.tile32_masked_count = num_tiles32m;
			}
			Some(parser::Token::Ident("Chunk")) => {
				let chunk_name = lexer.get_string_literal();
				headers.misc_chunks.push(MiscChunk::Chunk(chunk_name));
			}
			Some(parser::Token::Ident("Article")) => {
				let chunk_name = lexer.get_string_literal();
				headers.misc_chunks.push(MiscChunk::Article(chunk_name));
			}
			Some(parser::Token::Ident("B8000Text")) => {
				let chunk_name = lexer.get_string_literal();
				headers.misc_chunks.push(MiscChunk::B8000Text(chunk_name));
			}
			Some(parser::Token::Ident("Terminator")) => {
				let chunk_name = lexer.get_string_literal();
				headers.misc_chunks.push(MiscChunk::Terminator(chunk_name));
			}
			Some(parser::Token::Ident("Demo")) => {
				let demo_number = lexer.get_int_literal() as u32;
				headers.misc_chunks.push(MiscChunk::Demo(demo_number));
			}
			Some(_) => {
				panic!("Unknown token");
			}
		}
	}

	Ok(headers)
}

fn show_usage() {
	println!("Usage: idgrab <script> [options]");
	println!("\t--gfxinfo <filename>");
	println!("\t\tGenerates a GFXINFO(E) file for use with TED or Omnispeak");
	println!("\t--modid <filename>");
	println!("\t\tWrites a modid/ugrab compatible .def file.");
	println!("\t--omnispeak <filename>");
	println!("\t\tGenerates an omnispeak-compatible GFXCHUNKS variable file");
	println!("\t--igrab-header <filename>");
	println!("\t\tCreates a GRAPHEXT/GFXE_EXT C header file.");
	println!("\t--igrab-asm <filename>");
	println!("\t\tCreates a GRAPHEXT/GFXE_EXT assembly (.EQU) header.");
	println!("\t--igrab-version <0.24 | 0.4>");
	println!("\t\tEmulate the output from a specific IGRAB version.");
	println!("\t--igrab-underscore-separator");
	println!("\t\tAdd an underscore before chunk name suffixes (e.g., _SPR)");
}

fn main() {
	let args: Vec<std::string::String> = std::env::args().collect(); /* Skip the application name. */
	if args.len() <= 1 {
		show_usage();
		return;
	}
	let script_filename = &args[1];
	let headers = parse_gfx_script(script_filename).unwrap();

	let mut arg_iter = args.iter().skip(2);

	/* We default to 0.4 for igrab output. */
	let mut igrab_options = IGrabOptions::default();

	while let Some(arg) = arg_iter.next() {
		match arg.as_str() {
			"--gfxinfo" => {
				let filename = arg_iter.next().unwrap().as_str();
				headers.save_gfxinfoe(filename).unwrap();
			}
			"--modid" => {
				let filename = arg_iter.next().unwrap().as_str();
				headers.save_modid_script(filename).unwrap();
			}
			"--omnispeak" => {
				let filename = arg_iter.next().unwrap().as_str();
				headers.save_omnispeak_cfg(filename).unwrap();
			}
			"--igrab-version" => {
				let ver_str = arg_iter.next().unwrap().as_str();
				igrab_options.version = match ver_str {
					"0.24" => IGrabVersion::ZeroPointTwoFour,
					"0.4" => IGrabVersion::ZeroPointFour,
					_ => panic!("Invalid IGRAB version. Only 0.24 and 0.4 are supported!"),
				};
			}
			"--igrab-underscore-separator" => {
				igrab_options.append_underscores = true;
			}
			"--igrab-header" => {
				let filename = arg_iter.next().unwrap().as_str();
				headers.save_igrab_header(filename, &igrab_options).unwrap();
			}
			"--igrab-asm" => {
				let filename = arg_iter.next().unwrap().as_str();
				headers.save_igrab_asm_header(filename, &igrab_options)
					.unwrap();
			}
			_ => {
				show_usage();
				return;
			}
		}
	}
}
