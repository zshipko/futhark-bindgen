#[derive(Debug)]
pub enum Error {{
    Code(std::os::raw::c_int),
    NullPtr,
    InvalidShape,
}}

impl std::fmt::Display for Error {{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {{
        match self {{
            Error::Code(code) => write!(fmt, "Futhark error code: {{code}}"),
            Error::NullPtr => write!(fmt, "NULL pointer encountered"),
            Error::InvalidShape => write!(fmt, "Invalid image shape"),
        }}
    }} 
}}

impl std::error::Error for Error {{}}

#[derive(Debug, Clone)]
pub struct Options {{
    debug: bool,
    profile: bool,
    logging: bool,
    num_threads: u32,
    cache_file: std::option::Option<std::ffi::CString>,
    device: std::option::Option<std::ffi::CString>,
    auto_sync: bool,
}}

impl Default for Options {{
    fn default() -> Self {{
        Options::new()
    }}
}}

impl Options {{
    /// Create new `Options` with default settings
    pub fn new() -> Self {{
        Options {{
            debug: false,
            profile: false,
            logging: false,
            num_threads: 0,
            cache_file: None,
            device: None,
            auto_sync: true,
        }}
    }}

    /// Enable debug
    pub fn debug(mut self) -> Self {{
        self.debug = true;
        self
    }}

    /// Enable profiling
    pub fn profile(mut self) -> Self {{
        self.profile = true;
        self
    }}

    /// Enable logging
    pub fn log(mut self) -> Self {{
        self.logging = true;
        self
    }}

    /// Set Futhark cache file
    pub fn cache_file(mut self, s: impl AsRef<str>) -> Self {{
        self.cache_file = Some(std::ffi::CString::new(s.as_ref()).expect("Invalid cache file"));
        self
    }}

    pub fn auto_sync(mut self, sync: bool) -> Self {{
        self.auto_sync = sync;
        self
    }}


    {backend_options}
}}

/// Futhark context
pub struct Context {{
    config: *mut futhark_context_config,
    context: *mut futhark_context,
    auto_sync: bool,
    _cache_file: std::option::Option<std::ffi::CString>,
}}

impl Context {{
    /// Create a new context with default options
    pub fn new() -> std::result::Result<Self, Error> {{
        unsafe {{
            let config = futhark_context_config_new();
            if config.is_null() {{ return Err(Error::NullPtr) }}
            let context = futhark_context_new(config);
            if context.is_null() {{
                futhark_context_config_free(config);
                return Err(Error::NullPtr);
            }}
            Ok(Context {{ config, context, auto_sync: true, _cache_file: None }})
        }}
    }}

    /// Create a new context with custom options
    pub fn new_with_options(options: Options) -> std::result::Result<Self, Error> {{
        unsafe {{
            let config = futhark_context_config_new();
            if config.is_null() {{ return Err(Error::NullPtr) }}

            futhark_context_config_set_debugging(config, options.debug as std::os::raw::c_int);
            futhark_context_config_set_profiling(config, options.profile as std::os::raw::c_int);
            futhark_context_config_set_logging(config, options.logging as std::os::raw::c_int);

            if let Some(c) = &options.cache_file {{
                futhark_context_config_set_cache_file(config, c.as_ptr());
            }}

            {configure_num_threads}
            {configure_set_device}

            let context = futhark_context_new(config);
            if context.is_null() {{
                futhark_context_config_free(config);
                return Err(Error::NullPtr);
            }}
            Ok(Context {{ config, context, auto_sync: options.auto_sync, _cache_file: options.cache_file }})
        }}
    }}

    /// Sync the context, if `auto_sync` is enabled this shouldn't be needed
    pub fn sync(&self) {{
        unsafe {{ futhark_context_sync(self.context); }}
    }}

    /// Sync if `auto_sync` is enabled, otherwise this is a noop
    pub fn auto_sync(&self) {{
        if self.auto_sync {{
            self.sync();
        }}
    }}

    /// Clear Futhark caches
    pub fn clear_caches(&self) -> std::result::Result<(), Error> {{
        let rc = unsafe {{
            futhark_context_clear_caches(self.context)
        }};
        if rc != 0 {{ return Err(Error::Code(rc)) }}
        Ok(())
    }}

    /// Pause Futhark profiling
    pub fn pause_profiling(&self) {{
        unsafe {{
            futhark_context_pause_profiling(self.context);
        }}
    }}

    /// Resume profiling
    pub fn unpause_profiling(&self) {{
        unsafe {{
            futhark_context_unpause_profiling(self.context);
        }}
    }}

    /// Get the last error message or None
    pub fn get_error(&self) -> std::option::Option<String> {{
        unsafe {{
            let s = futhark_context_get_error(self.context);
            if s.is_null() {{ return None }}
            let r = std::ffi::CStr::from_ptr(s).to_string_lossy().to_string();
            free(s as *mut _);
            Some(r)
        }}
    }}

    pub fn report(&self) -> std::option::Option<String> {{
        unsafe {{
            let s = futhark_context_report(self.context);
            if s.is_null() {{ return None }}
            let r = std::ffi::CStr::from_ptr(s).to_string_lossy().to_string();
            free(s as *mut _);
            Some(r)
        }}
    }}
}}

impl Drop for Context {{
    fn drop(&mut self) {{
        unsafe {{
            futhark_context_sync(self.context);
            futhark_context_free(self.context);
            futhark_context_config_free(self.config);
        }}
    }}
}}

#[repr(C)]
#[allow(non_camel_case_types)]
struct futhark_context_config {{
    _private: [u8; 0]
}}

#[repr(C)]
#[allow(non_camel_case_types)]
struct futhark_context {{
    _private: [u8; 0]
}}

extern "C" {{
    fn futhark_context_config_new() -> *mut futhark_context_config;
    fn futhark_context_config_free(
        _: *mut futhark_context_config
    );
    fn futhark_context_config_set_debugging(
        _: *mut futhark_context_config,
        _: std::os::raw::c_int
    );

    fn futhark_context_config_set_profiling(
        _: *mut futhark_context_config,
        _: std::os::raw::c_int
    );

    fn futhark_context_config_set_logging(
        _: *mut futhark_context_config,
        _: std::os::raw::c_int
    );

    fn futhark_context_config_set_cache_file(
        _: *mut futhark_context_config,
        _: *const std::os::raw::c_char,
    );

    fn futhark_context_new(
        _: *mut futhark_context_config
    ) -> *mut futhark_context;

    fn futhark_context_free(
        _: *mut futhark_context
    );

    fn futhark_context_sync(
        _: *mut futhark_context,
    ) -> std::os::raw::c_int;

    fn futhark_context_clear_caches(
        _: *mut futhark_context,
    ) -> std::os::raw::c_int;

    fn futhark_context_pause_profiling(
        _: *mut futhark_context
    );

    fn futhark_context_unpause_profiling(
        _: *mut futhark_context
    );

    fn futhark_context_get_error(
        _: *mut futhark_context
    ) -> *mut std::os::raw::c_char;

    fn futhark_context_report(
        _: *mut futhark_context
    ) -> *mut std::os::raw::c_char;

    fn free(_: *mut std::ffi::c_void);

    {backend_extern_functions}
}}
