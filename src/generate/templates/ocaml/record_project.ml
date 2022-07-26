  let get_{name} t =
    let out = allocate_n ~count:1 {s} in
    let rc = Bindings.{project} t.opaque_ctx.Context.handle out t.opaque_ptr in
    if rc <> 0 then raise (Error (Code rc));
    Context.auto_sync t.opaque_ctx;
    {out}

