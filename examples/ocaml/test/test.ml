open Example
open Bigarray

let () =
  (* binary_search *)
  let ctx = Context.v ~debug:true ~profile:true ~log:true () in
  let data = [| 1L; 2L; 3L; 4L; 5L; 7L; 8L |] in
  let data' = Array1.of_array Int64 C_layout data |> genarray_of_array1 in
  let arr = Array_i64_1d.v ctx data' in
  let index = binary_search ctx arr 6L in
  assert (Int64.equal index 5L);
  Array_i64_1d.free arr;

  (* mul2 *)
  let data2 = Array2.of_array Float64 C_layout [| [| 1.0; 2.0; 3.0 |]; [| 4.0; 5.0; 6.0 |]|] |> genarray_of_array2 in
  let arr = Array_f64_2d.v ctx data2 in
  Printf.printf "%f\n" (Genarray.get data2 [| 0; 0 |]);
  let out = mul2 ctx arr in
  let out' = Array_f64_2d.get out in
  Printf.printf "%f\n" (Genarray.get out' [| 0; 0 |]);
  assert (Genarray.get out' [| 0; 0 |] = 2.0);
  Array_f64_2d.free out;
  Array_f64_2d.free arr;

  (* tup_mul *)
  let number = Number.v ctx 2.5 in
  let data3 = Array1.of_array Float32 C_layout [| 0.0; 1.0; 2.0; 3.0; 4.0; 5.0; 6.0; 7.0; 8.0; 9.0 |] |> genarray_of_array1 in
  let arr = Array_f32_1d.v ctx data3 in
  let tup = Tup.v ctx number arr in
  let out = tup_mul ctx tup in
  let out' = Array_f32_1d.get out |> array1_of_genarray in
  for i = 0 to 9 do
    assert (out'.{i} = Genarray.get data3 [| i |] *. (Number.get_x (Tup.get_0 tup)))
  done;

  (* count_lines *)
  let text = "this\nis\na\ntest\n" in
  let arr = Array.init (String.length text) (fun i -> String.get text i |> int_of_char) in
  let data = Array1.of_array Int8_unsigned C_layout arr |> genarray_of_array1 in
  let arr = Array_u8_1d.v ctx data in
  let n = count_lines ctx arr in
  assert (n = 4L);

  (* count_true *)
  let b = Array.init 10 (fun i -> if i mod 2 = 0 then 1 else 0) in
  let data = Array1.of_array Int8_unsigned C_layout b |> genarray_of_array1 in
  let arr = Array_bool_1d.v ctx data in
  let n = count_true ctx arr in
  assert (n = Int64.of_int @@ Array.fold_left (+) 0 b)