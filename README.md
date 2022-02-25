This is a Rust project to implement a similar style unix "Uptime" feature for Windows, using Microsoft's own crate windows-rs:

https://github.com/microsoft/windows-rs

Example output:

Load Average: Current: 46.00 1 min: 7.99 5 min: 1.89 15 min: 1.60 | CPU Usage: 100.00% | CPU: 1700 Mhz | CPU Cores: 8<br/>
Installed RAM: 16384 MB | Available RAM: 2595 MB | Total Page : 32671 MB | Available Page: 0 MB | Used Page: 32671 MB<br/>
Drive: C:\ | Type: DriveFixed | Size: 459 GB | Free: 82 GB<br/>
Drive: G:\ | Type: DriveFixed | Size: 1 GB | Free: 8 GB

Many projects use the winapi wrappers, however there is virtually no examples using the windows-rs crate so I thought I would provide 
some because the conversions are not straightforward for a novice like me.

Thanks to https://github.com/GuillaumeGomez/sysinfo for providing backbone code and the inspiration.

Any contributions and addtions are most welcome!
