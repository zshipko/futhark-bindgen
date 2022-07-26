include!(concat!(env!("OUT_DIR"), "/example.rs"));

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn it_works() {
        let ctx = Context::new().unwrap();
        let mut dest = 0.0;
        let number = Number::new(&ctx, 2.0).unwrap();
        ctx.test(&mut dest, &number).unwrap();
        assert_eq!(dest, 4.0)
    }
}
