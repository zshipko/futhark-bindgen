open Example

let () =
  let ctx = Context.v ~debug:true ~profile:true ~log:true () in
  let number = Number.v ctx 2.5 in
  let out = Ctypes.allocate Ctypes.float 0.0 in
  let () = Entry.test ctx ~out number in 
  assert (Ctypes.(!@out) = 5.0);

  let init = [| 1.0; 2.0; 3.0 |] in  
  let ba = Bigarray.Array1.of_array Bigarray.Float32 Bigarray.C_layout init in
  let ba = Bigarray.genarray_of_array1 ba in
  let arr = Array_f32_1d.of_bigarray ctx ba in
  let tup = Tup.v ctx number arr in
  let out = Array_f32_1d.v ctx [|3|] in
  let () = Entry.tup_mul ctx ~out tup in
  let () = Context.sync ctx in
  let ba = Bigarray.Genarray.create Bigarray.Float32 Bigarray.C_layout [| 3 |] in
  let () = Array_f32_1d.values out ba in
  let test: float array = Array.init 3 (fun i -> Bigarray.Genarray.get ba [|i|]) in
  Array.iter (Printf.printf "%f\n") test;
  Array.iter (Printf.printf "%f\n") init;
  assert (Array.for_all2 (fun a b -> Float.equal a (b *. 2.5)) test init)
