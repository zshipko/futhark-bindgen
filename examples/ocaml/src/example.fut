-- Simple game of life implementation with a donut world.  Tested with
-- a glider running for four iterations.
--
-- http://rosettacode.org/wiki/Conway's_Game_of_Life
--
-- ==
-- input {
--   [[0, 0, 0, 0, 0],
--    [0, 0, 1, 0, 0],
--    [0, 0, 0, 1, 0],
--    [0, 1, 1, 1, 0],
--    [0, 0, 0, 0, 0]]
--   4
--   }
-- output {
--   [[0, 0, 0, 0, 0],
--    [0, 0, 0, 0, 0],
--    [0, 0, 0, 1, 0],
--    [0, 0, 0, 0, 1],
--    [0, 0, 1, 1, 1]]
--   }
-- input {
--   [[0, 0, 0, 0, 0],
--    [0, 0, 1, 0, 0],
--    [0, 0, 0, 1, 0],
--    [0, 1, 1, 1, 0],
--    [0, 0, 0, 0, 0]]
--   8
--   }
-- output {
--   [[1, 0, 0, 1, 1],
--    [0, 0, 0, 0, 0],
--    [0, 0, 0, 0, 0],
--    [0, 0, 0, 0, 1],
--    [1, 0, 0, 0, 0]]
--   }

def bint: bool -> i32 = i32.bool
def intb : i32 -> bool = bool.i32

def to_bool_board(board: [][]i32): [][]bool =
  map (\r  -> map intb r) board

def to_int_board(board: [][]bool): [][]i32 =
  map (\r  -> map bint r) board

entry all_neighbours [n][m] (world: [n][m]bool): [n][m]i32 =
    let ns  = map (rotate (-1)) world
    let ss  = map (rotate   1)  world
    let ws  = rotate      (-1)  world
    let es  = rotate        1   world
    let nws = map (rotate (-1)) ws
    let nes = map (rotate (-1)) es
    let sws = map (rotate   1)  ws
    let ses = map (rotate   1)  es
    in map3 (\(nws_r, ns_r, nes_r) (ws_r, world_r, es_r) (sws_r, ss_r, ses_r) ->
             map3 (\(nw,n,ne) (w,_,e) (sw,s,se) ->
                   bint nw + bint n + bint ne +
                   bint w + bint e +
                   bint sw + bint s + bint se)
             (zip3 nws_r ns_r nes_r) (zip3 ws_r world_r es_r) (zip3 sws_r ss_r ses_r))
            (zip3 nws ns nes) (zip3 ws world es) (zip3 sws ss ses)

entry iteration [n][m] (board: [n][m]bool): [n][m]bool =
  let lives = all_neighbours(board) in
  map2 (\(lives_r: []i32) (board_r: []bool)  ->
            map2 (\(neighbors: i32) (alive: bool): bool  ->
                      if neighbors < 2
                      then false
                      else if neighbors == 3 then true
                      else if alive && neighbors < 4 then true
                      else false)
                    lives_r board_r)
           lives board

entry life (int_board: [][]i32) (iterations: i32): [][]i32 =
  -- We accept the board as integers for convenience, and then we
  -- convert to booleans here.
  let board = to_bool_board int_board
  in to_int_board (loop board for _i < iterations do iteration board)

-- Random tests
type number = {x: f32}

type point = {x: f32, y: f32}

entry distance (a: point) (b: point) : f32 =
  let dx = b.x - a.x in
  let dy = b.y - a.y in
  f32.sqrt ((dx * dx) + (dy * dy))

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

-- Check input and output array with 2 dimensions
entry mul2 (a: [][]f64) : [][]f64 =
  map (map (\b -> b * 2.0)) a

-- Check lots of arguments
entry sinking1 (as: []i32) (bs: []i32) (cs: []i32) (ds: []i32) (es: []i32) =
  map5 (\a b c d e -> if a == 0 then 0 else b + c + d + e) as bs cs ds es

entry count_lines (input: []u8) : i64 =
  map (\x -> i64.bool (x == 10)) input |> i64.sum

entry count_true (input: []bool) : i64 =
  map i64.bool input |> i64.sum

