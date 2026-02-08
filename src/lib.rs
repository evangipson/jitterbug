use rand_core::{TryCryptoRng, TryRng};
use std::arch::x86_64::_rdtsc;
use std::collections::hash_map::DefaultHasher;
use std::convert::Infallible;
use std::hash::{Hash, Hasher};
use std::hint::black_box;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

const BACKGROUND_NOISE: u64 = 0x517cc1b727220a95;

/// A True Random Number Generator (TRNG) based on CPU execution jitter.
///
/// [`Jitterbug`] harvests entropy by measuring subtle timing variations in
/// CPU cycles caused by a background 'disruptor' thread and system noise.
pub struct Jitterbug {
    /// [`running`](Jitterbug::running) is a flag that, when `true`, denotes
    /// the harvester thread is currently running.
    running: Arc<AtomicBool>,
    /// [`disruptor`](Jitterbug::disruptor) is a join handle to the disruptor
    /// thread.
    disruptor: Option<thread::JoinHandle<()>>,
    /// [`buffer`](Jitterbug::buffer) is a buffer for entropy bits.
    buffer: [u8; 32],
    /// [`index`](Jitterbug::index) is the current index of the
    /// [`buffer`](Jitterbug::buffer).
    index: usize,
    /// [`last_raw`](Jitterbug::last_raw) is the last unsigned 64-bit integer
    /// that was generated.
    last_raw: u64,
}

/// Implement [`Default`] for [`Jitterbug`]
impl Default for Jitterbug {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement [`Jitterbug`]
impl Jitterbug {
    /// Creates a `new` [`Jitterbug`] generator.
    /// ## Usage
    /// Use [`new`](Jitterbug::new) to create a [`Jitterbug`]:
    /// ```rust
    /// use jitterbug::Jitterbug;
    /// use rand_core::Rng;
    ///
    /// // create a new jitterbug, and unwrap for direct
    /// // `Infallable` `Result`
    /// let mut jitter_rng = Jitterbug::new();
    ///
    /// // generate a new `u64` number
    /// let random_number = jitter_rng.next_u64();
    /// println!("random number: {random_number}");
    /// ```
    pub fn new() -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        let disruptor = thread::spawn(move || {
            let mut junk = vec![0u64; 1024];
            while r.load(Ordering::Relaxed) {
                junk.iter_mut()
                    .take(1024)
                    .for_each(|x| *x = x.wrapping_add(BACKGROUND_NOISE));
            }
        });

        let mut rng = Self {
            running,
            disruptor: Some(disruptor),
            buffer: [0u8; 32],
            index: 32,
            last_raw: 0,
        };

        rng.harvest();
        rng
    }

    fn harvest(&mut self) {
        let mut pool = Vec::with_capacity(256);
        for _ in 0..256 {
            unsafe {
                let t1 = _rdtsc();
                for _ in 0..100 {
                    black_box(0);
                }
                let t2 = _rdtsc();
                let delta = t2.wrapping_sub(t1);

                if delta == self.last_raw {
                    thread::yield_now();
                }
                self.last_raw = delta;
                pool.push(delta);
            }
        }

        for salt in 0..4u64 {
            let mut hasher = DefaultHasher::new();
            (salt, &pool).hash(&mut hasher);
            let bytes = hasher.finish().to_le_bytes();
            self.buffer[salt as usize * 8..(salt as usize + 1) * 8].copy_from_slice(&bytes);
        }
        self.index = 0;
    }
}

/// Implement [`TryRng`] for [`Jitterbug`]
impl TryRng for Jitterbug {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        let mut bytes = [0u8; 4];
        self.try_fill_bytes(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let mut bytes = [0u8; 8];
        self.try_fill_bytes(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        for byte in dest.iter_mut() {
            if self.index >= 32 {
                self.harvest();
            }
            *byte = self.buffer[self.index];
            self.index += 1;
        }
        Ok(())
    }
}

/// Implement [`TryCryptoRng`] for [`Jitterbug`]
impl TryCryptoRng for Jitterbug {}

/// Implement [`Drop`] for [`Jitterbug`]
impl Drop for Jitterbug {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.disruptor.take() {
            let _ = handle.join();
        }
    }
}
