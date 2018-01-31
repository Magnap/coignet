#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(use_nested_groups)]
#![feature(never_type)]
#![feature(try_trait)]

extern crate rocket;
extern crate rocket_contrib;

extern crate failure;

extern crate bincode;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

extern crate lru;

pub mod endpoints;
pub mod backend;
pub mod utils;
