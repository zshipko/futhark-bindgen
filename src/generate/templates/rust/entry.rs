impl Context {{
    pub fn {entry_name}(&self, {entry_params}) -> Result<{entry_return_type}, Error> {{
        {out_decl}
        let rc = unsafe {{
            futhark_entry_{entry_name}(self.context, {call_args})
        }};
        if rc != 0 {{ return Err(Error::Code(rc)); }}
    
        unsafe {{
            Ok({entry_return})
        }}
    }}
}}

extern "C" {{
    fn futhark_entry_{entry_name}(
        _: *mut futhark_context,
        {futhark_entry_params}
    ) -> std::os::raw::c_int;
}}