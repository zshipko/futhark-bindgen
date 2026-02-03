impl<'a> {rust_type}<'a> {{
    /// Get the variant index of this sum type
    pub fn variant(&self) -> i32 {{
        unsafe {{ {variant_fn}(self.ctx.context, self.data) }}
    }}
}}

extern "C" {{
    fn {variant_fn}(_: *mut futhark_context, _: *const {futhark_type}) -> std::os::raw::c_int;
}}
