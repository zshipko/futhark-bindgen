  let get_{name} t =
    check_use_after_free `context t.opaque_ctx.Context.context_free;
    check_use_after_free `opaque t.opaque_free;
    let out = allocate_n ~count:1 {s} in
    let rc = Bindings.{project} t.opaque_ctx.Context.handle out t.opaque_ptr in
    if rc <> 0 then raise (Error (Code rc));
    let _ = t in 
    Context.auto_sync t.opaque_ctx;
    {out}

