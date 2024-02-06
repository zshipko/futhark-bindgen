open Bigarray

module Context = struct
  [@@@ocaml.warning "-69"]
  type t = {{ handle: unit ptr; config: unit ptr; cache_file: string option; auto_sync: bool; mutable context_free: bool }}
  [@@@ocaml.warning "+69"]

  let free t =
    if not t.context_free then
      let () = ignore (Bindings.futhark_context_sync t.handle) in
      let () = ignore (Bindings.futhark_context_free t.handle) in
      let () = ignore (Bindings.futhark_context_config_free t.config) in
      t.context_free <- true

  let v ?(debug = false) ?(log = false) ?(profile = false) ?cache_file ?(auto_sync = true) {extra_param} () =
    let config = Bindings.futhark_context_config_new () in
    if is_null config then raise (Error NullPtr);
    Bindings.futhark_context_config_set_debugging config (if debug then 1 else 0);
    Bindings.futhark_context_config_set_profiling config (if profile then 1 else 0);
    Bindings.futhark_context_config_set_logging config (if log then 1 else 0);
    {extra_line}
    Option.iter (Bindings.futhark_context_config_set_cache_file config) cache_file;
    let handle = Bindings.futhark_context_new config in
    if is_null handle then 
      let () = ignore @@ Bindings.futhark_context_config_free config in
      raise (Error NullPtr)
    else
      let t = {{ handle; config; cache_file; auto_sync; context_free = false }} in
      set_managed handle t; 
      let () = Gc.finalise free t in
      t

  let sync t =
    check_use_after_free `context t.context_free;
    let rc = Bindings.futhark_context_sync t.handle in
    if rc <> 0 then raise (Error (Code rc))

  let auto_sync t =
    if t.auto_sync then sync t
  
  let clear_caches t =
    check_use_after_free `context t.context_free;
    let rc = Bindings.futhark_context_clear_caches t.handle in
    if rc <> 0 then raise (Error (Code rc))

  let string_opt_of_ptr ptr = 
    if is_null ptr then None
    else
      let len = Bindings.strlen ptr |> Unsigned.Size_t.to_int in
      let s = String.init len (fun i -> !@(ptr +@ i)) in
      let () = Bindings.free (coerce (Ctypes.ptr Ctypes.char) (Ctypes.ptr void) ptr) in Some s

  let get_error t = 
    check_use_after_free `context t.context_free;
    let ptr = Bindings.futhark_context_get_error t.handle in string_opt_of_ptr ptr

  let report t = 
    check_use_after_free `context t.context_free;
    let ptr = Bindings.futhark_context_report t.handle in string_opt_of_ptr ptr

  let pause_profiling t = 
    check_use_after_free `context t.context_free;
    Bindings.futhark_context_pause_profiling t.handle

  let unpause_profiling t =
    check_use_after_free `context t.context_free;
    Bindings.futhark_context_unpause_profiling t.handle
end

[@@@ocaml.warning "-34"]
[@@@ocaml.warning "-69"]
type futhark_array = {{ mutable ptr: unit ptr ptr; shape: int array; ctx: Context.t }}
type opaque = {{ mutable opaque_ptr: unit ptr ptr; opaque_ctx: Context.t }}
[@@@ocaml.warning "+34"]
[@@@ocaml.warning "+69"]
