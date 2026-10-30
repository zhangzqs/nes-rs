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
        // 预填充三个设备缓冲区（约 70 毫秒），再开始播放。
        let prefill_bytes = u32::from(device.spec().samples) * 3 * size_of::<f32>() as u32;
        Ok(Self {
            device,
            prefill_bytes,
        })
    }

    pub fn queue(&self, samples: &[f32]) -> Result<(), String> {
        if self.device.size() == 0 {
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

    pub fn wait_duration(&self) -> Duration {
        // 超出两个预填充量时，先等待设备消耗样本，再生成下一帧。
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
    #[ignore = "需要 SDL_AUDIODRIVER=dummy，以测试真实 SDL 音频队列"]
    fn playback_buffers_recovers_and_preserves_samples() {
        assert_eq!(std::env::var("SDL_AUDIODRIVER").as_deref(), Ok("dummy"));
        let sdl = sdl2::init().unwrap();
        let playback = Playback::new(&sdl).unwrap();
        let block = vec![0.1; playback.device.spec().samples as usize];
        let block_bytes = size_of_val(block.as_slice()) as u32;

        // 启动和暂停/复位后的队列都需要重新预填充。
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

        // 连续写入大块音频时也保留样本，并要求调用方等待。
        let large = block.repeat(8);
        for _ in 0..3 {
            let before = playback.device.size();
            playback.queue(&large).unwrap();
            playback.device.pause();
            assert!(playback.device.size() > before, "队列积压时丢失了样本");
            assert!(playback.wait_duration() > Duration::ZERO);
        }

        // 设备耗尽队列后，恢复播放也应先预填充。
        playback.device.resume();
        let deadline = Instant::now() + Duration::from_secs(3);
        while playback.device.size() != 0 {
            assert!(Instant::now() < deadline, "虚拟音频设备未能耗尽队列");
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(playback.wait_duration(), Duration::ZERO);
        playback.queue(&block).unwrap();
        assert_eq!(playback.device.status(), AudioStatus::Paused);
        assert_eq!(playback.device.size(), block_bytes);
        playback.queue(&block).unwrap();
        playback.queue(&block).unwrap();
        assert_eq!(playback.device.status(), AudioStatus::Playing);
    }
}
