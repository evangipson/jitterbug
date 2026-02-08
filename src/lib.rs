use rand_core::{CryptoRng, Error, RngCore};
use std::arch::x86_64::_rdtsc;
use std::collections::hash_map::DefaultHasher;
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
}

/// Implement [`Default`] for [`Jitterbug`]
impl Default for Jitterbug {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement [`Jitterbug`]
impl Jitterbug {
    /// Creates a new instance and starts the background disruptor thread.
    pub fn new() -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        let disruptor = thread::spawn(move || {
            let mut junk = vec![0u64; 1024];
            while r.load(Ordering::Relaxed) {
                junk.iter_mut().take(1024).for_each(|x| {
                    *x = x.wrapping_add(BACKGROUND_NOISE);
                });
            }
        });

        let mut rng = Self {
            running,
            disruptor: Some(disruptor),
            buffer: [0u8; 32],
            index: 32,
        };

        rng.harvest();
        rng
    }

    /// Harvests entropy by measuring subtle timing variations in CPU cycles.
    fn harvest(&mut self) {
        let mut pool = Vec::with_capacity(256);
        for _ in 0..256 {
            unsafe {
                let t1 = _rdtsc();
                for _ in 0..100 {
                    black_box(0);
                }
                let t2 = _rdtsc();
                pool.push(t2.wrapping_sub(t1));
            }
        }

        // whiten the entropy into 4x 64-bit blocks
        for salt in 0..4u64 {
            let mut hasher = DefaultHasher::new();
            (salt, &pool).hash(&mut hasher);
            let bytes = hasher.finish().to_le_bytes();
            self.buffer[salt as usize * 8..(salt as usize + 1) * 8].copy_from_slice(&bytes);
        }
        self.index = 0;
    }
}

/// Implement [`Drop`] for [`Jitterbug`]
impl Drop for Jitterbug {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.disruptor.take() {
            let _ = handle.join();
        }
    }
}

/// Implement [`RngCore`] for [`Jitterbug`]
impl RngCore for Jitterbug {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        self.fill_bytes(&mut bytes);
        u32::from_le_bytes(bytes)
    }

    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        self.fill_bytes(&mut bytes);
        u64::from_le_bytes(bytes)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for byte in dest.iter_mut() {
            if self.index >= 32 {
                self.harvest();
            }
            *byte = self.buffer[self.index];
            self.index += 1;
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

/// Implement [`CryptoRng`] for [`Jitterbug`]
impl CryptoRng for Jitterbug {}
