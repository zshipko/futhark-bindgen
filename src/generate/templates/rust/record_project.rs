impl<'a> {rust_type}<'a> {{
    pub fn get_{field_name}(&self) -> Result<{rust_field_type}, Error> {{
        let mut out = std::mem::MaybeUninit::zeroed();
        let rc = unsafe {{
            futhark_project_opaque_{name}_{field_name}(
                self.ctx.context,
                out.as_mut_ptr(),
                self.data
            )
        }};
        if rc != 0 {{ return Err(Error::Code(rc)); }}
        self.ctx.auto_sync();
        let out = unsafe {{ out.assume_init() }};
        {output}
    }}
}}

extern "C" {{
    fn {project_fn}(
        _: *mut futhark_context,
        _: *mut {futhark_field_type},
        _: *const {futhark_type}
    ) -> std::os::raw::c_int;
}}
