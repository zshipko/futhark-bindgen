type number = {x: f32}

type~ tup = (number, []f32)
  
entry test (x: number) =
  x.x * 2

entry tup_mul (x: tup): []f32 =
  map (\a -> x.0.x * a) x.1
  
entry binary_search [n] (needle: i64) (haystack: [n]i64): i64 =
  let (L, _) =
    loop (L, R) = (0, n - 1) while L <= R do
      let m = i64.f64 (f64.floor ((f64.i64 L+ f64.i64 R) / 2)) in
      if haystack[m] < needle then
        (m + 1, R)
      else if haystack[m] > needle then
        (L, m - 1)
      else
        (m, 0)
  in L  
  
type option = #some i64 | #none

entry aaa (x: option): option = x