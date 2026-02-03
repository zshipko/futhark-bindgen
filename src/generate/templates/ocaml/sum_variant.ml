  let variant t =
    check_use_after_free `context t.opaque_ctx.Context.context_free;
    Bindings.{variant_fn} t.opaque_ctx.Context.handle !@(t.opaque_ptr)
