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

pub fn read_byte(reader: &mut dyn std::io::Read) -> std::io::Result<u8> {
	let mut out_byte: u8 = 0;
	reader.read_exact(std::slice::from_mut(&mut out_byte))?;
	return Ok(out_byte);
}

pub fn read_le16(reader: &mut dyn std::io::Read) -> std::io::Result<u16> {
	let mut raw_bytes = [0 as u8; 2];
	reader.read_exact(&mut raw_bytes)?;
	return Ok(u16::from_le_bytes(raw_bytes));
}

pub fn read_le32(reader: &mut dyn std::io::Read) -> std::io::Result<u32> {
	let mut raw_bytes = [0 as u8; 4];
	reader.read_exact(&mut raw_bytes)?;
	return Ok(u32::from_le_bytes(raw_bytes));
}

pub fn read_be16(reader: &mut dyn std::io::Read) -> std::io::Result<u16> {
	let mut raw_bytes = [0 as u8; 2];
	reader.read_exact(&mut raw_bytes)?;
	return Ok(u16::from_be_bytes(raw_bytes));
}

pub fn read_be32(reader: &mut dyn std::io::Read) -> std::io::Result<u32> {
	let mut raw_bytes = [0 as u8; 4];
	reader.read_exact(&mut raw_bytes)?;
	return Ok(u32::from_be_bytes(raw_bytes));
}

pub fn write_byte(out_byte: u8, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
	writer.write_all(std::slice::from_ref(&out_byte))
}

pub fn write_be16(out_val: u16, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
	let raw_bytes = out_val.to_be_bytes();
	writer.write_all(&raw_bytes)
}

pub fn write_be32(out_val: u32, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
	let raw_bytes = out_val.to_be_bytes();
	writer.write_all(&raw_bytes)
}

pub fn write_le16(out_val: u16, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
	let raw_bytes = out_val.to_le_bytes();
	writer.write_all(&raw_bytes)
}

pub fn write_le32(out_val: u32, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
	let raw_bytes = out_val.to_le_bytes();
	writer.write_all(&raw_bytes)
}
