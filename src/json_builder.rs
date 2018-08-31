extern crate std;

use std::io::Write;
use std::vec::Vec;

// TODO: custom derive

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum State {
	Begin,
	End,
	ArrayFirstElement,
	ArrayElement,
	ObjectFirstKey,
	ObjectKey,
	ObjectValue
}

pub enum Error {
	IO(std::io::Error),
	State(State)
}

pub struct JSONBuilder<'a> {
	stack: Vec<State>,
	writer: &'a mut Write,
	indent_size: usize,
	tab_indent: bool
}

pub type Result = std::result::Result<(), Error>;

pub trait IntoJSON {
	fn into_json(&self, builder: &mut JSONBuilder) -> Result;

	fn to_json(&self) -> std::result::Result<String, Error> {
		let mut data = Vec::<u8>::new();
		{
			let mut builder = JSONBuilder::new(&mut data);
			self.into_json(&mut builder)?;
			builder.end()?;
		}
		Ok(String::from_utf8(data).unwrap())
	}

	fn to_pretty_json(&self, indent_size: usize, tab_indent: bool) -> std::result::Result<String, Error> {
		let mut data = Vec::<u8>::new();
		{
			let mut builder = JSONBuilder::new_pretty(&mut data, indent_size, tab_indent);
			self.into_json(&mut builder)?;
			builder.end()?;
		}
		Ok(String::from_utf8(data).unwrap())
	}
}

macro_rules! write_bytes {
	($builder:expr, $bytes:expr) => {
		match $builder.writer.write_all($bytes) {
			Err(err) => return Err(Error::IO(err)),
			_ => {}
		}
	};
}

pub fn escape_json(s: &str, writer: &mut Write) -> std::io::Result<()> {
	let mut prev = 0usize;
	for (i, c) in s.char_indices() {
		match c {
			'\\' => {
				writer.write_all(s[prev..i].as_bytes())?;
				writer.write_all(b"\\\\")?;
				prev = i + 1;
			},

			'"' => {
				writer.write_all(s[prev..i].as_bytes())?;
				writer.write_all(b"\\\"")?;
				prev = i + 1;
			},

			'\n' => {
				writer.write_all(s[prev..i].as_bytes())?;
				writer.write_all(b"\\n")?;
				prev = i + 1;
			},

			'\r' => {
				writer.write_all(s[prev..i].as_bytes())?;
				writer.write_all(b"\\r")?;
				prev = i + 1;
			},

			'<' => {
				writer.write_all(s[prev..i].as_bytes())?;
				writer.write_all(b"\\u003c")?;
				prev = i + 1;
			},

			'>' => {
				writer.write_all(s[prev..i].as_bytes())?;
				writer.write_all(b"\\u003e")?;
				prev = i + 1;
			},

			'\0' => {
				writer.write_all(s[prev..i].as_bytes())?;
				writer.write_all(b"\\u0000")?;
				prev = i + 1;
			},
			

			_ => {}
		}
	}
	writer.write_all(s[prev..].as_bytes())
}

macro_rules! write_string {
	( $builder:expr, $str:expr ) => {		
		write_bytes!($builder, b"\"");
		match escape_json($str, $builder.writer) {
			Ok(()) => {},
			Err(err) => return Err(Error::IO(err)),
		}
		write_bytes!($builder, b"\"");
	}
}

// optimization to write up to 512 tabs/spaces at once (instead of byte-for-byte)
const TABS:[u8; 512] = [9u8; 512];
const SPACES:[u8; 512] = [32u8; 512];

impl<'a> JSONBuilder<'a> {
	pub fn new(writer: &mut Write) -> JSONBuilder {
		JSONBuilder {
			stack: vec![ State::Begin ],
			writer: writer,
			indent_size: 0,
			tab_indent: true
		}
	}

	pub fn new_pretty(writer: &mut Write, indent_size: usize, tab_indent: bool) -> JSONBuilder {
		JSONBuilder {
			stack: vec![ State::Begin ],
			writer: writer,
			indent_size: indent_size,
			tab_indent: tab_indent
		}
	}

	fn before_value(&mut self) -> Result {
		let current = *self.stack.last().unwrap();
		match current {
			State::ObjectFirstKey | State::ObjectKey | State::End =>
				return Err(Error::State(current)),

			State::ArrayElement => {
				write_bytes!(self, b",");
				self.indent()?;
			},

			State::ArrayFirstElement => {
				self.indent()?;
			},

			_ => {}
		}

		Ok(())
	}

	fn indent(&mut self) -> Result {
		if self.indent_size > 0 {
			write_bytes!(self, b"\n");
			let need = (self.stack.len() - 1) * self.indent_size;
			let indent = if self.tab_indent { TABS } else { SPACES };
			let avail = indent.len();
			if need < avail {
				write_bytes!(self, &indent[..need]);
			} else {
				let blocks = need / avail;
				for _ in 0..blocks {
					write_bytes!(self, &indent);
				}
				write_bytes!(self, &indent[..(need - blocks * avail)]);
			}
		}

		Ok(())
	}

