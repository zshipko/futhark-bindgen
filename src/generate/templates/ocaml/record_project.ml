  let get_{name} t =
    check_use_after_free `context t.opaque_ctx.Context.context_free;
    let out = allocate_n ~count:1 {s} in
    let rc = Bindings.{project} t.opaque_ctx.Context.handle out (get_opaque_ptr t) in
    if rc <> 0 then raise (Error (Code rc));
    Context.auto_sync t.opaque_ctx;
    {out}

