//! Tools for Lichta Calendar. Compatible with `#![no_std]`.

#![no_std]
extern crate alloc;

mod ngay_ta;
mod util;
pub use ngay_ta::NgayTa;
pub use time::Date;
pub use util::convert_date_to_lichta;