	fn after_value(&mut self) {
		let i = self.stack.len() - 1;
		match self.stack[i] {
			State::ArrayFirstElement => {
				self.stack[i] = State::ArrayElement;
			},

			State::ObjectValue => {
				self.stack[i] = State::ObjectKey;
			},
			
			State::Begin => {
				self.stack[i] = State::End;
			},

			_ => {}
		}
	}

	pub fn value<Value: IntoJSON>(&mut self, value: Value) -> Result {
		value.into_json(self)
	}

	pub fn null(&mut self) -> Result {
		self.before_value()?;
		write_bytes!(self, b"null");
		self.after_value();
		Ok(())
	}

	pub fn key(&mut self, key: &str) -> Result {
		let i = self.stack.len() - 1;
		match self.stack[i] {
			State::ObjectFirstKey => {
				self.stack[i] = State::ObjectValue;
			},

			State::ObjectKey => {
				write_bytes!(self, b",");
				self.stack[i] = State::ObjectValue;
			},

			_ => return Err(Error::State(self.stack[i]))
		}

		self.indent()?;
		write_string!(self, key);
		if self.indent_size > 0 {
			write_bytes!(self, b": ");
		} else {
			write_bytes!(self, b":");
		}

		Ok(())
	}

	pub fn item<Value : IntoJSON>(&mut self, key: &str, value: Value) -> Result {
		self.key(key)?;
		self.value(value)
	}

	pub fn begin_array(&mut self) -> Result {
		self.before_value()?;
		self.stack.push(State::ArrayFirstElement);
		write_bytes!(self, b"[");
		Ok(())
	}

	pub fn end_array(&mut self) -> Result {
		let i = self.stack.len() - 1;
		match self.stack[i] {
			State::ArrayElement => {
				self.stack.pop();
				self.indent()?;
				write_bytes!(self, b"]");
				self.after_value();
			},

			State::ArrayFirstElement => {
				self.stack.pop();
				write_bytes!(self, b"]");
				self.after_value();
			},

			_ => return Err(Error::State(self.stack[i]))
		}

		Ok(())
	}

	pub fn begin_object(&mut self) -> Result {
		self.before_value()?;
		self.stack.push(State::ObjectFirstKey);
		write_bytes!(self, b"{");
		Ok(())
	}

	pub fn end_object(&mut self) -> Result {
		let i = self.stack.len() - 1;
		match self.stack[i] {
			State::ObjectKey =>
				{
					self.stack.pop();
					self.indent()?;
					write_bytes!(self, b"}");
					self.after_value();
				},

			State::ObjectFirstKey =>
				{
					self.stack.pop();
					write_bytes!(self, b"}");
					self.after_value();
				},

			_ => return Err(Error::State(self.stack[i]))
		}

		Ok(())
	}

	pub fn end(&mut self) -> Result {
		let n = self.stack.len();
		let current = self.stack[n - 1];

		if n != 1 || current != State::End {
			return Err(Error::State(current));
		}

		if self.indent_size > 0 {
			write_bytes!(self, b"\n");
		}

		Ok(())
	}
}

macro_rules! impl_into_json_for_primitive {
	($($t:ty),+) => {
		$(impl IntoJSON for $t {
			fn into_json(&self, builder: &mut JSONBuilder) -> Result {
				builder.before_value()?;
				match write!(builder.writer, "{}", self) {
					Err(err) => return Err(Error::IO(err)),
					_ => {}
				}
				builder.after_value();
				Ok(())
			}
		})*
	}
}

impl_into_json_for_primitive!{
	bool,
	i8, i16, i32, i64, i128,
	u8, u16, u32, u64, u128,
	isize, usize,
	f32, f64
}

impl<'a, T: IntoJSON> IntoJSON for &'a T {
	fn into_json(&self, builder: &mut JSONBuilder) -> Result {
		(*self).into_json(builder)
	}
}

impl<'a> IntoJSON for &'a IntoJSON {
	fn into_json(&self, builder: &mut JSONBuilder) -> Result {
		(*self).into_json(builder)
	}
}

impl<'a> IntoJSON for &'a str {
	fn into_json(&self, builder: &mut JSONBuilder) -> Result {
		builder.before_value()?;
		write_string!(builder, self);
		builder.after_value();
		Ok(())
	}
}

impl IntoJSON for String {
	fn into_json(&self, builder: &mut JSONBuilder) -> Result {
		builder.before_value()?;
		write_string!(builder, &self);
		builder.after_value();
		Ok(())
	}
}

impl IntoJSON for char {
	fn into_json(&self, builder: &mut JSONBuilder) -> Result {
		builder.before_value()?;
		write_string!(builder, self.to_string().as_str());
		builder.after_value();
		Ok(())
	}
}

impl<T: IntoJSON> IntoJSON for Option<T> {
	fn into_json(&self, builder: &mut JSONBuilder) -> Result {
		match self {
			Some(value) => value.into_json(builder),
			None => builder.null()
		}
	}
}

impl<T: IntoJSON> IntoJSON for Box<T> {
	fn into_json(&self, builder: &mut JSONBuilder) -> Result {
		(**self).into_json(builder)
	}
}

