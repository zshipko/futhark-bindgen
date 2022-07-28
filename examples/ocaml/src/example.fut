type number = {x: f32}

type~ tup = (number, []f32)
  
entry test (x: number) =
  x.x * 2

entry tup_mul (x: tup): []f32 =
  map (\a -> x.0.x * a) x.1

