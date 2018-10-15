#[macro_use]
extern crate log;

extern crate collision;

pub mod picking;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
