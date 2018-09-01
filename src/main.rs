#![recursion_limit="128"]

#[macro_use]
mod json_builder;

use json_builder::{Result, JSONBuilder, IntoJSON, Error};

const FOO: &'static str = "const FOO";

struct MyStruct {
	hidden: String,
	foo: i32,
	bar: String,
	baz: Vec<bool>,
	opt: Option<f32>
}

struct TinyStruct {
	i: i32
}

impl_into_json! {
	MyStruct,
	foo, bar, baz, opt,
	virtual_field: "...",
	"with spaces": |this| this.foo - 44,
	bla: |this| this.foo as usize + this.baz.len(),
	x: {1 + 2},
	y: 3 + 4,
	z: |_this| {
		println!("/* serializing MyStruct */");
		None as Option<i32>
	},
	// use an expression as key
	[FOO]: "FOO?",
	["\r".to_string().as_str()]: "\n"
}

impl_into_json! {
	TinyStruct, i
}

fn do_stuff() -> Result {
	let mut out = std::io::stdout();
	let mut b = JSONBuilder::new(&mut out);
	let null: Option<String> = None;
	let optstr = Some("foo");
	let multi = Some(Some(Some("multi")));
	let boxed = Box::new(12);
	let my_struct = MyStruct {
		hidden: "this is not serialized".to_string(),
		foo: 354,
		bar: "bl bla".to_string(),
		baz: vec![true, false],
		opt: Some(-1.3e-2)
	};
	println!("the my_struct.hidden field won't be serialized: {}", my_struct.hidden);

	b.begin_object()?;
		b.item("foo", "bar")?;
		b.key("bla \" \n")?;
		b.begin_array()?;
			b.value(true)?;
			b.value(false)?;
			b.null()?;
			b.value(123)?;
			b.value(12.3)?;
			b.value(optstr)?;
			b.value(&multi)?;
			b.value(&null)?;
			b.value('x')?;
			b.value("a string")?;
			b.value(&boxed)?;
			b.value(&my_struct)?;
			b.value(TinyStruct { i: 1 })?;
		b.end_array()?;
	b.end_object()?;
	b.end()?;
	println!();
	println!("{}", my_struct.to_json().ok().unwrap());
	println!("{}", my_struct.to_pretty_json(3, false).ok().unwrap());

	let s = "a string";
	let i = 123;
	let b = false;
	let list = vec![1, 2, 3];
	let nest_vec = vec![vec![], vec![2], vec![3]];
	let mut map = std::collections::HashMap::new();
	map.insert("foo", "bar");

	let mut map2 = std::collections::HashMap::new();
	map2.insert("egg".to_string(), "spam");

	let array = ["a", " "];

	let json = json!(pretty {
		"key": ["foo", -12, 1 - 2, [], [[]]],
		"another key": true,
		s: s,
		"null": None as Option<i16>,
		"str": optstr,
		"bool": b,
		&true.to_string(): true,
		"int": i,
		"vec": &list,
		"vec2": &nest_vec,
		"empty1": [],
		"empty2": {},
		"boxed": boxed,
		"nested": {
			s: [1, {
				"nested": 2
			}]
		},
		"my_struct": &my_struct,
		"tiny_struct": TinyStruct { i: 1 },
		"vec3": vec!['a', 'b'],
		"map": &map,
		"map2": &map2,
		"json": json!({"foo": -12})?,
		"array": &array
	})?;
	println!("{}", json);

	Ok(())
}

fn main() {
	match do_stuff() {
		Err(Error::State(got, expected)) => println!("Error: illegal state: {:?}, expected one of: {:?}", got, expected),
		Err(Error::IO(err)) => println!("Error: IO error: {}", err),
		_ => {}
	}
}
