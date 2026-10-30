use nes_base::Apu;
use std::cell::Cell;
mod dmc;
mod envelope;
mod mixer;
mod noise;
mod pulse;
mod triangle;
pub use mixer::{CPU_HZ, SAMPLE_RATE};

pub struct ApuImpl {
    pulses: [pulse::Pulse; 2],
    triangle: triangle::Triangle,
    noise: noise::Noise,
    dmc: dmc::Dmc,
    mixer: mixer::Mixer,
    cycle: u32,
    even_cycle: bool,
    frame_control: u8,
    frame_irq: Cell<bool>,
    reset_delay: u8,
}
impl Default for ApuImpl {
    fn default() -> Self {
        let mut pulses = [pulse::Pulse::default(), pulse::Pulse::default()];
        pulses[0].first = true;
        Self {
            pulses,
            triangle: Default::default(),
            noise: Default::default(),
            dmc: Default::default(),
            mixer: Default::default(),
            cycle: 0,
            even_cycle: false,
            frame_control: 0,
            frame_irq: Cell::new(false),
            reset_delay: 0,
        }
    }
}
impl ApuImpl {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn take_samples(&mut self) -> Vec<f32> {
        self.mixer.take_samples()
    }
    fn quarter(&mut self) {
        for pulse in &mut self.pulses {
            pulse.quarter();
        }
        self.triangle.quarter();
        self.noise.quarter();
    }
    fn half(&mut self) {
        for pulse in &mut self.pulses {
            pulse.half();
        }
        self.triangle.half();
        self.noise.half();
    }
}
impl Apu for ApuImpl {
    fn write_reg_pulse1_control(&mut self, value: u8) {
        self.pulses[0].write(0, value);
    }
    fn write_reg_pulse1_sweep(&mut self, value: u8) {
        self.pulses[0].write(1, value);
    }
    fn write_reg_pulse1_timer_low(&mut self, value: u8) {
        self.pulses[0].write(2, value);
    }
    fn write_reg_pulse1_timer_high(&mut self, value: u8) {
        self.pulses[0].write(3, value);
    }
    fn write_reg_pulse2_control(&mut self, value: u8) {
        self.pulses[1].write(0, value);
    }
    fn write_reg_pulse2_sweep(&mut self, value: u8) {
        self.pulses[1].write(1, value);
    }
    fn write_reg_pulse2_timer_low(&mut self, value: u8) {
        self.pulses[1].write(2, value);
    }
    fn write_reg_pulse2_timer_high(&mut self, value: u8) {
        self.pulses[1].write(3, value);
    }
    fn write_reg_triangle_control(&mut self, value: u8) {
        self.triangle.write(0, value);
    }
    fn write_reg_triangle_timer_low(&mut self, value: u8) {
        self.triangle.write(2, value);
    }
    fn write_reg_triangle_timer_high(&mut self, value: u8) {
        self.triangle.write(3, value);
    }
    fn write_reg_noise_control(&mut self, value: u8) {
        self.noise.write(0, value);
    }
    fn write_reg_noise_period(&mut self, value: u8) {
        self.noise.write(2, value);
    }
    fn write_reg_noise_length(&mut self, value: u8) {
        self.noise.write(3, value);
    }
    fn write_reg_dmc_control(&mut self, value: u8) {
        self.dmc.write(0, value);
    }
    fn write_reg_dmc_value(&mut self, value: u8) {
        self.dmc.write(1, value);
    }
    fn write_reg_dmc_address(&mut self, value: u8) {
        self.dmc.write(2, value);
    }
    fn write_reg_dmc_length(&mut self, value: u8) {
        self.dmc.write(3, value);
    }
    fn write_reg_control(&mut self, value: u8) {
        for (index, pulse) in self.pulses.iter_mut().enumerate() {
            pulse.enabled = value & (1 << index) != 0;
            if !pulse.enabled {
                pulse.length = 0;
            }
        }
        self.triangle.enabled = value & 4 != 0;
        if !self.triangle.enabled {
            self.triangle.length = 0;
        }
        self.noise.enabled = value & 8 != 0;
        if !self.noise.enabled {
            self.noise.length = 0;
        }
        self.dmc.set_enabled(value & 16 != 0);
    }
    fn write_reg_frame_counter(&mut self, value: u8) {
        self.frame_control = value;
        if value & 0x40 != 0 {
            self.frame_irq.set(false);
        }
        self.reset_delay = if self.even_cycle { 3 } else { 4 };
    }
    fn read_reg_status(&self) -> u8 {
        u8::from(self.pulses[0].length > 0)
            | (u8::from(self.pulses[1].length > 0) << 1)
            | (u8::from(self.triangle.length > 0) << 2)
            | (u8::from(self.noise.length > 0) << 3)
            | (u8::from(self.dmc.remaining > 0) << 4)
            | (u8::from(self.frame_irq.replace(false)) << 6)
            | (u8::from(self.dmc.irq) << 7)
    }
    fn clock(&mut self) {
        self.cycle += 1;
        self.even_cycle = !self.even_cycle;
        if self.even_cycle {
            for pulse in &mut self.pulses {
                pulse.clock_timer();
            }
        }
        self.triangle.clock_timer();
        self.noise.clock_timer();
        self.dmc.clock();
        let five_step = self.frame_control & 0x80 != 0;
        if matches!(self.cycle, 7457 | 14913 | 22371) {
            self.quarter();
            if self.cycle == 14913 {
                self.half();
            }
        }
        if !five_step {
            if (29828..=29830).contains(&self.cycle) && self.frame_control & 0x40 == 0 {
                self.frame_irq.set(true);
            }
            if self.cycle == 29829 {
                self.quarter();
                self.half();
            }
            if self.cycle == 29830 {
                self.cycle = 0;
            }
        } else {
            if self.cycle == 37281 {
                self.quarter();
                self.half();
            }
            if self.cycle == 37282 {
                self.cycle = 0;
            }
        }
        if self.reset_delay > 0 {
            self.reset_delay -= 1;
            if self.reset_delay == 0 {
                self.cycle = 0;
                if five_step {
                    self.quarter();
                    self.half();
                }
            }
        }
        self.mixer.clock(
            self.pulses[0].output() + self.pulses[1].output(),
            self.triangle.output(),
            self.noise.output(),
            self.dmc.output,
        );
    }
    fn reset(&mut self) {
        *self = Self::new();
    }
    fn check_irq_interrupt(&self) -> bool {
        self.frame_irq.get() || self.dmc.irq
    }
    fn clear_irq_interrupt(&mut self) {
        self.frame_irq.set(false);
    }
    fn take_dmc_request(&mut self) -> Option<u16> {
        self.dmc.request()
    }
    fn supply_dmc_sample(&mut self, sample: u8) {
        self.dmc.supply(sample);
    }
}
