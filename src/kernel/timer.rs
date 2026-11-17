//! Timer wheel + interrupt vector table abstractions.

use std::collections::BTreeMap;

/// Timer ticks are the kernel's base time unit.
pub type Tick = u64;

/// A scheduled callback identified by an id.
pub type TimerId = u64;

/// Errors from the timer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerError {
    /// Timer id not found.
    NoSuchTimer(TimerId),
    /// Deadline is in the past.
    PastDeadline,
}

impl std::fmt::Display for TimerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSuchTimer(i) => write!(f, "no such timer {i}"),
            Self::PastDeadline => write!(f, "deadline in the past"),
        }
    }
}
impl std::error::Error for TimerError {}

/// A scheduled timer.
#[derive(Debug, Clone)]
pub struct Timer {
    pub id: TimerId,
    pub deadline: Tick,
    pub periodic: bool,
    pub period: Tick,
    /// Opaque payload tag returned on fire.
    pub tag: u64,
}

/// A hierarchical-ish timer wheel (simple BTreeMap by deadline).
#[derive(Debug, Default)]
pub struct TimerWheel {
    timers: BTreeMap<Tick, Vec<Timer>>,
    next_id: TimerId,
    now: Tick,
}

impl TimerWheel {
    pub fn new() -> Self { Self::default() }
    pub fn now(&self) -> Tick { self.now }

    /// Schedule a one-shot timer `after` ticks from now.
    pub fn after(&mut self, after: Tick, tag: u64) -> TimerId {
        self.next_id += 1;
        let id = self.next_id;
        let t = Timer { id, deadline: self.now + after, periodic: false, period: 0, tag };
        self.timers.entry(t.deadline).or_default().push(t);
        id
    }

    /// Schedule a periodic timer firing every `period` ticks, first after `delay`.
    pub fn every(&mut self, delay: Tick, period: Tick, tag: u64) -> TimerId {
        self.next_id += 1;
        let id = self.next_id;
        let t = Timer { id, deadline: self.now + delay, periodic: true, period, tag };
        self.timers.entry(t.deadline).or_default().push(t);
        id
    }

    /// Cancel a timer.
    pub fn cancel(&mut self, id: TimerId) -> Result<(), TimerError> {
        for (dl, list) in self.timers.iter_mut() {
            if let Some(pos) = list.iter().position(|t| t.id == id) {
                list.remove(pos);
                let _ = dl;
                // clean empty buckets lazily
                return Ok(());
            }
        }
        Err(TimerError::NoSuchTimer(id))
    }

    /// Advance time by one tick; returns tags of timers that fired.
    pub fn tick(&mut self) -> Vec<u64> {
        self.now += 1;
        let mut fired = Vec::new();
        // collect deadlines <= now
        let due: Vec<Tick> = self.timers.range(..=self.now).map(|(&d, _)| d).collect();
        for d in due {
            if let Some(mut list) = self.timers.remove(&d) {
                for t in list.drain(..) {
                    fired.push(t.tag);
                    if t.periodic && t.period > 0 {
                        let nt = Timer { deadline: t.deadline + t.period, ..t };
                        self.timers.entry(nt.deadline).or_default().push(nt);
                    }
                }
            }
        }
        fired
    }

    /// Number of pending timers.
    pub fn pending(&self) -> usize {
        self.timers.values().map(|v| v.len()).sum()
    }
}

// ---- interrupt vector table ----

/// An interrupt/exception handler.
pub type IrqHandler = fn(u64) -> ();

/// Interrupt controller: maps vector numbers to handlers.
#[derive(Debug, Default)]
pub struct Idt {
    handlers: BTreeMap<u8, IrqHandler>,
}

impl Idt {
    pub fn new() -> Self { Self::default() }

    /// Install a handler for interrupt vector `vec`.
    pub fn set_handler(&mut self, vec: u8, h: IrqHandler) {
        self.handlers.insert(vec, h);
    }

    /// Dispatch interrupt `vec` with `irq` payload.
    pub fn dispatch(&self, vec: u8, irq: u64) {
        if let Some(h) = self.handlers.get(&vec) {
            h(irq);
        }
    }

    pub fn len(&self) -> usize { self.handlers.len() }
}
