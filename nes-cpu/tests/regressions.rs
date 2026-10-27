use nes_base::{BusAdapter, Cpu, Interrupt, Reader, Writer};
use nes_cpu::CpuImpl;
use std::{cell::RefCell, rc::Rc};
struct Memory(Vec<u8>);
impl Reader for Memory {
    fn read(&self, address: u16) -> u8 {
        self.0[address as usize]
    }
}
impl Writer for Memory {
    fn write(&mut self, address: u16, value: u8) {
        self.0[address as usize] = value;
    }
}
impl BusAdapter for Memory {
    fn address_accept(&self, _: u16) -> bool {
        true
    }
}
fn setup(pc: u16, program: &[u8]) -> (CpuImpl, Rc<RefCell<Memory>>) {
    let memory = Rc::new(RefCell::new(Memory(vec![0; 65536])));
    for (index, &value) in program.iter().enumerate() {
        memory
            .borrow_mut()
            .write(pc.wrapping_add(index as u16), value);
    }
    memory.borrow_mut().write_u16(0xfffc, pc);
    memory.borrow_mut().write_u16(0xfffa, 0x9000);
    memory.borrow_mut().write_u16(0xfffe, 0xa000);
    let mut cpu = CpuImpl::new();
    cpu.attach_bus(memory.clone());
    cpu.reset();
    for _ in 0..7 {
        cpu.clock();
    }
    (cpu, memory)
}
fn step(cpu: &mut CpuImpl) -> u32 {
    let before = cpu.dump_state().total_cycles;
    loop {
        cpu.clock();
        if cpu.dump_state().remaining_cycles == 0 {
            break;
        }
    }
    cpu.dump_state().total_cycles - before
}
#[test]
fn each_clock_counts_once_including_instruction_and_interrupt_start() {
    let (mut cpu, _) = setup(0x8000, &[0xa9, 1, 0xea, 0xea]);
    assert_eq!(step(&mut cpu), 2);
    assert_eq!(cpu.dump_state().reg_a, 1);
    assert_eq!(step(&mut cpu), 2);
    cpu.trigger_interrupt(Interrupt::Nmi);
    step(&mut cpu); // Edge after the previous instruction poll is deferred.
    assert_eq!(step(&mut cpu), 7);
    assert_eq!(cpu.dump_state().reg_pc, 0x9000);
}
#[test]
fn branch_page_cross_is_relative_to_pc_after_operand() {
    let (mut cpu, _) = setup(0x80fd, &[0xd0, 0xfe]);
    assert_eq!(step(&mut cpu), 3);
    let (mut cpu, _) = setup(0x80fe, &[0xd0, 0xfe]);
    assert_eq!(step(&mut cpu), 4);
}
#[test]
fn rotates_update_zero_for_memory_operands() {
    let (mut cpu, memory) = setup(0x8000, &[0x26, 0x10, 0x66, 0x11]);
    memory.borrow_mut().write(0x10, 0x80);
    step(&mut cpu);
    assert!(cpu.dump_state().reg_status.zero);
    step(&mut cpu);
    assert!(!cpu.dump_state().reg_status.zero);
    assert_eq!(memory.borrow().read(0x11), 0x80);
}
#[test]
fn brk_and_nmi_push_different_break_bits() {
    let (mut cpu, memory) = setup(0x8000, &[0]);
    assert_eq!(step(&mut cpu), 7);
    assert_eq!(memory.borrow().read(0x1fb) & 0x30, 0x30);
    assert_eq!(memory.borrow().read(0x1fc), 2);
    assert!(!cpu.dump_state().reg_status.break_command);
    let (mut cpu, memory) = setup(0x8000, &[0xea]);
    cpu.trigger_interrupt(Interrupt::Nmi);
    step(&mut cpu);
    step(&mut cpu);
    assert_eq!(memory.borrow().read(0x1fb) & 0x30, 0x20);
}
#[test]
fn nmi_supersedes_irq_and_masked_irq_does_not_consume_cycles() {
    let (mut cpu, _) = setup(0x8000, &[0x58, 0xea]);
    cpu.trigger_interrupt(Interrupt::Irq);
    assert_eq!(step(&mut cpu), 2);
    cpu.trigger_interrupt(Interrupt::Irq);
    cpu.trigger_interrupt(Interrupt::Nmi);
    assert_eq!(step(&mut cpu), 2);
    assert_eq!(step(&mut cpu), 7);
    assert_eq!(cpu.dump_state().reg_pc, 0x9000);
}
#[test]
fn address_and_rts_wrap_at_ffff() {
    let (mut cpu, memory) = setup(0x8000, &[0x60]);
    memory.borrow_mut().write(0x1fe, 0xff);
    memory.borrow_mut().write(0x1ff, 0xff);
    assert_eq!(step(&mut cpu), 6);
    assert_eq!(cpu.dump_state().reg_pc, 0);
    memory.borrow_mut().write(0xffff, 0xa9);
    memory.borrow_mut().write(0, 0x55);
    cpu.set_reg_pc(0xffff);
    step(&mut cpu);
    assert_eq!(cpu.dump_state().reg_a, 0x55);
}

#[test]
fn irq_polls_the_mask_before_cli_and_sei_finish() {
    let (mut cpu, _) = setup(0x8000, &[0x58, 0xea, 0xea]);
    cpu.set_irq_line(true);
    step(&mut cpu); // CLI clears I after the interrupt poll.
    assert_eq!(step(&mut cpu), 2); // following NOP must execute
    assert_eq!(step(&mut cpu), 7);
    assert_eq!(cpu.dump_state().reg_pc, 0xa000);

    let (mut cpu, _) = setup(0x8000, &[0x58, 0x78, 0xea]);
    step(&mut cpu);
    cpu.set_irq_line(true);
    step(&mut cpu); // SEI sets I after the interrupt poll.
    assert_eq!(step(&mut cpu), 7);
    assert_eq!(cpu.dump_state().reg_pc, 0xa000);
}

#[test]
fn jam_halts_until_reset_without_panicking_or_executing_the_next_opcode() {
    let (mut cpu, memory) = setup(0x8000, &[0x22, 0xa9, 0xff]);
    for _ in 0..20 {
        cpu.clock();
    }
    assert_eq!(cpu.halt_reason(), Some((0x8000, 0x22)));
    assert_eq!(cpu.dump_state().reg_a, 0);
    memory.borrow_mut().write(0x8000, 0xea);
    cpu.reset();
    assert!(cpu.halt_reason().is_none());
    for _ in 0..11 {
        cpu.clock();
    }
    assert_eq!(cpu.dump_state().reg_a, 0xff);
}
