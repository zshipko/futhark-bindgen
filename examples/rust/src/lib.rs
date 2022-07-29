include!(concat!(env!("OUT_DIR"), "/example.rs"));

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn it_works() {
        let ctx = Context::new_with_options(Options::new().debug().log().profile()).unwrap();
        let data = &[1, 2, 3, 4, 5, 7, 8];
        let arr = ArrayI64D1::new(&ctx, [data.len() as i64], data).unwrap();
        let index = ctx.binary_search(&arr, 6).unwrap();
        assert_eq!(index, 5);
    }
}
