  type t = opaque
  let t = Bindings.{name}
  let _ = t

  let free t = 
    ignore (Bindings.{free_fn} t.opaque_ctx.Context.handle t.opaque_ptr)

  let of_raw ctx ptr =
    if is_null ptr then raise (Error NullPtr);
    let t = {{ opaque_ptr = ptr; opaque_ctx = ctx }} in
    Gc.finalise free t; t

  let _ = of_raw


