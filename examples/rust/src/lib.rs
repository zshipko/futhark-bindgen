include!(concat!(env!("OUT_DIR"), "/example.rs"));

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn binary_search() {
        let ctx = Context::new_with_options(Options::new().debug().log().profile()).unwrap();
        let data = &[1, 2, 3, 4, 5, 7, 8];
        let arr = ArrayI64D1::new(&ctx, [data.len() as i64], data).unwrap();
        let index = ctx.binary_search(&arr, 6).unwrap();
        assert_eq!(index, 5);
    }

    #[test]
    fn tuple() {
        let ctx = Context::new_with_options(Options::new().debug().log().profile()).unwrap();
        let data = &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let number = Number::new(&ctx, 2.5).unwrap();
        let arr = ArrayF32D1::new(&ctx, [data.len() as i64], data).unwrap();

        let t = Tup::new(&ctx, &number, &arr).unwrap();
        let out = ctx.tup_mul(&t).unwrap();
        let data1 = out.get().unwrap();

        for i in 0..10 {
            assert_eq!(data1[i], data[i] * t.get_0().unwrap().get_x().unwrap());
        }
    }

    #[test]
    fn count_lines() {
        let ctx = Context::new_with_options(Options::new().debug().log().profile()).unwrap();

        let data = String::from("this\nis\na\ntest\n").into_bytes();
        let data = ArrayU8D1::new(&ctx, [data.len() as i64], &data).unwrap();
        let n = ctx.count_lines(&data).unwrap();
        assert_eq!(n, 4);
    }
}
