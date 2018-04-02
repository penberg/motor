#![feature(plugin)]
#![plugin(dynasm)]

extern crate clap;
extern crate dynasmrt;
extern crate motor;

use clap::{App, Arg};
use dynasmrt::DynasmApi;
use motor::binary::Module;
use motor::opcode::*;
use std::fs::File;
use std::mem;

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
    let module = Module::parse(&mut f).unwrap();
    let start_fn = module.find_start_func().unwrap();
    let mut ops = dynasmrt::x64::Assembler::new();
    let entry = ops.offset();
    for insn in &start_fn.code {
        match *insn {
            OPC_RETURN => {
                dynasm!(ops
                  ; ret
              );
            }
            _ => panic!("Unsupported instruction {:x}", insn),
        }
    }
    let buf = ops.finalize().unwrap();
    let entry_fn: extern "C" fn() -> bool = unsafe { mem::transmute(buf.ptr(entry)) };
    entry_fn();
}
