use clap::Parser;
use crossterm::{cursor, terminal, ExecutableCommand, QueueableCommand};
use notify_rust::{Notification, Timeout};
use rodio::{Decoder, OutputStream, Sink};
use std::error::Error;
use std::io::{self, stdout, Stdout, Write};
use std::{thread, time};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Time to work in minutes
    #[arg(short, long, default_value_t = 25)]
    work_time: u32,

    /// Time to take a break in minutes
    #[arg(short, long, default_value_t = 5)]
    break_time: u32,

    /// Number of cycles to run
    #[arg(short, long, default_value_t = 4)]
    cycles: u8,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    intro(&args);

    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();
    notify("Work time!");

    // Main loop
    for i in 1..=args.cycles {
        thread::spawn(|| play_sound("clock.mp3"));
        println!("Work time! Cycle {}", i);
        run_clock(args.work_time, &mut stdout);
        thread::spawn(|| play_sound("success.mp3"));
        notify("Time to take a break!");

        println!("Nice job! Time to take a break");
        run_clock(args.break_time, &mut stdout);

        if i != args.cycles {
            thread::spawn(|| play_sound("endbreak.mp3"));
            notify("Back to work!");
        }

        println!("");
    }

    // TODO: keyboard interrupt to stop the clock / continue to next cycle

    println!("Hope you got everything done! Goodbye!");
    stdout.execute(cursor::Show).unwrap();
    Ok(())
}

fn intro(args: &Args) {
    println!("Welcome to the Pomodoro CLI!");
    println!(
        "You've chosen to work in {} minute intervals with {} minute breaks",
        args.work_time, args.break_time
    );
    println!("This cycle will continue {} times", args.cycles);

    // Wait for user to press enter
    println!("Press enter to start");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

/// Print a countdown clock to the terminal
fn run_clock(minutes: u32, stdout: &mut Stdout) {
    // Output to buffer the remaining time in clock format
    for i in (0..minutes * 60).rev() {
        let seconds = i % 60;
        let minutes = i / 60;

        // Write to the position, format in mm:ss
        stdout.queue(cursor::SavePosition).unwrap();
        stdout
            .write(format!("{:02}:{:02}", minutes, seconds).as_bytes())
            .unwrap();

        stdout.queue(cursor::RestorePosition).unwrap();
        stdout.flush().unwrap();

        // Sleep for a second
        thread::sleep(time::Duration::from_secs(1));

        // Clear the line
        stdout.queue(cursor::RestorePosition).unwrap();
        stdout
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
            .unwrap();
    }
}

fn notify(message: &str) {
    Notification::new()
        .summary("Pomodoro CLI")
        .body(message)
        .icon("pomodoro")
        .appname("pomodoro")
        .timeout(Timeout::Never)
        .show()
        .unwrap();
}

// Use rodio to play an audio sample
fn play_sound(filename: &str) {
    // Get a output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let file = std::fs::File::open(format!("audio/{filename}")).unwrap();
    let source = Decoder::new(file).unwrap();

    sink.append(source);
    sink.sleep_until_end();
}
