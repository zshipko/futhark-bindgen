impl<'a> {rust_type}<'a> {{
    /// Create new {rust_type}
    pub fn new(ctx: &'a Context, {new_params}) -> std::result::Result<Self, Error> {{
        unsafe {{
            let mut out = std::ptr::null_mut();
            let rc = {new_fn}(ctx.context, &mut out, {new_call_args});
            if rc != 0 {{ return Err(Error::Code(rc)); }}
            ctx.auto_sync();
            Ok(Self {{ data: out, ctx }})
        }}
    }}
}}

extern "C" {{
    fn {new_fn}(
        _: *mut futhark_context,
        _: *mut *mut {futhark_type},
        {new_extern_params}
    ) -> std::os::raw::c_int;
}}