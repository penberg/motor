extern crate clap;
extern crate motor;

use clap::{Arg, App};
use std::fs::File;
use motor::binary::Module;

fn main() {
    let matches = App::new("Motor")
        .version("0.1")
        .author("Pekka Enberg <penberg@iki.fi>")
        .about("Motor is a runtime for executing WebAssembly programs")
        .arg(
            Arg::with_name("input")
                .help("WebAssembly program to run")
                .required(true)
                .index(1),
        )
        .get_matches();
    let filename = matches.value_of("input").unwrap();
    let mut f = File::open(filename).expect("file not found");
    let module = Module::parse(&mut f);
    println!("{:?}", module);
}
