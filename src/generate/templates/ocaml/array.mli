module {module_name}: sig
  type t
  type kind = ({ocaml_elemtype}, {ba_elemtype}) Bigarray.kind
  val kind: kind
  val shape: t -> int array
  val v: Context.t -> ({ocaml_elemtype}, {ba_elemtype}, Bigarray.c_layout) Bigarray.Genarray.t -> t
  val values: t -> ({ocaml_elemtype}, {ba_elemtype}, Bigarray.c_layout) Bigarray.Genarray.t -> unit
  val get: t -> ({ocaml_elemtype}, {ba_elemtype}, Bigarray.c_layout) Bigarray.Genarray.t
  val free: t -> unit
end
