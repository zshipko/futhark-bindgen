type number = {x: f32}

type tup = (number, [3]f32)
  
entry test (x: number) =
  x.x * 2

entry tup_mul (x: tup) =
  map (\a -> a * x.0.x) x.1
