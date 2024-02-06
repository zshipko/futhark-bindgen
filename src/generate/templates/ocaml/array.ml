module {module_name} = struct
  type t = futhark_array

  type kind = ({ocaml_elemtype}, {ba_elemtype}) Bigarray.kind
  
  let kind = {ba_kind}

  let free t =
    if not t.array_free && not t.ctx.Context.context_free then
      let () = ignore (Bindings.futhark_free_{elemtype}_{rank}d t.ctx.Context.handle t.ptr) in
      t.array_free <- true

  let cast x =
    coerce (ptr void) (ptr {ocaml_ctype}) (to_voidp x)
  
  let v ctx ba =
    check_use_after_free `context ctx.Context.context_free;
    let dims = Genarray.dims ba in
    let ptr = Bindings.futhark_new_{elemtype}_{rank}d ctx.Context.handle (cast @@ bigarray_start genarray ba) {dim_args} in
    if is_null ptr then raise (Error NullPtr);
    Context.auto_sync ctx;
    let t = {{ ptr; ctx; shape = dims; array_free = false }} in
    set_managed ptr t; 
    Gc.finalise free t; t

  let values t ba =
    check_use_after_free `context t.ctx.Context.context_free;
    check_use_after_free `array t.array_free;
    let dims = Genarray.dims ba in
    let a = Array.fold_left ( * ) 1 t.shape in
    let b = Array.fold_left ( * ) 1 dims in
    if (a <> b) then raise (Error (InvalidShape (a, b)));
    let rc = Bindings.futhark_values_{elemtype}_{rank}d t.ctx.Context.handle t.ptr (cast @@ bigarray_start genarray ba) in
    let _ = ba in
    Context.auto_sync t.ctx;
    if rc <> 0 then raise (Error (Code rc))

  let values_array1 t ba =
    let ba = genarray_of_array1 ba in
    let ba = reshape ba t.shape in
    values t ba

  let get t =
    let dims = t.shape in
    let g = Genarray.create kind C_layout dims in
    values t g;
    g

  let get_array1 t =
    let len = Array.fold_left ( * ) 1 t.shape in
    let g = Array1.create kind C_layout len in
    values_array1 t g;
    g

  let shape t = t.shape

  let of_array1 ctx dims arr =
    let len = Array.fold_left ( * ) 1 dims in
    assert (len = Array1.dim arr);
    let g = genarray_of_array1 arr in
    let g = reshape g dims in
    v ctx g

  let of_array ctx dims arr =
    let arr = Array1.of_array kind C_layout arr in
    of_array1 ctx dims arr

  let ptr_shape ctx ptr =
    let s = Bindings.futhark_shape_{elemtype}_{rank}d ctx ptr in
    Array.init {rank} (fun i -> Int64.to_int !@ (s +@ i))

  let of_ptr ctx ptr =
    check_use_after_free `context ctx.Context.context_free;
    if is_null ptr then raise (Error NullPtr);
    let shape = ptr_shape ctx.Context.handle ptr in
    let t = {{ ptr; ctx; shape; array_free = false }} in
    set_managed ptr t; 
    Gc.finalise free t; t
    
  let _ = of_ptr
end

