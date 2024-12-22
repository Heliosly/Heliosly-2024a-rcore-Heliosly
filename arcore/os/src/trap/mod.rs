//! Trap handling functionality
//!
//! For rCore, we have a single trap entry point, namely `__alltraps`. At
//! initialization in [`init()`], we set the `stvec` CSR to point to it.
//!
//! All traps go through `__alltraps`, which is defined in `trap.S`. The
//! assembly language code does just enough work restore the kernel space
//! context, ensuring that Rust code safely runs, and transfers control to
//! [`trap_handler()`].
//!
//! It then calls different functionality based on what exactly the exception
//! was. For example, timer interrupts trigger task preemption, and syscalls go
//! to [`syscall()`].

mod context;

/* use crate::syscall::syscall; */
use crate::task::{
    check_signals_of_current, current_add_signal, current_trap_cx, /* current_trap_cx_user_va, */
    /* current_user_token, */suspend_current_and_run_next, SignalFlags,
};
use crate::timer::{check_timer, set_next_trigger};
use core::arch::{/* asm, */ global_asm};
use core::panic;

use riscv::register::{satp, sepc};
use riscv::register::sstatus::FS;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};
use crate::syscall::syscall;
global_asm!(include_str!("trap.S"));
extern "C" {
    fn __trap_from_user();
    fn __trap_from_kernel();
}
/// Initialize trap handling
pub fn init() {
    set_kernel_trap_entry();
}
/// set trap entry for traps happen in kernel(supervisor) mode
fn set_kernel_trap_entry() {
    
    unsafe {
        stvec::write(__trap_from_kernel as usize, TrapMode::Direct);
    }
}
/// set trap entry for traps happen in user mode
fn set_user_trap_entry() {
    unsafe {
        stvec::write(__trap_from_user as usize, TrapMode::Direct);
    }
}

/// enable timer interrupt in supervisor mode
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

/// trap handler
#[no_mangle]
pub async fn trap_handler() {
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    trace!("into {:?}", scause.cause());
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            let mut cx = (unsafe { *current_trap_cx() });
            cx.sepc += 4;
            // get system call return value
            let result = syscall(
                cx.x[17],
                [
                    cx.x[10],
                    cx.x[11],
                    cx.x[12],
                    cx.x[13],
                   
                ],
            ).await
            ;
            // cx is changed during sys_exec, so we have to call it again
            cx = unsafe { *current_trap_cx() };
           
            cx.x[10] = result as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            error!(
                "[kernel] trap_handler: {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                scause.cause(),
                stval,
             unsafe{( *current_trap_cx())}  .sepc,
            );
           // panic!("s.");
            //current_add_signal(SignalFlags::SIGSEGV);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            current_add_signal(SignalFlags::SIGILL);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            check_timer();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    // check signals
    if let Some((_errno, msg)) = check_signals_of_current() {
        trace!("[kernel]  trap_handler: .. check signals {}", msg);
        /* exit_current_and_run_next(errno); */
    }
    trap_return();
}

/// return to user space
#[no_mangle]
pub  fn trap_return()  {
     // Important!
     close_interrupt();

     set_user_trap_entry();
 
     extern "C" {
         // fn __alltraps();
         fn __return_to_user(cx: *mut TrapContext);
     }
     
     // If no pending sig for process, then check for thread.
     // TODO: not sure whether this is the right way
     // if !check_signal_for_current_process() {
     //     check_signal_for_current_thread();
     // }
     //check_signal_for_current_task();
     
     debug!("satp:{:x}",satp::read().bits());
     unsafe {
        // (current_task().inner.get()).time_info.when_trap_ret();
 
         // Restore the float regs if needed.
         // Two cases that may need to restore regs:
         // 1. This task has yielded after last trap
         // 2. This task encounter a signal handler
          (*current_trap_cx()).user_fx.restore();
          (*current_trap_cx()).sstatus.set_fs(FS::Clean);
         
         __return_to_user(current_trap_cx());
 
         (*current_trap_cx())
             .user_fx
             .mark_save_if_needed((*current_trap_cx()).sstatus); 
         // Next trap will arrive here
         // current_trap_cx().user_fx.save();
 
         //(*current_task().inner.get()).time_info.when_trap_in();
     }
}

/// handle trap from kernel
/* #[no_mangle]
pub fn trap_from_kernel() -> ! {
    use riscv::register::sepc;
    trace!("stval = {:#x}, sepc = {:#x}", stval::read(), sepc::read());
    panic!("a trap {:?} from kernel!", scause::read().cause());
} */

pub use context::TrapContext;
/// Kernel trap handler
#[no_mangle]
pub fn kernel_trap_handler() {
    let scause = scause::read();
    let _stval = stval::read();
    match scause.cause() {
        /* Trap::Interrupt(Interrupt::SupervisorExternal) => {
            // error!("external interrrupt!!");
            crate::driver::intr_handler();
        } */
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            // log::error!("kernel timer interrrupt!!");
          /*   IRQ_COUNTER.add1(1);
            handle_timeout_events();
            set_next_trigger(); */
        panic!("WAIT");
        }
        _ => {
            // error!("other exception!!");
            error!(
                "[kernel] {:?}(scause:{}) in application, bad addr = {:#x}, bad instruction = {:#x}, kernel panicked!!",
                scause::read().cause(),
                scause::read().bits(),
                stval::read(),
                sepc::read(),
            );
            panic!(
                "a trap {:?} from kernel! stval {:#x}",
                scause::read().cause(),
                stval::read()
            );
        }
    }
}
///CS
pub fn close_interrupt() {
    //#[cfg(feature = "kernel_interrupt")]
    unsafe {
        riscv::register::sstatus::clear_sie()
    }
}
///CS
pub fn open_interrupt() {
    // info!("open interrupt");
   // #[cfg(feature = "kernel_interrupt")]
    unsafe {
        riscv::register::sstatus::set_sie();
    }
}
