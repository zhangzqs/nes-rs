use sdl2::audio::{AudioQueue, AudioSpecDesired, AudioStatus};
use std::time::Duration;

pub struct Playback {
    device: AudioQueue<f32>,
    prefill_bytes: u32,
}

impl Playback {
    pub fn new(sdl: &sdl2::Sdl) -> Result<Self, String> {
        let device = sdl.audio()?.open_queue::<f32, _>(
            None,
            &AudioSpecDesired {
                freq: Some(nes_apu::SAMPLE_RATE as i32),
                channels: Some(1),
                samples: Some(1024),
            },
        )?;
        // Keep three device buffers ready before playback (about 70 ms).
        // A single emulated frame cannot cover the audio thread's scheduling jitter.
        let prefill_bytes = u32::from(device.spec().samples) * 3 * size_of::<f32>() as u32;
        Ok(Self {
            device,
            prefill_bytes,
        })
    }

    pub fn queue(&self, samples: &[f32]) -> Result<(), String> {
        let queued = self.device.size();
        if queued > self.prefill_bytes * 4 {
            // A stalled device must not accumulate seconds of stale game audio.
            self.clear();
        } else if queued == 0 {
            self.device.pause();
        }
        self.device.queue_audio(samples)?;
        if self.device.status() == AudioStatus::Paused && self.device.size() >= self.prefill_bytes {
            self.device.resume();
        }
        Ok(())
    }

    pub fn clear(&self) {
        self.device.pause();
        self.device.clear();
    }

    pub fn frame_duration(&self, nominal: Duration) -> Duration {
        if self.device.status() != AudioStatus::Playing {
            return nominal;
        }
        // The host's audio clock can drift relative to Instant. Adjust pacing by
        // at most 0.5% to keep the buffer near its target over long sessions.
        let error = self.device.size() as f64 / self.prefill_bytes as f64 - 1.0;
        nominal.mul_f64(1.0 + error.clamp(-1.0, 1.0) * 0.005)
    }

    pub fn wait_duration(&self) -> Duration {
        // Slow down an overfull producer instead of discarding whole audio frames.
        let excess = self.device.size().saturating_sub(self.prefill_bytes * 2);
        let bytes_per_second = self.device.spec().freq as f64 * size_of::<f32>() as f64;
        Duration::from_secs_f64(excess as f64 / bytes_per_second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore = "Requires SDL_AUDIODRIVER=dummy; exercises a real SDL audio queue"]
    fn playback_buffers_recovers_and_preserves_samples() {
        assert_eq!(std::env::var("SDL_AUDIODRIVER").as_deref(), Ok("dummy"));
        let sdl = sdl2::init().unwrap();
        let playback = Playback::new(&sdl).unwrap();
        let block = vec![0.1; playback.device.spec().samples as usize];
        let block_bytes = size_of_val(block.as_slice()) as u32;

        // Neither startup nor a cleared/reset queue may start on one short frame.
        for _ in 0..2 {
            playback.queue(&block).unwrap();
            assert_eq!(playback.device.status(), AudioStatus::Paused);
            assert_eq!(playback.device.size(), block_bytes);
            assert_eq!(playback.wait_duration(), Duration::ZERO);
            playback.queue(&block).unwrap();
            assert_eq!(playback.device.status(), AudioStatus::Paused);
            assert_eq!(playback.device.size(), block_bytes * 2);
            playback.queue(&block).unwrap();
            assert_eq!(playback.device.status(), AudioStatus::Playing);
            playback.clear();
            assert_eq!(playback.device.status(), AudioStatus::Paused);
            assert_eq!(playback.device.size(), 0);
        }

        // An overfull queue retains every sample and asks the producer to wait.
        let large = block.repeat(8);
        playback.queue(&large).unwrap();
        playback.device.pause();
        let before = playback.device.size();
        playback.queue(&large).unwrap();
        playback.device.pause();
        let after = playback.device.size();
        assert!(
            after > before,
            "samples were dropped from an overfull queue"
        );
        assert!(playback.wait_duration() > Duration::ZERO);

        // A device that stops consuming must not grow an unbounded stale queue.
        playback.queue(&block).unwrap();
        assert_eq!(playback.device.status(), AudioStatus::Paused);
        assert_eq!(playback.device.size(), block_bytes);
        playback.queue(&block).unwrap();
        playback.queue(&block).unwrap();

        // Let a playing device run dry, then verify that recovery also prefills.
        playback.device.resume();
        let deadline = Instant::now() + Duration::from_secs(3);
        while playback.device.size() != 0 {
            assert!(Instant::now() < deadline, "dummy audio did not drain");
            std::thread::sleep(Duration::from_millis(5));
        }
        playback.queue(&block).unwrap();
        assert_eq!(playback.device.status(), AudioStatus::Paused);
        assert_eq!(playback.device.size(), block_bytes);
        playback.queue(&block).unwrap();
        playback.queue(&block).unwrap();
        assert_eq!(playback.device.status(), AudioStatus::Playing);
    }
}
