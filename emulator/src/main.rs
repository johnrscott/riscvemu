use riscvemu::hart::memory::Wordsize;
use riscvemu::{hart::Hart, elf_utils::load_elf};
use std::io::{self, stdout};
use crossterm::{
    event::{self, Event, KeyCode},
    ExecutableCommand,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui::{prelude::*, widgets::*};

fn reg_index_to_abi_name(index: usize) -> String {
    match index {
	0 => "zero".to_string(),
	1 => "ra (return address)".to_string(),
	2 => "sp (stack pointer)".to_string(),
	3 => "gp (global pointer)".to_string(),
	4 => "tp (thread_pointer)".to_string(),
	5..=7 => format!("t{} (temporary)", index-5),
	8 => "s0/fp (saved register/frame pointer)".to_string(),
	9 => "s1 (saved register)".to_string(),
	10..=11 => format!("a{} (function arg/return value)", index-10),
	12..=17 => format!("a{} (function arg)", index-10),
	18..=27 => format!("s{} (saved register)", index-18),
	28..=31 => format!("t{} (temporary)", index-25),
	_ => panic!("Invalid register index"),
    }
}

fn main() -> io::Result<()> {

    let mut hart = Hart::default();
    let elf_name = "c/hello.out".to_string();

    // Load text section at 0 offset
    load_elf(&mut hart, &elf_name);

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut hart_stdout = String::new();
    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| ui(f, &mut hart, &mut hart_stdout))?;
	//hart.step().unwrap();
	match handle_events()? {
	    Keypress::Quit => should_quit = true,
	    Keypress::Step => {
		hart.step().unwrap()
	    }
	    Keypress::None => {}
	}
	
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

enum Keypress {
    Quit,
    Step,
    None,
}

fn handle_events() -> io::Result<Keypress> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(Keypress::Quit);
            } else if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Enter {
		return Ok(Keypress::Step);
	    }
	}
    }
    Ok(Keypress::None)
}

fn get_registers(hart: &Hart) -> Paragraph {
    let mut lines = Vec::new();
    for n in 0..32 {
	let value = hart.registers.read(n).unwrap();
	let reg_abi_name = reg_index_to_abi_name(n);
	let span = if value == 0 {
	    Span::raw(format!("{reg_abi_name}: 0x{value:x}"))
	} else {
	    format!("{reg_abi_name}: 0x{value:x}").bold().yellow()
	};
	lines.push(Line::from(vec![span]));
    }
    Paragraph::new(lines)
}

fn get_stack(hart: &Hart) -> Paragraph {

    let mut lines = Vec::new();
    
    let stack_bottom = 0xff0;
    let stack_pointer = hart.registers.read(2).expect("valid register read");

    if stack_pointer == 0 || stack_pointer > stack_bottom {
	Paragraph::new(format!("Stack pointer value sp={stack_pointer} not currently valid"))
    } else {
	for addr in (stack_pointer..=stack_bottom).rev().step_by(4) {
	    let value = hart.memory.read(addr, Wordsize::Word).expect("valid memory read");
	    let span = if value == 0 {
		Span::raw(format!("0x{addr:x}: 0x{value:x}"))
	    } else {
		format!("0x{addr:x}: 0x{value:x}").bold().yellow()
	    };
	    lines.push(Line::from(vec![span]));
	}
	Paragraph::new(lines)
    }
}


fn ui(frame: &mut Frame, hart: &mut Hart, hart_stdout: &mut String) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.size());

    let pc = hart.pc;
    let instr = hart.memory.read(pc.into(), Wordsize::Word).unwrap();
    //let instr = Rv32i::from(instr.try_into().unwrap());
    hart_stdout.push_str(&hart.memory.flush_stdout());
	
    let lines = vec![
	Line::from(format!("Current pc = 0x{pc:x}")),
	Line::from(format!("Next instruction: {:x?}", instr))
    ];
	
    frame.render_widget(
        Block::new().borders(Borders::TOP).title("RISCV Emulator").bold().red().on_white(),
        main_layout[0],
    );
    frame.render_widget(
        Block::new().borders(Borders::TOP).title("Status Bar"),
        main_layout[2],
    );

    let stdout_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(main_layout[1]);
    
    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(stdout_layout[0]);

    frame.render_widget(
	Paragraph::new(hart_stdout.clone()).wrap(Wrap { trim: false })
	    .block(Block::default().borders(Borders::ALL).title("Hart Console Output")),
        stdout_layout[1],
    );
    
    let hart_info_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)])
        .split(inner_layout[0]);

    frame.render_widget(
	Paragraph::new(lines).wrap(Wrap { trim: false })
	    .block(Block::default().borders(Borders::ALL).title("Next Instruction")),
        hart_info_layout[0],
    );
    
    frame.render_widget(
	get_registers(hart)
	    .block(Block::default().borders(Borders::ALL).title("Registers")),
        hart_info_layout[1],
    );

    frame.render_widget(
	get_stack(hart)
	    .block(Block::default().borders(Borders::ALL).title("Stack Frame")),
	inner_layout[1],
    );
}

/*
fn press_enter_to_continue() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    write!(stdout, "Press enter to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}
*/
