type number = {x: f32}

type tup = (number, [3]f32)
  
entry test (x: number) =
  x.x * 2

entry tup_mul (x: tup): [3]f32 =
  map (\a -> x.0.x * a) x.1
