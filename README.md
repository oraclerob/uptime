This is a Rust project to implement a similar style unix "Uptime" feature for Windows, using Microsoft's own crate windows-rs:

https://github.com/microsoft/windows-rs

Example output:

Uptime: 13 Days 13 Hours 32 Minutes 33 Seconds | Load Average: 0.00 Cores 8 Activity 13.20% Current Mhz 800
Installed RAM: 16384 MB Available RAM: 2778 MB Total Page : 32671 MB Available Page : 4878 MB Used Page: 27793 MB
Drive: C:\ Type: DriveFixed Size: 459 GB Free: 83 GB
Drive: G:\ Type: DriveFixed Size: 1 GB Free: 8 GB

Many projects use the winapi wrappers, however there is virtually no examples using the windows-rs crate so I thought I would provide 
some because the conversions are not straightforward for a novice like me.

Thanks to https://github.com/GuillaumeGomez/sysinfo for providing backbone code and the inspiration.

Any contributions and addtions are most welcome!
