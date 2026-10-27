use nes_apu::{ApuImpl, CPU_HZ, SAMPLE_RATE};
use nes_base::Apu;

#[test]
fn pulse_produces_finite_audio_at_the_requested_sample_rate() {
    let mut apu = ApuImpl::new();
    apu.write_reg_control(1);
    apu.write_reg_pulse1_control(0xbf); // 50% duty, constant volume, hold length
    apu.write_reg_pulse1_timer_low(253); // about 440 Hz
    apu.write_reg_pulse1_timer_high(0x08);
    for _ in 0..CPU_HZ {
        apu.clock();
    }
    let samples = apu.take_samples();
    assert_eq!(samples.len(), SAMPLE_RATE as usize);
    assert!(
        samples
            .iter()
            .all(|sample| sample.is_finite() && sample.abs() <= 1.0)
    );
    let crossings = samples
        .windows(2)
        .filter(|pair| pair[0] <= 0.0 && pair[1] > 0.0)
        .count();
    assert!(
        (435..=445).contains(&crossings),
        "{crossings} rising crossings"
    );
    apu.reset();
    for _ in 0..CPU_HZ {
        apu.clock();
    }
    assert!(
        apu.take_samples()
            .iter()
            .rev()
            .take(100)
            .all(|sample| sample.abs() < 1e-6)
    );
}

#[test]
fn dmc_wraps_sample_reads_and_irq_is_not_cleared_by_status_read() {
    let mut apu = ApuImpl::new();
    apu.write_reg_dmc_control(0x8f);
    apu.write_reg_dmc_address(0xff); // $FFC0
    apu.write_reg_dmc_length(4); // 65 bytes: final byte wraps to $8000
    apu.write_reg_control(0x10);
    let mut addresses = Vec::new();
    for _ in 0..40_000 {
        if let Some(address) = apu.take_dmc_request() {
            addresses.push(address);
            apu.supply_dmc_sample(0xaa);
        }
        apu.clock();
    }
    assert_eq!(addresses.len(), 65);
    assert_eq!(&addresses[63..], &[0xffff, 0x8000]);
    assert_eq!(apu.read_reg_status() & 0x90, 0x80);
    assert_eq!(apu.read_reg_status() & 0x80, 0x80);
    apu.write_reg_control(0);
    assert_eq!(apu.read_reg_status() & 0x80, 0);
}

#[test]
fn frame_irq_acknowledgement_and_inhibition_are_independent_of_channels() {
    let mut apu = ApuImpl::new();
    for _ in 0..29830 {
        apu.clock();
    }
    assert!(apu.check_irq_interrupt());
    assert_eq!(apu.read_reg_status() & 0x40, 0x40);
    assert!(!apu.check_irq_interrupt());
    apu.write_reg_frame_counter(0x40);
    for _ in 0..60_000 {
        apu.clock();
    }
    assert!(!apu.check_irq_interrupt());
}
