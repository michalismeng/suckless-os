use spin::Mutex;
use uart_16550::SerialPort;

static mut SERIALPORT: SerialPort = unsafe { SerialPort::new(0x3F8) };

#[doc(hidden)]
pub static mut PORT_GUARD: Mutex<()> = Mutex::new(());

/// Acquire the kernel debugging context. This ensures exclusive access to
/// the debugging output device, the serial port.
/// ### Warnings
/// Possible deadlock, if you try to acquire the context inside
/// an interrupt handler, while already holding it in another execution path
/// on the same processor.
#[macro_export]
macro_rules! kdbg_ctx {
    ($($arg:stmt)+) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let _l = crate::light::kdebug::PORT_GUARD.lock();
            $( $arg )+
            crate::light::kdebug::PORT_GUARD.force_unlock();
        }
    }};
}

/// Write the given bytes to the debugging output device.
/// ### Safety
/// This function is unsafe, because it must be called within a
/// [`debugging context`](kdbg_ctx).
pub unsafe fn print(buffer: &[u8]) {
    for x in buffer {
        SERIALPORT.send(*x);
    }
}
