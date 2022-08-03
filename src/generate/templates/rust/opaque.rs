#[repr(C)]
#[allow(non_camel_case_types)]
struct {futhark_type} {{
    _private: [u8; 0]
}}

extern "C" {{
    fn {free_fn}(
        _: *mut futhark_context,
        _: *mut {futhark_type}
    ) -> std::os::raw::c_int;
}}

/// Futhark type
pub struct {rust_type}<'a> {{
    data: *mut {futhark_type},
    ctx: &'a Context,
}}

impl<'a> {rust_type}<'a> {{
    #[allow(unused)]
    fn from_ptr(ctx: &'a Context, data: *mut {futhark_type}) -> Self {{
        Self {{ ctx, data }}
    }}
}}

impl<'a> Drop for {rust_type}<'a> {{
    fn drop(&mut self) {{
        unsafe {{
            {free_fn}(self.ctx.context, self.data);
        }}
    }}
}}
