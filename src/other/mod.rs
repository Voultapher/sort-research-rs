#[cfg(feature = "rust_radsort")]
pub mod rust_radsort;

// Call simdsort sort via FFI.
#[cfg(feature = "cpp_simdsort")]
pub mod cpp_simdsort;

// Call highwaysort sort via FFI.
#[cfg(feature = "cpp_highwaysort")]
pub mod cpp_highwaysort;

#[cfg(feature = "evolution")]
pub mod sort_evolution;

pub mod partition;