impl<T: IntoJSON> IntoJSON for Vec<T> {
	fn into_json(&self, builder: &mut JSONBuilder) -> Result {
		builder.begin_array()?;
		for item in self {
			builder.value(item)?;
		}
		builder.end_array()
	}
}

impl<'a, T: IntoJSON> IntoJSON for &'a [T] {
	fn into_json(&self, builder: &mut JSONBuilder) -> Result {
		builder.begin_array()?;
		for item in *self {
			builder.value(item)?;
		}
		builder.end_array()
	}
}

macro_rules! build_json_with_builder {
	// $x:tt$($y:expr)* is a hacky pattern that allows things like -12 and &obj without parenthesis
	($b:expr, [ $($x:tt$($y:expr)*),* ]) => {
		match $b.begin_array() { Err(err) => break Err(err), _ => {} }
		$(build_json_with_builder!($b, $x$($y)*);)*
		match $b.end_array() { Err(err) => break Err(err), _ => {} }
	};

	($b:expr, { $($key:expr => $x:tt$($y:expr)*),* }) => {
		match $b.begin_object() { Err(err) => break Err(err), _ => {} }
		$(
			match $b.key($key) { Err(err) => break Err(err), _ => {} }
			build_json_with_builder!($b, $x$($y)*);
		)*
		match $b.end_object() { Err(err) => break Err(err), _ => {} }
	};
	
	($b:expr, $val:expr) => {
		match $b.value($val) { Err(err) => break Err(err), _ => {} }
	};
}

#[macro_export]
macro_rules! build_json {
	($b:expr, $json:tt) => {
		loop {
			let mut builder = JSONBuilder::new($b);
			build_json_with_builder!(builder, $json);
			break builder.end();
		}
	}
}

#[macro_export]
macro_rules! json {
	($json:tt) => {
		loop {
			let mut data = Vec::<u8>::new();
			{
				let mut builder = JSONBuilder::new(&mut data);
				build_json_with_builder!(builder, $json);
				match builder.end() { Err(err) => break Err(err), _ => {} }
			}
			break Ok(String::from_utf8(data).unwrap());
		}
	}
}

#[macro_export]
macro_rules! pretty_json {
	($json:tt) => {
		pretty_json!(1, true, $json)
	};

	($indent_size:expr, $json:tt) => {
		pretty_json!($indent_size, true, $json)
	};

	($indent_size:expr, $tab_indent:expr, $json:tt) => {
		loop {
			let mut data = Vec::<u8>::new();
			{
				let mut builder = JSONBuilder::new_pretty(&mut data, $indent_size, $tab_indent);
				build_json_with_builder!(builder, $json);
				match builder.end() { Err(err) => break Err(err), _ => {} }
			}
			break Ok(String::from_utf8(data).unwrap());
		}
	};
}

macro_rules! impl_into_json_ {
	($b:expr, $s:expr) => {};
	($b:expr, $s:expr, ) => {};

	($b:expr, $s:expr, $id:ident => |$l:ident| $ex:expr) => {
		{
			let $l = $s;
			$b.item(stringify!($id), $ex)?;
		}
	};

	($b:expr, $s:expr, $id:ident => $ex:expr) => {
		$b.item(stringify!($id), $ex)?;
	};

	($b:expr, $s:expr, $id:ident) => {
		$b.item(stringify!($id), &$s.$id)?;
	};

	($b:expr, $s:expr, $id:ident => |$l:ident| $ex:expr, $($more:tt)*) => {
		{
			let $l = $s;
			$b.item(stringify!($id), $ex)?;
		}
		impl_into_json_!($b, $s, $($more)*);
	};

	($b:expr, $s:expr, $id:ident => $ex:expr, $($more:tt)*) => {
		$b.item(stringify!($id), $ex)?;
		impl_into_json_!($b, $s, $($more)*);
	};

	($b:expr, $s:expr, $id:expr => |$l:ident| $ex:expr) => {
		{
			let $l = $s;
			$b.item($id, $ex)?;
		}
	};

	($b:expr, $s:expr, $id:expr => $ex:expr) => {
		$b.item($id, $ex)?;
	};

	($b:expr, $s:expr, $id:expr => |$l:ident| $ex:expr, $($more:tt)*) => {
		{
			let $l = $s;
			$b.item($id, $ex)?;
		}
		impl_into_json_!($b, $s, $($more)*);
	};

	($b:expr, $s:expr, $id:expr => $ex:expr, $($more:tt)*) => {
		$b.item($id, $ex)?;
		impl_into_json_!($b, $s, $($more)*);
	};

	($b:expr, $s:expr, $id:ident, $($more:tt)*) => {
		$b.item(stringify!($id), &$s.$id)?;
		impl_into_json_!($b, $s, $($more)*);
	};
}

#[macro_export]
macro_rules! impl_into_json {
	($t:ty, $($def:tt)*) => {
		impl IntoJSON for $t {
			fn into_json(&self, builder: &mut JSONBuilder) -> Result {
				builder.begin_object()?;
				impl_into_json_!(builder, self, $($def)*);
				builder.end_object()
			}
		}
	}
}