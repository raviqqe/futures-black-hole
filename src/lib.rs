mod black_hole;

extern crate futures;

pub use black_hole::BlackHole;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
