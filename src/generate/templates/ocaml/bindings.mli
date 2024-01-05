open! Signed
open! Unsigned

type error = 
  | InvalidShape of int * int 
  | NullPtr 
  | Code of int
  | UseAfterFree of [`context | `array | `opaque]

exception Error of error
