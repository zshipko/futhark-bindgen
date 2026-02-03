impl<'a> {rust_type}<'a> {{
    /// Construct a {rust_type} with variant {variant_name}
    pub fn {variant_name}(ctx: &'a Context{params}) -> std::result::Result<Self, Error> {{
        unsafe {{
            let mut out = std::ptr::null_mut();
            let rc = {construct_fn}(ctx.context, &mut out{args});
            if rc != 0 {{ return Err(Error::Code(rc)); }}
            ctx.auto_sync();
            Ok(Self {{ data: out, ctx }})
        }}
    }}
}}

extern "C" {{
    fn {construct_fn}(
        _: *mut futhark_context,
        _: *mut *mut {futhark_type}{extern_params}
    ) -> std::os::raw::c_int;
}}
