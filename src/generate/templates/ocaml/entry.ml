let {name} ctx {entry_params} =
{out_decl}
  let rc = Bindings.futhark_entry_{name} ctx.Context.handle {call_args} in
  if rc <> 0 then raise (Error (Code rc));
  ({out_return})
