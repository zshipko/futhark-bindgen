let {name} ctx {entry_params} =
  check_use_after_free `context ctx.Context.context_free;
{out_decl}
  let rc = Bindings.futhark_entry_{name} ctx.Context.handle {call_args} in
  if rc <> 0 then raise (Error (Code rc));
  ({out_return})
