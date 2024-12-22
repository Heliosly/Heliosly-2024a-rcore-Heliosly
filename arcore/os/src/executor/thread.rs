
pub struct ThreadInner {
    // TODO: add more members
    /// Trap context that saves both kernel and user msg
    pub trap_context: TrapContext,
    /// Used for signal handle
    pub signal_context: Option<SignalContext>,
    /// Tid address, which may be modified by `set_tid_address` syscall
    pub tid_addr: TidAddress,
    /// Time info
    pub time_info: ThreadTimeInfo,
    /// Waker
    pub waker: Option<Waker>,
    /// Ustack top
    pub ustack_top: usize,
    /// Thread cpu affinity
    pub cpu_set: CpuSet,
    /// Note that the process may modify this value in the another thread
    /// (e.g. `exec`)
    pub terminated: bool,
}
