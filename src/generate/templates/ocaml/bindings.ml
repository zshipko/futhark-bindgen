open Ctypes
open! Unsigned
open! Signed

module Bindings = struct
  external _stub: unit -> unit = "futhark_context_new"

  let fn = Foreign.foreign ~release_runtime_lock:true
  let context = typedef (ptr void) "context"
  let context_config = typedef (ptr void) "context_config"
  let futhark_context_new = fn "futhark_context_new" (context_config @-> returning context)
  let futhark_context_free = fn "futhark_context_free" (context @-> returning int)
  let futhark_context_sync = fn "futhark_context_sync" (context @-> returning int)
  let futhark_context_config_new = fn "futhark_context_config_new" (void @-> returning context_config)
  let futhark_context_config_free = fn "futhark_context_config_free" (context_config @-> returning int)
  let futhark_context_config_set_profiling = fn "futhark_context_config_set_profiling" (context_config @-> int @-> returning void)
  let futhark_context_config_set_debugging = fn "futhark_context_config_set_debugging" (context_config @-> int @-> returning void)
  let futhark_context_config_set_logging = fn "futhark_context_config_set_logging" (context_config @-> int @-> returning void)
  let futhark_context_config_set_cache_file = fn "futhark_context_config_set_cache_file" (context_config @-> string @-> returning void)
  let futhark_context_pause_profiling = fn "futhark_context_pause_profiling" (context @-> returning void)
  let futhark_context_unpause_profiling = fn "futhark_context_unpause_profiling" (context @-> returning void)
  let futhark_context_clear_caches = fn "futhark_context_clear_caches" (context @-> returning int)
  let futhark_context_get_error = fn "futhark_context_get_error" (context @-> returning (ptr char))
  let futhark_context_report = fn "futhark_context_report" (context @-> returning (ptr char))
  let free = fn "free" (ptr void @-> returning void)
  let strlen = fn "strlen" (ptr char @-> returning size_t)

{generated_foreign_functions}
end

type error =
  | InvalidShape of int * int
  | NullPtr
  | Code of int
  | UseAfterFree of [`context | `array | `opaque]

exception Error of error

let check_use_after_free t b = if b then raise (Error (UseAfterFree t))

let () = Printexc.register_printer (function
  | Error (InvalidShape (a, b)) -> Some (Printf.sprintf "futhark error: invalid shape, expected %d but got %d" a b)
  | Error NullPtr -> Some "futhark error: null pointer"
  | Error (Code c) -> Some (Printf.sprintf "futhark error: code %d" c) 
  | Error (UseAfterFree `context) -> Some "futhark: context used after beeing freed"
  | Error (UseAfterFree `array) -> Some "futhark: array used after beeing freed"
  | Error (UseAfterFree `opaque) -> Some "futhark: opqaue value used after beeing freed"
  | _ -> None)


