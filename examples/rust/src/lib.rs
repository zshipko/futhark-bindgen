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
        assert_eq!(dest, 4.0);

        let init = &[1.0, 2.0, 3.0];
        let arr = ArrayF32D1::from_slice(&ctx, [3], init).unwrap();
        let tup = Tup::new(&ctx, &number, &arr).unwrap();
        let mut out = ArrayF32D1::new(&ctx, [3]).unwrap();
        ctx.tup_mul(&mut out, &tup).unwrap();
        ctx.sync().unwrap();

        let values = &mut [0.0, 0.0, 0.0];
        out.values(values).unwrap();
        for i in 0..3 {
            assert_eq!(values[i], init[i] * 2.0)
        }
    }
}
