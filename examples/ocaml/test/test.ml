open Example
open Bigarray

let () =
  let ctx = Context.v ~debug:true ~profile:true ~log:true () in
  let data = [| 1L; 2L; 3L; 4L; 5L; 7L; 8L |] in
  let data' = Array1.of_array Int64 C_layout data |> genarray_of_array1 in
  let arr = Array_i64_1d.v ctx data' in
  let index = binary_search ctx arr 6L in
  assert (Int64.equal index 5L);
  Array_i64_1d.free arr;

  let data2 = Array2.of_array Float64 C_layout [| [| 1.0; 2.0; 3.0 |]; [| 4.0; 5.0; 6.0 |]|] |> genarray_of_array2 in
  let arr = Array_f64_2d.v ctx data2 in
  Printf.printf "%f\n" (Genarray.get data2 [| 0; 0 |]);
  let out = mul2 ctx arr in
  let out' = Array_f64_2d.get out in
  Printf.printf "%f\n" (Genarray.get out' [| 0; 0 |]);
  assert (Genarray.get out' [| 0; 0 |] = 2.0);
  Array_f64_2d.free out;
  Array_f64_2d.free arr

