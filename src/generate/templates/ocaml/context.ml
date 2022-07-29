open Bigarray

module Context = struct
  type t = {{ handle: unit ptr; config: unit ptr; cache_file: string option }}

  let free t = 
    ignore (Bindings.futhark_context_sync t.handle);
    ignore (Bindings.futhark_context_free t.handle);
    ignore (Bindings.futhark_context_config_free t.config)


  let v ?(debug = false) ?(log = false) ?(profile = false) ?cache_file {extra_param} () =
    let config = Bindings.futhark_context_config_new () in
    if is_null config then raise (Error NullPtr);
    Bindings.futhark_context_config_set_debugging config (if debug then 1 else 0);
    Bindings.futhark_context_config_set_profiling config (if profile then 1 else 0);
    Bindings.futhark_context_config_set_logging config (if log then 1 else 0);
    {extra_line}
    Option.iter (Bindings.futhark_context_config_set_cache_file config) cache_file;
    let handle = Bindings.futhark_context_new config in
    if is_null handle then (ignore @@ Bindings.futhark_context_config_free config; raise (Error NullPtr));
    let t = {{ handle; config; cache_file; }} in
    Gc.finalise free t; t

  let sync t =
    let rc = Bindings.futhark_context_sync t.handle in
    if rc <> 0 then raise (Error (Code rc))
  
  let clear_caches t =
    let rc = Bindings.futhark_context_clear_caches t.handle in
    if rc <> 0 then raise (Error (Code rc))

  let string_opt_of_ptr ptr = 
    if is_null ptr then None
    else
      let len = Bindings.strlen ptr |> Unsigned.Size_t.to_int in
      let s = String.init len (fun i -> !@(ptr +@ i)) in
      let () = Bindings.free (coerce (Ctypes.ptr Ctypes.char) (Ctypes.ptr void) ptr) in Some s

  let get_error t = let ptr = Bindings.futhark_context_get_error t.handle in string_opt_of_ptr ptr

  let report t = let ptr = Bindings.futhark_context_report t.handle in string_opt_of_ptr ptr

  let pause_profiling t = Bindings.futhark_context_pause_profiling t.handle
  let unpause_profiling t = Bindings.futhark_context_unpause_profiling t.handle
end

type futhark_array = {{ ptr: unit ptr; shape: int array; ctx: Context.t }}
type opaque = {{ opaque_ptr: unit ptr; opaque_ctx: Context.t }}
