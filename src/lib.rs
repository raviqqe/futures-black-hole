#![feature(proc_macro, conservative_impl_trait, generators)]

extern crate futures_await as futures;
#[cfg(test)]
extern crate futures_cpupool;

mod black_hole;

pub use black_hole::BlackHole;
