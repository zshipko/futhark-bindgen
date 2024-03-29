  type t = opaque
  let t = Bindings.{name}
  let _ = t

  let free' ctx ptr = 
    let is_null = Ctypes.is_null ptr || Ctypes.is_null (!@ptr) in
    if not ctx.Context.context_free && not is_null then
      let () = ignore (Bindings.{free_fn} ctx.Context.handle (!@ptr)) in
      ptr <-@ Ctypes.null

  let of_ptr ctx ptr =
    if is_null ptr then raise (Error NullPtr);
    {{ opaque_ptr = allocate ~finalise:(free' ctx) (Ctypes.ptr Ctypes.void) ptr; opaque_ctx = ctx }}

  let free t = free' t.opaque_ctx t.opaque_ptr 

  let _ = of_ptr
