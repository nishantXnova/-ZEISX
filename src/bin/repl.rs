use std::io::{self, Write};
use calc::{evaluate, fast_two, fmt_op};

fn main() {
    println!("\n  ____             _             ___           _                ");
    println!(" |  _ \\  __ _  ___| | _____ _ __|_ _|_ __   __| | _____   __   ");
    println!(" | | | |/ _` |/ __| |/ / _ | '__|| || '_ \\ / _` |/ _ \\ \\ / /   ");
    println!(" | |_| | (_| | (__|   <  __| |   | || | | | (_| |  __/\\ V /    ");
    println!(" |____/ \\__,_|\\___|_|\\_\\___|_|  |___|_| |_|\\__,_|\\___| \\_/    ");
    println!("              ZEISX v1.0 · REPL · type 'quit' to exit\n");

    loop {
        print!(">>> ");
        let _ = io::stdout().lock().flush();
        let mut line = String::with_capacity(64);
        match io::stdin().read_line(&mut line) {
            Ok(0) => { println!("\nbye!"); break; }
            Ok(_) => {}
            Err(_) => break,
        }
        let input = line.trim().to_ascii_lowercase();
        if input == "exit" || input == "quit" || input == "q" {
            println!("bye!");
            return;
        }
        if input.is_empty() { continue; }
        if input == "help" {
            println!("ZEISX calc compiler: type any expression. Examples:");
            println!("  > 12 + 30          → 12 + 30 = 42");
            println!("  > 100 / 4          → 100 / 4 = 25");
            println!("  > -7 * 8           → -7 * 8 = -56");
            println!("  > (2 + 3) * 4      → (2 + 3) * 4 = 20");
            continue;
        }

        match evaluate(&input) {
            Ok(n) => {
                if let Some(p) = fast_two(&input) {
                    println!("{} {} {} = {}", p.0, fmt_op(p.1), p.2, n);
                } else {
                    println!("  => {}", n);
                }
            }
            Err(e) => eprintln!("error: {e}"),
        }
    }
}
