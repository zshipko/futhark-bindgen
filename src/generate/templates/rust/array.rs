#[repr(C)]
#[allow(non_camel_case_types)]
struct {futhark_type} {{
    _private: [u8; 0]
}}

pub struct {rust_type}<'a> {{
    ptr: *mut {futhark_type},
    pub shape: [i64; {rank}],
    ctx: &'a Context,
}}

impl<'a> {rust_type}<'a> {{
    pub fn new(ctx: &'a Context, dims: [i64; {rank}], data: impl AsRef<[{elemtype}]>) -> std::result::Result<Self, Error> {{
        let size: i64 = dims.iter().product();
        let data = data.as_ref();
        if data.len() as i64 != size {{
            return Err(Error::InvalidShape)
        }}
        let ptr = unsafe {{
            {new_fn}(ctx.context, data.as_ptr(), {dim_params})
        }};
        if ptr.is_null() {{ return Err(Error::NullPtr); }}
        ctx.auto_sync();
        Ok(Self {{
            ptr: ptr as *mut _,
            shape: dims,
            ctx,
        }})
    }}

    pub fn shape(&self) -> &[i64; {rank}] {{
        &self.shape
    }}

    pub fn values(&self, mut data: impl AsMut<[{elemtype}]>) -> std::result::Result<(), Error> {{
        let size: i64 = self.shape.iter().product();
        let data = data.as_mut();
        if data.len() as i64 != size {{
            return Err(Error::InvalidShape);
        }}
        let rc = unsafe {{
            futhark_values_{elemtype}_{rank}d(self.ctx.context, self.ptr, data.as_mut_ptr())
        }};
        if rc != 0 {{
            return Err(Error::Code(rc));
        }}
        self.ctx.auto_sync();
        Ok(())
    }}

    pub fn get(&self) -> std::result::Result<Vec<{elemtype}>, Error> {{
        let size: i64 = self.shape.iter().product();
        let mut vec = vec![0_{elemtype}; size as usize];
        self.values(&mut vec)?;
        Ok(vec)
    }}


    #[allow(unused)]
    fn from_ptr(ctx: &'a Context, ptr: *mut {futhark_type}) -> Self {{
        let len_ptr = unsafe {{ futhark_shape_{elemtype}_{rank}d(ctx.context, ptr) }};
        let mut shape = [0i64; {rank}];
        unsafe {{
            for (i, s) in shape.iter_mut().enumerate() {{
                *s = *len_ptr.add(i);
            }}
        }}
        Self {{ ctx, shape, ptr }}
    }}
}}


impl<'a> Drop for {rust_type}<'a> {{
    fn drop(&mut self){{
        unsafe {{
            futhark_free_{elemtype}_{rank}d(self.ctx.context, self.ptr as *mut _);
        }}
    }}
}}

#[allow(unused)]
extern "C" {{
    fn {shape_fn}(
        _: *mut futhark_context,
        _: *mut {futhark_type}
    ) -> *const i64;

    fn {new_fn}(
        _: *mut futhark_context,
        _: *const {elemtype},
        {new_dim_args}
    ) -> *mut {futhark_type};

    fn {free_fn}(
        _: *mut futhark_context,
        _: *mut {futhark_type}
    ) -> std::os::raw::c_int;

    fn {values_fn}(
        _: *mut futhark_context,
        _: *mut {futhark_type},
        _: *mut {elemtype}
    ) -> std::os::raw::c_int;
}}
