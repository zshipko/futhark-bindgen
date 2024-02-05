  let v ctx {new_params} =
    check_use_after_free `context ctx.Context.context_free;
    let ptr = allocate (ptr void) null in
    let rc = Bindings.{new_fn} ctx.Context.handle ptr {new_call_args} in
    if rc <> 0 then raise (Error (Code rc));
    let _ = {new_tmp} in
    Context.auto_sync ctx;
    let opaque_ptr = !@ptr in
    let t = {{ opaque_ptr; opaque_ctx = ctx; opaque_free = false }} in
    Gc.finalise free t; t
