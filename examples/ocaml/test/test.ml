open Example

let () =
  let ctx = Context.v () in
  let number = Number.v ctx 2.5 in
  let out = Ctypes.allocate Ctypes.float 0.0 in
  let () = Entry.test ctx ~out number in 
  assert (Ctypes.(!@out) = 5.0)
