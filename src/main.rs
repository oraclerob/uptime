#![feature(derive_default_enum)]
#[macro_use]
mod macros;
mod disks;
use disks::Disks;
mod system;
use system::{Memory, SystemInfo, Uptime, Load, Processor};

fn main() {
    //Initialise all our traits
    let mut ut: Uptime = SystemInfo::new();
    let u = ut.info();

    let mut l: Load = SystemInfo::new();
    let ul = l.info();

    let mut pr: Processor = SystemInfo::new();
    let p = pr.info();

    let mut mi: Memory = SystemInfo::new();
    let m = mi.info();


    println!(
        "Uptime: {} Days {} Hours {} Minutes {} Seconds",
        u.uptime_days, u.uptime_hours, u.uptime_minutes, u.uptime_seconds
    );

    println!(
        "Load Average: Current: {:.2} 1 min: {:.2} 5 min: {:.2} 15 min: {:.2} | CPU Usage: {:.2}% | CPU: {} Mhz | CPU Cores: {} ",
        ul.current_load, ul.avg_1_load,ul.avg_5_load,ul.avg_15_load, ul.processor_activity, p.current_mhz, p.number_processors,
    );
    
    println!("Installed RAM: {} MB | Available RAM: {} MB | Total Page : {} MB | Available Page: {} MB | Used Page: {} MB",
            m.installed_ram,m.available_memory,m.page_file_size,m.available_page,m.used_page);

    let mut d = Disks::new();
    unsafe {
        for x in d.info() {
            println!(
                "Drive: {} | Type: {:?} | Size: {} GB | Free: {} GB",
                x.drive, x.drive_type, x.total_gigabytes, x.available_gigabytes
            );
        }
    }
}
