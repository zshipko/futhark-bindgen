  let v ctx {new_params} =
    check_use_after_free `context ctx.Context.context_free;
    let ptr = allocate ~finalise:(free' ctx) (ptr void) null in
    let rc = Bindings.{new_fn} ctx.Context.handle ptr {new_call_args} in
    if rc <> 0 then raise (Error (Code rc));
    Context.auto_sync ctx;
    {{ opaque_ptr = ptr; opaque_ctx = ctx }}
