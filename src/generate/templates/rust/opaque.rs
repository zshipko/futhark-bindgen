#[repr(C)]
#[allow(non_camel_case_types)]
struct {futhark_type} {{
    _private: [u8; 0]
}}

extern "C" {{
    fn futhark_free_opaque_{name}(
        _: *mut futhark_context,
        _: *mut {futhark_type}
    ) -> std::os::raw::c_int;
}}

pub struct {rust_type}<'a> {{
    data: *mut {futhark_type},
    ctx: *mut futhark_context,
    _t: std::marker::PhantomData<&'a ()>,
}}

impl<'a> {rust_type}<'a> {{
    #[allow(unused)]
    fn from_raw(ctx: *mut futhark_context, data: *mut {futhark_type}) -> Self {{
        Self {{ ctx, data, _t: std::marker::PhantomData }}
    }}
}}

impl<'a> Drop for {rust_type}<'a> {{
    fn drop(&mut self) {{
        unsafe {{
            futhark_free_opaque_{name}(self.ctx, self.data);
        }}
    }}
}}
