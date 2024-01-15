#[cfg(not(target_os = "linux"))]
pub mod wasabi;
#[cfg(not(target_os = "linux"))]
pub use wasabi as os;
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux as os;

pub use os::draw_point;
