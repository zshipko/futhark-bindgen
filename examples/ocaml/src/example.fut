type number = {x: f32}

type~ tup = (number, []f32)

-- Check struct argument with numeric return
entry test (x: number) =
  x.x * 2

-- Check tuple argument with array return value
entry tup_mul (x: tup): []f32 =
  map (\a -> x.0.x * a) x.1

entry binary_search [n] (xs: [n]i64) (x: i64) : i64 =
  let (l, _) =
    loop (l, r) = (0, n-1) while l < r do
    let t = l + (r - l) / 2 in
    if x <= xs[t] then (l, t)
    else (t+1, r)
  in l

type option = #some i64 | #none

-- Check bool type
entry is_none (x: option): bool =
  match x
  case #some _ -> false
  case #none -> true

-- Check entry point with sum-type argument
entry option_get (x: option) : i64 =
  match x
  case #some x -> x
  case #none -> -1

-- Check entry point with sum-type argument and return value
entry return_option (x: option): option = x

entry mul2 (a: [][]f64) : [][]f64 =
  map (map (\b -> b * 2.0)) a
