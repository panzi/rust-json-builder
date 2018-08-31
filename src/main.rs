#[macro_use]
mod json_builder;

use json_builder::{Result, JSONBuilder, IntoJSON, Error};

struct MyStruct {
	foo: i32,
	bar: String,
	baz: Vec<bool>,
	opt: Option<f32>
}

impl_into_json! {
	MyStruct, foo, bar, baz, opt,
	virtual_field => {12},
	"with spaces" => |s| s.foo - 44
}

fn do_stuff() -> Result {
	let mut out = std::io::stdout();
	let mut b = JSONBuilder::new(&mut out);
	let null: Option<String> = None;
	let optstr = Some("foo");
	let multi = Some(Some(Some("multi")));
	let boxed = Box::new(12);
	let my_struct = MyStruct {
		foo: 354,
		bar: "bl bla".to_string(),
		baz: vec![true, false],
		opt: Some(-1.3e-2)
	};

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

	let json = pretty_json!({
		"key" => ["foo", -12, (1 - 2), [], [[]]],
		"another key" => true,
		s => s,
		"null" => (None as Option<i16>),
		"str" => optstr,
		"bool" => b,
		"true" => true,
		"int" => i,
		"vec" => &list,
		"vec2" => &nest_vec,
		"empty1" => [],
		"empty2" => {},
		"boxed" => boxed,
		"my_struct" => &my_struct
	})?;
	println!("{}", json);

	Ok(())
}

fn main() {
	match do_stuff() {
		Err(Error::State(state)) => println!("Error: illegal state: {:?}", state),
		Err(Error::IO(err)) => println!("Error: IO error: {}", err),
		_ => {}
	}
}
