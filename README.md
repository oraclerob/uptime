This is a Rust project to implement a similar style unix "Uptime" feature for Windows, using Microsoft's own crate windows-rs:

https://github.com/microsoft/windows-rs

Example output:
![image](https://user-images.githubusercontent.com/11176397/159222994-404aeea0-fa86-4199-ad40-f564686a5d0f.png)

Many projects use the winapi wrappers, however there is virtually no examples using the windows-rs crate so I thought I would provide 
some because the conversions are not straightforward for a novice like me.

Thanks to https://github.com/GuillaumeGomez/sysinfo for providing backbone code and the inspiration.

Any contributions and addtions are most welcome!
