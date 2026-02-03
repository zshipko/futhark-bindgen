  let destruct_{variant_name} t =
    check_use_after_free `context t.opaque_ctx.Context.context_free;
{params}
    let rc = Bindings.{destruct_fn} t.opaque_ctx.Context.handle {out_ptrs} !@(t.opaque_ptr) in
    if rc <> 0 then raise (Error (Code rc));
    Context.auto_sync t.opaque_ctx;
    ({returns})
