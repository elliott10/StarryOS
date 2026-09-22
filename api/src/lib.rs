#![no_std]
#![feature(likely_unlikely)]
#![feature(bstr)]
#![allow(missing_docs)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

#[macro_use]
extern crate axlog;

extern crate alloc;

pub mod file;
pub mod io;
pub mod mm;
pub mod signal;
pub mod socket;
pub mod syscall;
pub mod task;
pub mod terminal;
pub mod time;
pub mod vfs;

#[cfg(feature = "sg2002-usb")]
const SG2002_USB_IRQ: usize = 0x1e;

#[cfg(feature = "sg2002-usb")]
fn init_sg2002_tinyusb() {
    if !tinyusb_sys::init_usb_device_msc() {
        warn!("TinyUSB init failed on SG2002");
        return;
    }

    if !axhal::irq::register(SG2002_USB_IRQ, || {
        tinyusb_sys::irq();
    }) {
        warn!("Failed to register TinyUSB IRQ handler: {}", SG2002_USB_IRQ);
        return;
    }

    axhal::irq::set_enable(SG2002_USB_IRQ, true);

    axtask::spawn_with_name(
        || loop {
            tinyusb_sys::usb_device_task();
            axtask::yield_now();
        },
        "tinyusb-task".into(),
    );

    info!("TinyUSB initialized on SG2002 (IRQ={}, MSC device ready)", SG2002_USB_IRQ);
}

/// Initialize.
pub fn init() {
    info!("Initialize VFS...");
    vfs::mount_all().expect("Failed to mount vfs");

    #[cfg(feature = "sg2002-usb")]
    init_sg2002_tinyusb();

    info!("Initialize /proc/interrupts...");
    axtask::register_timer_callback(|_| {
        time::inc_irq_cnt();
    });

    info!("Initialize alarm...");
    starry_core::time::spawn_alarm_task();
}
