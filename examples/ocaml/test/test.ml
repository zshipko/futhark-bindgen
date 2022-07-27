open Example
open Bigarray

let () =
  let ctx = Context.v ~debug:true ~profile:true ~log:true () in
  let number = Number.v ctx 2.5 in
  let out = Entry.test ctx number in 
  assert (out = 5.0);
  let init = [| 1.0; 2.0; 3.0 |] in  
  let ba = Array1.of_array Float32 C_layout init |> genarray_of_array1 in
  let arr = Array_f32_1d.v ctx ba in
  let tup = Tup.v ctx number arr in
  let out = Entry.tup_mul ctx tup in
  let () = Array_f32_1d.values out ba in
  let test: float array = Array.init 3 (fun i -> Genarray.get ba [|i|]) in
  Array.iter (Printf.printf "%f\n") test;
  Array.iter (Printf.printf "%f\n") init;
  assert (Array.for_all2 (fun a b -> Float.equal a (b *. 2.5)) test init);
  Array_f32_1d.free arr;
  Array_f32_1d.free out;
  Tup.free tup

