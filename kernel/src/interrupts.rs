use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use pic8259_simple::ChainedPics;
use spin;
use lazy_static::lazy_static;
use crate::{println, print, gdt, hlt_loop, driver::serial::COM1, memory::{PAGE_ALLOCATOR, MAPPER}};
use x86_64::registers::control::Cr2;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.divide_error.set_handler_fn(divided_by_zero_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::COM1.as_usize()]
            .set_handler_fn(com1_interrupt_handler);
        idt
    };
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// TODO: WRITE MY OWN PICS

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn init_idt() {
    IDT.load();
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    COM1 = PIC_1_OFFSET + 4,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn divided_by_zero_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: DIVIDED BY ZERO\n{:#?}", stack_frame);
}

fn create_page() -> bool {
    use x86_64::{
        structures::paging::{Page, PageTableFlags as Flags, FrameAllocator, Mapper},
    };

    let virtual_address = Cr2::read();
    let page = Page::containing_address(virtual_address);
    let flags = Flags::PRESENT | Flags::WRITABLE;
    let frame = match (*PAGE_ALLOCATOR.lock()).as_mut().unwrap().allocate_frame()
    {
        Some(frame) => frame,
        None => return false,
    };

    let map_to_result = unsafe {
        (*MAPPER.lock()).as_mut().unwrap().
            map_to(page, frame, flags, (*PAGE_ALLOCATOR.lock()).as_mut().unwrap())
    };
    map_to_result.expect("map_to failed").flush();
    true
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame, error_code: PageFaultErrorCode)
{
    if (error_code & (PageFaultErrorCode::INSTRUCTION_FETCH
                    | PageFaultErrorCode::PROTECTION_VIOLATION))
        == PageFaultErrorCode::from_bits(0 as u64).unwrap()
    // when access is permitted and not instruction fetch
    {
        // we don't need to specify flags as
        // user part will take care of it.
        if create_page() {
            return ;
        }
    }

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame)
{
    // print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame)
{
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn com1_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame)
{
    let data = COM1.lock().receive();

    // TODO: SET A BUFFER FOR COMMANDS

    match data {
        0x0A | 0x0D => println!(),
        0x20..0x7F => print!("{}", data as char),
        _ => (),
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::COM1.as_u8());
    }
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}

#[test_case]
fn test_page_fault_exception() {
    unsafe {
        let point = 0xdeadbeef000 as *mut u8;
        *point = 42;
    }
}