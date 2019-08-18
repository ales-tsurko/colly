#![deny(
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts
)]
#![warn(
    deprecated_in_future,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unreachable_pub
)]

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate failure;
pub mod clock;
pub mod interpreter;
pub mod parser;
pub mod ast;
pub mod types;
