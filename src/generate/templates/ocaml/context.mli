module Context: sig
  type t
  (** Futhark context *)

  val v: ?debug:bool -> ?log:bool -> ?profile:bool -> ?cache_file:string -> ?auto_sync:bool -> {extra_mli} unit -> t
  (** Create a new context *)
  
  val sync: t -> unit
  (** Sync the context, if auto_sync is enabled this is not needed *)
  
  val free: t -> unit
  (** Free the context *)
  
  val clear_caches: t -> unit
  (** Clear Futhark caches *)

  val get_error: t -> string option
  (** Get last error message or None *)

  val report: t -> string option
  val pause_profiling: t -> unit
  val unpause_profiling: t -> unit
end