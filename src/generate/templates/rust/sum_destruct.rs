impl<'a> {rust_type}<'a> {{
    /// Destruct {rust_type} variant {variant_name}
    pub fn destruct_{variant_name}(&self) -> std::result::Result<{return_type}, Error> {{
        unsafe {{
{out_decls}
            let rc = {destruct_fn}(self.ctx.context{out_args}, self.data);
            if rc != 0 {{ return Err(Error::Code(rc)); }}
            self.ctx.auto_sync();
            Ok({return_expr})
        }}
    }}
}}

extern "C" {{
    fn {destruct_fn}(
        _: *mut futhark_context,
        {extern_params}_: *const {futhark_type}
    ) -> std::os::raw::c_int;
}}
