#![no_main]
#![no_std]
use cortex_m_rt::entry;
use panic_halt as _;
#[entry]
fn main() -> ! {
	loop{}
}
