module {module_name}: sig
  type t
  (** Futhark array *)

  type kind = ({ocaml_elemtype}, {ba_elemtype}) Bigarray.kind
  (** The Bigarray kind that matches the correct element type for this array *)

  val kind: kind

  val shape: t -> int array
  (** Array shape *)

  val v: Context.t -> ({ocaml_elemtype}, {ba_elemtype}, Bigarray.c_layout) Bigarray.Genarray.t -> t
  (** Initialize an array with the data from the provided bigarray *)

  val values: t -> ({ocaml_elemtype}, {ba_elemtype}, Bigarray.c_layout) Bigarray.Genarray.t -> unit
  (** Load the values into the provided bigarray *)

  val values_array1: t -> ({ocaml_elemtype}, {ba_elemtype}, Bigarray.c_layout) Bigarray.Array1.t -> unit
  (** Similar to [values] but takes an [Array1] instead of [Genarray] *)

  val get: t -> ({ocaml_elemtype}, {ba_elemtype}, Bigarray.c_layout) Bigarray.Genarray.t
  (** Get a new bigarray with the values loaded *)

  val get_array1: t -> ({ocaml_elemtype}, {ba_elemtype}, Bigarray.c_layout) Bigarray.Array1.t
  (** Similar to [get] but returns an [Array1] *)

  val of_array: Context.t -> int array -> ({ocaml_elemtype}) array -> t
  (** Create [t] from an array of values *)

  val of_array1: Context.t -> int array ->  ({ocaml_elemtype}, {ba_elemtype}, Bigarray.c_layout) Bigarray.Array1.t-> t
  (** Create [t] from an [Array1] instead of [Genarray] *)

  val free: t -> unit
  (** Free the array *)
end
