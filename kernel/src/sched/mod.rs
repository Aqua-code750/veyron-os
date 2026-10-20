//! Preemptive Multitasking Scheduler
//! Round-robin scheduler supporting kernel threads and Ring 3 user mode processes.

pub mod context;

use context::context_switch;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ThreadState {
    Unused,
    Ready,
    Running,
    Blocked,
    Terminated,
}

const STACK_SIZE: usize = 16384; // 16 KB stack per thread
const MAX_THREADS: usize = 64;

#[repr(C, align(16))]
pub struct Thread {
    pub id: usize,
    pub name: [u8; 32],
    pub rsp: u64,
    pub cr3: u64,
    pub is_user: bool,
    pub state: ThreadState,
    pub sleep_until: u64,
    pub stack: [u8; STACK_SIZE],
}

impl Thread {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            name: [0; 32],
            rsp: 0,
            cr3: 0,
            is_user: false,
            state: ThreadState::Unused,
            sleep_until: 0,
            stack: [0; STACK_SIZE],
        }
    }
}

pub struct Scheduler {
    threads: [Thread; MAX_THREADS],
    current_index: usize,
    initialized: bool,
}

impl Scheduler {
    pub const fn new() -> Self {
        const EMPTY_THREAD: Thread = Thread::empty();
        Self {
            threads: [EMPTY_THREAD; MAX_THREADS],
            current_index: 0,
            initialized: false,
        }
    }

    pub fn init(&mut self) {
        // Thread 0 is the currently running bootstrap kernel thread
        self.threads[0].id = 0;
        let name = b"kmain\0";
        self.threads[0].name[..name.len()].copy_from_slice(name);
        self.threads[0].state = ThreadState::Running;
        self.threads[0].cr3 = crate::mm::vmm::read_cr3();
        self.threads[0].is_user = false;

        self.current_index = 0;
        self.initialized = true;

        crate::serial_println!("[SCHED] Preemptive scheduler initialized with Thread 0 (kmain).");
    }

    pub fn spawn_thread(&mut self, name_str: &str, entry: extern "C" fn()) -> Option<usize> {
        for i in 1..MAX_THREADS {
            if self.threads[i].state == ThreadState::Unused || self.threads[i].state == ThreadState::Terminated {
                let thread = &mut self.threads[i];
                thread.id = i;
                thread.state = ThreadState::Ready;
                thread.cr3 = crate::mm::vmm::read_cr3();
                thread.is_user = false;

                // Copy name
                thread.name.fill(0);
                let bytes = name_str.as_bytes();
                let len = bytes.len().min(31);
                thread.name[..len].copy_from_slice(&bytes[..len]);

                // Stack layout for context_switch:
                // When context_switch executes `ret`, it pops the entry point from the stack.
                // Before `ret`, it pops: popfq (8 bytes), r15..r12, rbx, rbp (6 * 8 = 48 bytes).
                // Total saved registers = 7 * 8 = 56 bytes.
                let stack_top = thread.stack.as_ptr() as usize + STACK_SIZE;
                // Align to 16 bytes
                let aligned_top = (stack_top & !0xF) - 8;

                unsafe {
                    let ptr = aligned_top as *mut u64;
                    // Return address for `ret`
                    *ptr.offset(0) = entry as usize as u64;
                    // RBP
                    *ptr.offset(-1) = 0;
                    // RBX
                    *ptr.offset(-2) = 0;
                    // R12
                    *ptr.offset(-3) = 0;
                    // R13
                    *ptr.offset(-4) = 0;
                    // R14
                    *ptr.offset(-5) = 0;
                    // R15
                    *ptr.offset(-6) = 0;
                    // RFLAGS: 0x202 (Interrupts Enabled, Reserved bit 1 set)
                    *ptr.offset(-7) = 0x202;

                    thread.rsp = ptr.offset(-7) as usize as u64;
                }

                crate::serial_println!("[SCHED] Spawned kernel thread #{} ('{}')", i, name_str);
                return Some(i);
            }
        }
        None
    }

    pub fn schedule(&mut self) {
        if !self.initialized {
            return;
        }

        let now = crate::drivers::timer::uptime_ms();

        // Wake up sleeping threads
        for t in self.threads.iter_mut() {
            if t.state == ThreadState::Blocked && t.sleep_until > 0 && now >= t.sleep_until {
                t.state = ThreadState::Ready;
                t.sleep_until = 0;
            }
        }

        let current = self.current_index;
        let mut next = (current + 1) % MAX_THREADS;

        while next != current {
            if self.threads[next].state == ThreadState::Ready {
                break;
            }
            next = (next + 1) % MAX_THREADS;
        }

        // If next thread is found and ready
        if self.threads[next].state == ThreadState::Ready && next != current {
            if self.threads[current].state == ThreadState::Running {
                self.threads[current].state = ThreadState::Ready;
            }
            self.threads[next].state = ThreadState::Running;
            self.current_index = next;

            // Set TSS rsp0 so interrupts occurring in user mode return to this thread's kernel stack
            let kstack_top = self.threads[next].stack.as_ptr() as u64 + STACK_SIZE as u64;
            crate::arch::x86_64::gdt::set_kernel_stack(kstack_top);

            unsafe {
                let prev_rsp_ptr = &mut self.threads[current].rsp as *mut u64;
                let next_rsp = self.threads[next].rsp;
                let next_cr3 = self.threads[next].cr3;
                context_switch(prev_rsp_ptr, next_rsp, next_cr3);
            }
        }
    }

    pub fn current_tid(&self) -> usize {
        self.current_index
    }

    pub fn yield_now(&mut self) {
        self.schedule();
    }
}

pub static mut SCHEDULER: Scheduler = Scheduler::new();

pub fn schedule() {
    unsafe {
        SCHEDULER.schedule();
    }
}

pub fn yield_now() {
    unsafe {
        SCHEDULER.yield_now();
    }
}
