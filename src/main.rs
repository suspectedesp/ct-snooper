extern crate chrono;
extern crate crossterm;

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write, Result};
use structopt::StructOpt;
use chrono::Local;

#[derive(Debug, StructOpt)]
#[structopt(name = "tag_searcher", about = "Search for LuaScript and Code tags in .ct files")]
struct Opt {
    /// Path to ct file
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn set_console_title(title: &str) {
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "title", title])
            .output()
            .expect("Failed to set console title");
    }

    #[cfg(not(windows))]
    {
        print!("\x1b]0;{}\x07", title);
        std::io::stdout().flush().expect("Failed to flush stdout");
    }
}

fn attribute_value(line: &str, attribute: &str) -> Option<String> {
    line.split(&format!("{}=", attribute))
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .map(|s| s.trim_matches('"').to_string())
}

fn log_message(log_file: &mut File, message: &str) -> Result<()> {
    println!("{}", message);
    writeln!(log_file, "{}", message)?;
    Ok(())
}

fn search_tags(file_path: &std::path::Path, log_file: &mut File) -> Result<()> {
    let current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let ct_version = 45;
    let mut line_count = 0;
    //let mut cheatentry_count = 0;
    let mut code_count = 0;
    let mut structure_count = 0;
    let mut structure_list: Vec<String> = Vec::new();
    let mut code_list: Vec<String> = Vec::new();

    let term_size = crossterm::terminal::size();
    let mut term_width = term_size.unwrap_or((80, 24)).0 as usize;
    term_width -= 1;
    log_message(log_file, &format!("{:-^term_width$}", current_time))?;

    let opt = Opt::from_args();
    let file_name = &opt.path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown file");
    log_message(log_file, &format!("Filename: {}", file_name))?;
    log_message(log_file, &format!("Made for CT Version {}", ct_version))?;

    for (line_number, line) in reader.lines().enumerate() {
        line_count += 1;
        let line = line?;

        if let Some(version) = attribute_value(&line, "encoding") {
            log_message(log_file, &format!("Encoding: {}", &version[..version.len() - 3]))?;
        }

        if let Some(version) = attribute_value(&line, "CheatEngineTableVersion") {
            log_message(log_file, &format!("CT Version: v{}", &version[..version.len() - 2]))?;
        }

        if let Some(version) = attribute_value(&line, "StructVersion") {
            log_message(log_file, &format!("Structures Version: v{}", &version[..version.len() - 2]))?;
        }

        if line.contains("<LuaScript") {
            code_count += 1;
            log_message(log_file, &format!("Found LuaScript on line {}", line_number + 1))?;
            code_list.push(line);
        } else if line.contains("<Code") {
            code_count += 1;
            log_message(log_file, &format!("Found Code on line {}", line_number + 1))?;
            code_list.push(line);
        } else if line.contains("<AssemblerScript") {
            code_count += 1;
            log_message(log_file, &format!("Found AssemblerScript on line {}", line_number + 1))?;
            code_list.push(line);
        } else if line.contains("Structure Name") {
            structure_count += 1;
            log_message(log_file, &format!("Found Structure no. {} on line {}", structure_count, line_number + 1))?;
            structure_list.push(line);
        } /*
        else if line.contains("<CheatEntry") {
            cheatentry_count += 1;
            log_message(log_file, &format!("Found CheatEntry no. {} on line {}", cheatentry_count, line_number + 1))?;
        }
        */
    }

    let text = "Summary";
    log_message(log_file, &format!("{:-^term_width$}", text))?;

    log_message(log_file, &format!("Total lines checked: {}", line_count))?;
    log_message(log_file, &format!("Total amount of Structures found: {}", structure_count))?;
    //log_message(log_file, &format!("Total amount of CheatEntries found: {}", cheatentry_count))?;
    log_message(log_file, &format!("Total amount of Scripts/Code found: {}", code_count))?;


    if structure_count <= 0 {
        log_message(log_file, "No structures found.")?;
        return Ok(());
    }

    println!("{:-^term_width$}", "-");
    println!("Would you like to see all found structures? [y/n]");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let input = input.trim();
    if input.is_empty() {
        println!("Input was empty, return");
    } else {
        if input == "y" {
            for item in &structure_list {
                println!("{}", item);
            }
        }
    }

    Ok(())
}

fn main() {
    let opt = Opt::from_args();
    let file_name = &opt.path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown file");
    set_console_title("CT Snooper: made in Rust");

    let log_file_name = format!("{}_log.txt", file_name);
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_name)
        .expect("Failed to open log file");

    if let Err(e) = search_tags(&opt.path, &mut log_file) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    println!("Press Enter to exit...");
    let mut _input = String::new();
    io::stdin().read_line(&mut _input).expect("Failed to read line");
}
