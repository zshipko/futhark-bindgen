open Example
open Bigarray

let () =
  (* binary_search *)
  let ctx = Context.v ~debug:true ~profile:true ~log:true () in
  let data = [| 1L; 2L; 3L; 4L; 5L; 7L; 8L |] in
  let arr = Array_i64_1d.of_array ctx [| 7 |] data in
  let index = binary_search ctx arr 6L in
  assert (Int64.equal index 5L);
  Array_i64_1d.free arr;

  (* mul2 *)
  let data2 = [| 1.0; 2.0; 3.0; 4.0; 5.0; 6.0 |] in
  let arr = Array_f64_2d.of_array ctx [| 2; 3 |] data2 in
  Printf.printf "%f\n" (data2.(0));
  let out = mul2 ctx arr in
  let out' = Array_f64_2d.get out in
  Printf.printf "%f\n" (Genarray.get out' [| 0; 0 |]);
  assert (Genarray.get out' [| 0; 0 |] = 2.0);
  assert (Genarray.get out' [| 1; 2 |] = 12.0);
  Array_f64_2d.free out;
  Array_f64_2d.free arr;

  let () = 
    try
      let _ = Array_f64_2d.get out in
      assert false
    with Error (UseAfterFree `array) -> print_endline "Detected use after free"
  in

  (* tup_mul *)
  let number = Number.v ctx 2.5 in
  let data3 = [| 0.0; 1.0; 2.0; 3.0; 4.0; 5.0; 6.0; 7.0; 8.0; 9.0 |] in
  let arr = Array_f32_1d.of_array ctx [| Array.length data3 |] data3 in
  let tup = Tup.v ctx number arr in
  let out = tup_mul ctx tup in
  let out' = Array_f32_1d.get_array1 out in
  for i = 0 to 9 do
    assert (out'.{i} = Array.get data3 i *. (Number.get_x (Tup.get_0 tup)))
  done;

  (* count_lines *)
  let text = "this\nis\na\ntest\n" in
  let arr = Array.init (String.length text) (fun i -> String.get text i |> int_of_char) in
  let data = Array1.of_array Int8_unsigned C_layout arr in
  let arr = Array_u8_1d.of_array1 ctx [|Array.length arr|] data in
  let n = count_lines ctx arr in
  assert (n = 4L);

  (* count_true *)
  let b = Array.init 10 (fun i -> if i mod 2 = 0 then 1 else 0) in
  let arr = Array_bool_1d.of_array ctx [| Array.length b |] b in
  let n = count_true ctx arr in
  assert (n = Int64.of_int @@ Array.fold_left (+) 0 b)
