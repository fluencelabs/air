#![deny(unused_imports, unused_variables, dead_code)]

#[cfg(test)]
#[macro_use]
extern crate fstrings;

pub mod ast;
mod lalrpop {
    #[cfg(test)]
    mod tests;

    #[rustfmt::skip]
    mod aqua;
    mod parser;

    pub use parser::parse;
}

pub use lalrpop::parse;
