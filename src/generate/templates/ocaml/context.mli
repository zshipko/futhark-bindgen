module Context: sig
  type t
  val v: ?debug:bool -> ?log:bool -> ?profile:bool -> ?cache_file:string -> {extra_mli} unit -> t
  val sync: t -> unit
  val free: t -> unit
  val clear_caches: t -> unit
  val get_error: t -> string option
  val report: t -> string option
  val pause_profiling: t -> unit
  val unpause_profiling: t -> unit
end