pub use crossterm::{
    cursor,
    event::{
        self,
        Event,
        KeyCode,
        KeyEvent,
    },
    execute,
    queue,
    style,
    terminal::{
        self,
        ClearType,
    },
    Command,
    Result,
};
use inquire::Text;
use rand::{
    rngs::StdRng,
    Rng,
    SeedableRng,
};
use std::{
    cmp::{
        max,
        min,
    },
    collections::HashSet,
    io,
};

// Descriptive title for the CLI app
const TITLE: &str = "Passtrain helps you remember passwords through repetition";
// The max number of allowed attempts at each difficulty level
const ATTEMPTS_PER_LEVEL: usize = 5;
// Starting difficulty percentage
const STARTING_DIFFICULTY: f32 = 0.2f32;
// Initial attempts
const STARTING_ATTEMPTS: usize = 2;
// max difficulty jump percentage
const MAX_DIFFICULTY_INCREASE: f32 = 0.2f32;

fn main() {
    run();
    terminal::disable_raw_mode().unwrap();
}

fn run() -> Option<()> {
    let mut w = io::stdout();
    println!("{}", TITLE);
    let password = capture_password()?;

    terminal::enable_raw_mode().ok()?;

    // start at
    let mut difficulty = ((password.len() as f32) * STARTING_DIFFICULTY) as usize;
    let mut attempts = STARTING_ATTEMPTS;

    loop {
        terminal::enable_raw_mode().ok()?;
        queue!(
            w,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(0, 0)
        )
        .ok()?;
        if !train_iter(&mut difficulty, &mut attempts, password.as_str()) {
            break
        }

        println!("press any key to continue (q to quit)");
        execute!(w, cursor::Hide).ok()?;
        terminal::enable_raw_mode().unwrap();
        if read_char() {
            break
        }
    }
    Some(())
}

fn train_iter(difficulty: &mut usize, attempts: &mut usize, password: &str) -> bool {
    println!(
        "Difficulty {}/{}, attempts remaining {}",
        difficulty,
        password.len(),
        attempts
    );
    queue!(io::stdout(), cursor::MoveTo(0, 2)).unwrap();
    let obscured_password = hide_difficulty(password, *difficulty);
    let input =
        Text::new(format!("Enter password (hint {})", obscured_password).as_str())
            .prompt()
            .unwrap();

    if input == password {
        println("✅ Success!");
        if *difficulty == password.len() {
            println("Full password length memorized, training complete!");
            return false
        } else {
            // raise difficulty percentage based on number of attempts remaining
            // jump levels = TOTAL_LEVELS * 0.2 * (attempts / max_attempts)
            let difficulty_jump = ((password.len() as f32)
                * (MAX_DIFFICULTY_INCREASE * (*attempts as f32)
                    / (ATTEMPTS_PER_LEVEL as f32)))
                .ceil() as usize;
            // increase difficulty by at least 1 or by the difficuly jump, whichever is greater
            let new_difficulty = max(*difficulty + difficulty_jump, *difficulty + 1);
            // set difficulty to new difficulty or password length, whichever is lesser
            *difficulty = min(password.len(), new_difficulty);
            *attempts = ATTEMPTS_PER_LEVEL;
            println("⏫ Raising difficulty level")
        }
    } else {
        // if first difficulty level, dont adjust attempts
        println(&format!("❌ Incorrect password ({})", input));
        if *difficulty > 1 {
            if *attempts == 0 {
                *difficulty -= 1;
                *attempts = ATTEMPTS_PER_LEVEL;
                println("⏬ Max attempts failed, lowering difficulty level..")
            } else {
                *attempts -= 1;
            }
        }
    }
    true
}

fn capture_password() -> Option<String> {
    Text::new("Enter the password you would like to memorize: ")
        .prompt()
        .ok()
}

// randomly hide characters from the input string based on the `level`
fn hide_difficulty(input: &str, level: usize) -> String {
    let mut output = input.to_string();
    let mut rng = StdRng::from_entropy();
    let mut chars_to_hide = HashSet::<usize>::new();
    while chars_to_hide.len() < level {
        let idx = rng.gen_range(0..input.len());
        chars_to_hide.insert(idx);
    }
    for idx in chars_to_hide {
        output.replace_range((idx)..(idx + 1), "*");
    }
    output
}

pub fn read_char() -> bool {
    loop {
        if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
            return code == KeyCode::Char('q')
        }
    }
}

fn println(s: &str) {
    queue!(io::stdout(), style::Print(s), cursor::MoveToNextLine(1)).unwrap();
}
