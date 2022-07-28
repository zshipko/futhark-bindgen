impl<'a> {rust_type}<'a> {{
    pub fn new(ctx: &'a Context, {new_params}) -> std::result::Result<Self, Error> {{
        unsafe {{
            let mut out = std::ptr::null_mut();
            let rc = futhark_new_opaque_{name}(ctx.context, &mut out, {new_call_args});
            if rc != 0 {{ return Err(Error::Code(rc)); }}
            Ok(Self {{ data: out, ctx: ctx.context, _t: std::marker::PhantomData}})
        }}
    }}
}}

extern "C" {{
    fn futhark_new_opaque_{name}(
        _: *mut futhark_context,
        _: *mut *mut {futhark_type},
        {new_extern_params}
    ) -> std::os::raw::c_int;
}}