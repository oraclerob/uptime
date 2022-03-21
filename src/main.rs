#![feature(derive_default_enum)]
#[macro_use]
mod macros;
mod disks;

use disks::Disks;
mod system;
use system::{Memory, SystemInfo, Uptime, Load, Processor};
use colored::*;

fn main() {
    //Make sure ansi support works in terminal
    let _ = ansi_term::enable_ansi_support();
    
    //Initialise all our traits
    let mut ut: Uptime = SystemInfo::new();
    let u = ut.info();

    println!(
        "Uptime: {} Days {} Hours {} Minutes {} Seconds",
        u.uptime_days.to_string().blue().bold(), u.uptime_hours.to_string().blue().bold(), u.uptime_minutes.to_string().blue().bold(), u.uptime_seconds.to_string().blue().bold()
    );

    let mut pr: Processor = SystemInfo::new();
    let p = pr.info();

    let mut mi: Memory = SystemInfo::new();
    let m = mi.info();
    println!("Installed RAM: {} MB | Available RAM: {} MB | Total Page : {} MB | Available Page: {} MB | Used Page: {} MB",
    m.installed_ram.to_string().green().bold(),m.available_memory.to_string().red().bold(),m.page_file_size.to_string().green().bold(),m.available_page.to_string().red().bold(),m.used_page);

    let mut d = Disks::new();
    unsafe {
        for x in d.info() {
            println!(
                "Drive: {} | Type: {:?} | Size: {} GB | Free: {} GB",
                x.drive, x.drive_type, x.total_gigabytes.to_string().green().bold(), x.available_gigabytes.to_string().red().bold()
            );
        }
    }
    
    let mut l: Load = SystemInfo::new();
    let ul = l.info();
    print!(
        "Load Average: Current: {:.2} 1 min: {:.2} 5 min: {:.2} 15 min: {:.2} | CPU Usage: {:.2}% | CPU: {} Mhz | CPU Cores: {} ",
        ul.current_load.to_string().green().bold(), ul.avg_1_load,ul.avg_5_load,ul.avg_15_load, ul.processor_activity.round().to_string().red().bold(), p.current_mhz, p.number_processors,
    );
    
   
}
