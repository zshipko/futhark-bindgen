  type t = opaque
  let t = Bindings.{name}
  let _ = t

  let free t = 
    if not t.opaque_free && not t.opaque_ctx.Context.context_free then
      let () = ignore (Bindings.{free_fn} t.opaque_ctx.Context.handle t.opaque_ptr) in
      t.opaque_free <- true

  let of_ptr ctx ptr =
    if is_null ptr then raise (Error NullPtr);
    let t = {{ opaque_ptr = ptr; opaque_ctx = ctx; opaque_free = false }} in
    set_managed ptr t;
    Gc.finalise free t; t

  let _ = of_ptr
