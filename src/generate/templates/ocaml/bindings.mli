open! Signed
open! Unsigned

type error = 
  | InvalidShape of int * int 
  | NullPtr 
  | Code of int

exception Error of error
