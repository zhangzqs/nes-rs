pub const CPU_HZ: u32 = 1_789_773;
pub const SAMPLE_RATE: u32 = 44_100;

#[derive(Default)]
pub(crate) struct Mixer {
    phase: u32,
    sum: f32,
    count: u32,
    levels: [u8; 4],
    mixed: f32,
    input: f32,
    high_pass: f32,
    low_pass: f32,
    samples: Vec<f32>,
}
impl Mixer {
    pub fn clock(&mut self, pulse: u8, triangle: u8, noise: u8, dmc: u8) {
        let levels = [pulse, triangle, noise, dmc];
        if levels != self.levels {
            // DAC 电平通常连续多个 CPU 周期保持不变；在电平变化前
            // 复用非线性混音结果，但仍逐周期计算平均值。
            self.levels = levels;
            let pulse = pulse as f32;
            let pulse_out = if pulse == 0.0 {
                0.0
            } else {
                95.88 / (8128.0 / pulse + 100.0)
            };
            let tnd = triangle as f32 / 8227.0 + noise as f32 / 12241.0 + dmc as f32 / 22638.0;
            let tnd_out = if tnd == 0.0 {
                0.0
            } else {
                159.79 / (1.0 / tnd + 100.0)
            };
            self.mixed = pulse_out + tnd_out;
        }
        self.sum += self.mixed;
        self.count += 1;
        self.phase += SAMPLE_RATE;
        if self.phase >= CPU_HZ {
            self.phase -= CPU_HZ;
            let input = self.sum / self.count as f32;
            // 先去除直流分量，再执行低通滤波，处理频率为 44.1 kHz。
            self.high_pass = 0.987 * (self.high_pass + input - self.input);
            self.input = input;
            self.low_pass += 0.65 * (self.high_pass - self.low_pass);
            // 限制无窗口模式下未被读取的音频样本，避免缓冲无限增长。
            if self.samples.len() < SAMPLE_RATE as usize {
                self.samples.push(self.low_pass);
            }
            self.sum = 0.0;
            self.count = 0;
        }
    }
    pub fn take_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.samples)
    }
}
