use nes_base::Apu;

pub struct DummyApu {
    irq_pending: bool,
}

impl DummyApu {
    pub fn new() -> Self {
        Self { irq_pending: false }
    }
}

impl Apu for DummyApu {
    fn write_reg_pulse1_control(&mut self, _value: u8) {}
    fn write_reg_pulse1_sweep(&mut self, _value: u8) {}
    fn write_reg_pulse1_timer_low(&mut self, _value: u8) {}
    fn write_reg_pulse1_timer_high(&mut self, _value: u8) {}

    fn write_reg_pulse2_control(&mut self, _value: u8) {}
    fn write_reg_pulse2_sweep(&mut self, _value: u8) {}
    fn write_reg_pulse2_timer_low(&mut self, _value: u8) {}
    fn write_reg_pulse2_timer_high(&mut self, _value: u8) {}

    fn write_reg_triangle_control(&mut self, _value: u8) {}
    fn write_reg_triangle_timer_low(&mut self, _value: u8) {}
    fn write_reg_triangle_timer_high(&mut self, _value: u8) {}

    fn write_reg_noise_control(&mut self, _value: u8) {}
    fn write_reg_noise_period(&mut self, _value: u8) {}
    fn write_reg_noise_length(&mut self, _value: u8) {}

    fn write_reg_dmc_control(&mut self, _value: u8) {}
    fn write_reg_dmc_value(&mut self, _value: u8) {}
    fn write_reg_dmc_address(&mut self, _value: u8) {}
    fn write_reg_dmc_length(&mut self, _value: u8) {}

    fn write_reg_control(&mut self, _value: u8) {}
    fn write_reg_frame_counter(&mut self, _value: u8) {}
    fn read_reg_status(&self) -> u8 {
        0
    }

    fn clock(&mut self) {}

    fn check_irq_interrupt(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq_interrupt(&mut self) {
        self.irq_pending = false;
    }
}
