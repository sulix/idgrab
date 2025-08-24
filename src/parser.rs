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

#[derive(PartialEq, Debug)]
pub enum Token<'a> {
	Ident(&'a str),
	Symbol(char),
	StringLiteral(String),
	NumericLiteral(i64),
}

pub struct Lexer<'a> {
	data: &'a str,
	offset: usize,
	line: usize,
	buffered_token: Option<Token<'a>>,
}

impl<'a> Lexer<'a> {
	pub fn from_str(data: &'a str) -> Lexer<'a> {
		Lexer {
			data,
			offset: 0,
			line: 1,
			buffered_token: None,
		}
	}

	fn unget_token(&mut self, token: Token<'a>) {
		assert!(self.buffered_token.is_none());
		self.buffered_token = Some(token);
	}

	fn peek_char(&self) -> Option<char> {
		self.data[self.offset..].chars().next()
	}

	fn eat_char(&mut self) {
		let c = self.peek_char().unwrap();
		if c == '\n' {
			self.line += 1;
		}
		self.offset += c.len_utf8();
	}

	fn eat_whitespace(&mut self) {
		loop {
			let c = self.peek_char();
			if c.is_none() {
				break;
			}
			if !c.unwrap().is_whitespace() {
				break;
			}
			self.eat_char();
		}
	}

	pub fn next_token(&mut self) -> Option<Token<'a>> {
		self.eat_whitespace();
		let start_offset = self.offset;
		loop {
			let opt_c = self.peek_char();
			match opt_c {
				None => {
					break;
				}
				Some(c) => {
					if c == '#' {
						// Start of a comment.
						loop {
							let comment_c = self.peek_char();
							if comment_c.is_none() {
								break;
							}
							if comment_c.unwrap() == '\n' {
								self.eat_char();
								break;
							}
							self.eat_char();
						}
						return self.next_token();
					} else if c == '"' {
						// Start of a string literal.
						let mut str_val = String::new();
						// Eat the opening quote.
						self.eat_char();
						loop {
							let str_c = self.peek_char();
							if str_c.is_none() {
								panic!("Unexpected end of file (missing '\"') on line {}", self.line);
							}
							self.eat_char();
							if str_c.unwrap() == '\"' {
								break;
							}
							str_val.push(str_c.unwrap());
						}
						return Some(Token::StringLiteral(str_val));
					} else if start_offset == self.offset
						&& (c.is_numeric() || c == '-')
					{
						// Start of a numeric (integer) literal.
						self.eat_char();
						loop {
							let int_c = self.peek_char();
							if int_c.is_none()
								|| !int_c.unwrap().is_numeric()
							{
								break;
							}
							self.eat_char();
						}
						let int_slice =
							&self.data[start_offset..self.offset];
						let int_val = int_slice.parse::<i64>().unwrap();
						return Some(Token::NumericLiteral(int_val));
					} else if c.is_whitespace() {
						if c == '\n' {
							self.line += 1;
						}
						break;
					} else if !c.is_alphanumeric() && c != '_' {
						if self.offset != start_offset {
							break;
						}
						self.eat_char();
						return Some(Token::Symbol(c));
					} else {
						self.eat_char();
					}
				}
			}
		}
		let end_offset = self.offset;
		if start_offset == end_offset {
			return None;
		}
		Some(Token::Ident(&self.data[start_offset..end_offset]))
	}

	pub fn expect_ident(&mut self, ident: &str) {
		let line = self.line;
		let tok = self.next_token();
		if tok.is_none() {
			panic!("Expected {} on line {}, but got EOF!", ident, line);
		}
		let tok_value = tok.unwrap();

		if tok_value != Token::Ident(ident) {
			panic!(
				"Expected {} on line {}, but got {:?}!",
				ident, line, tok_value
			);
		}
	}

	pub fn expect_symbol(&mut self, sym: char) {
		let line = self.line;
		let tok = self.next_token();
		if tok.is_none() {
			panic!("Expected '{}' on line {}, but got EOF!", sym, line);
		}
		let tok_value = tok.unwrap();

		if tok_value != Token::Symbol(sym) {
			panic!(
				"Expected '{}' on line {}, but got {:?}!",
				sym, line, tok_value
			);
		}
	}

	pub fn get_string_literal(&mut self) -> String {
		let line = self.line;
		let tok = self.next_token();
		if tok.is_none() {
			panic!("Expected string literal on line {}, but got EOF!", line);
		}
		let tok_value = tok.unwrap();
		if let Token::StringLiteral(str_val) = tok_value {
			return str_val;
		} else {
			panic!("Expected string on line {}, but got {:?}!", line, tok_value);
		}
	}

	pub fn get_int_literal(&mut self) -> i64 {
		let line = self.line;
		let tok = self.next_token();
		if tok.is_none() {
			panic!("Expected integer literal on line {}, but got EOF!", line);
		}
		let tok_value = tok.unwrap();
		if let Token::NumericLiteral(int_val) = tok_value {
			return int_val;
		} else {
			panic!(
				"Expected integer literal on line {}, but got {:?}!",
				line, tok_value
			);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn lexer_hello() {
		let hello_world = "Hello World";
		let mut lexer = Lexer::from_str(hello_world);
		let first_token = lexer.next_token().unwrap();
		assert_eq!(first_token, Token::Ident("Hello"));
		let second_token = lexer.next_token().unwrap();
		assert_eq!(second_token, Token::Ident("World"));

		assert!(lexer.next_token().is_none());
	}
	#[test]
	fn lexer_string_literal() {
		let input = "  \" This is a string \" ";
		let mut lexer = Lexer::from_str(input);
		let token = lexer.next_token().unwrap();
		assert_eq!(
			token,
			Token::StringLiteral(" This is a string ".to_string())
		);
		assert!(lexer.next_token().is_none());
	}
	#[test]
	fn lexer_script() {
		let test_input = "Filename=\"test.txt\"";
		let mut lexer = Lexer::from_str(test_input);
		assert_eq!(lexer.next_token().unwrap(), Token::Ident("Filename"));
		assert_eq!(lexer.next_token().unwrap(), Token::Symbol('='));
		assert_eq!(
			lexer.next_token().unwrap(),
			Token::StringLiteral("test.txt".to_string())
		);
		assert!(lexer.next_token().is_none());
	}
	#[test]
	fn lexer_script_with_ws() {
		let test_input = " Filename  =\n \"test.txt\"\n\n";
		let mut lexer = Lexer::from_str(test_input);
		assert_eq!(lexer.next_token().unwrap(), Token::Ident("Filename"));
		assert_eq!(lexer.next_token().unwrap(), Token::Symbol('='));
		assert_eq!(
			lexer.next_token().unwrap(),
			Token::StringLiteral("test.txt".to_string())
		);
		assert!(lexer.next_token().is_none());
	}
}
