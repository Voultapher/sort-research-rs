//! This module implements functionality for measuring and comparing the duration of some task.

use std::cmp;

/// Represents a duration, the metric of the duration is platform specific. On x86_64 its cycles and
/// on other platforms its nanoseconds.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DurationOpaque {
    #[cfg(target_arch = "x86_64")]
    cycles: u64,
    #[cfg(not(target_arch = "x86_64"))]
    duration: std::time::Duration,
}

/// Measures the time it takes to execute the function `test_fn`.
#[inline(never)]
pub fn measure_duration(mut test_fn: impl FnMut()) -> DurationOpaque {
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: We checked the arch.
        unsafe {
            // Wait for earlier instructions to retire before reading the clock.
            std::arch::x86_64::_mm_lfence();
        }
    }
    let start = current_time_stamp();

    test_fn();

    let end = current_time_stamp();
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: We checked the arch.
        unsafe {
            // Block later instructions until reading the clock retires.
            std::arch::x86_64::_mm_lfence();
        }
    }

    end - start
}

impl DurationOpaque {
    #[cfg(target_arch = "x86_64")]
    fn new(cycles: u64) -> Self {
        Self { cycles }
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn new(duration: std::time::Duration) -> Self {
        Self { duration }
    }

    /// Analyzes multiple measurement samples and returns variance and median duration.
    pub fn analyze(durations: &mut [Self]) -> (f64, Self) {
        let len = durations.len();

        if len < 3 {
            panic!("Needs at least 3 samples");
        }

        durations.sort_unstable_by_key(|val| {
            #[cfg(target_arch = "x86_64")]
            {
                val.cycles
            }

            #[cfg(not(target_arch = "x86_64"))]
            {
                val.duration.as_nanos()
            }
        });

        let mid = len / 2;
        let offset = cmp::max((len as f64 / 10.0).round() as usize, 1);

        let variance = durations[mid + offset].as_opaque() / durations[mid - offset].as_opaque();
        let median_duration = durations[mid];

        (variance, median_duration)
    }

    pub fn as_opaque(&self) -> f64 {
        #[cfg(target_arch = "x86_64")]
        {
            self.cycles as f64
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            self.duration.as_nanos() as f64
        }
    }

    pub fn from_opaque(opaque: f64) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            Self {
                cycles: opaque as u64,
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            Self {
                duration: std::time::Duration::from_nanos(opaque as f64),
            }
        }
    }

    #[allow(dead_code)]
    pub fn as_nanos(&self, cpu_frequency_ghz: f64) -> f64 {
        // With 53 mantissa bits, we will see errors once the duration is larger than ~104 days.
        // Uses f64 to allow for sub-nanosecond precision.

        #[cfg(target_arch = "x86_64")]
        {
            self.cycles as f64 / cpu_frequency_ghz
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            self.duration.as_nanos() as f64
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TimeStamp {
    #[cfg(target_arch = "x86_64")]
    cycles_since_machine_start: u64,
    #[cfg(not(target_arch = "x86_64"))]
    instant: std::time::Instant,
}

impl TimeStamp {
    #[cfg(target_arch = "x86_64")]
    fn new(cycles_since_machine_start: u64) -> Self {
        Self {
            cycles_since_machine_start,
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn new(instant: std::time::Instant) -> Self {
        Self { instant }
    }
}

impl std::ops::Sub for TimeStamp {
    type Output = DurationOpaque;

    fn sub(self, rhs: Self) -> Self::Output {
        #[cfg(target_arch = "x86_64")]
        {
            DurationOpaque::new(self.cycles_since_machine_start - rhs.cycles_since_machine_start)
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            DurationOpaque::new(self.instant.duration_since(rhs.instant))
        }
    }
}

fn current_time_stamp() -> TimeStamp {
    let time_stamp;

    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: We checked the arch.
        time_stamp = unsafe { std::arch::x86_64::_rdtsc() };
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        time_stamp = std::time::Instant::now();
    }

    TimeStamp::new(time_stamp)
}
