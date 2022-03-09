use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Foundation::PSTR;
use windows::Win32::System::Performance::PdhAddEnglishCounterA;
use windows::Win32::System::Performance::PdhCloseQuery;
use windows::Win32::System::Performance::PdhCollectQueryData;
use windows::Win32::System::Performance::PdhCollectQueryDataEx;
use windows::Win32::System::Performance::PdhGetFormattedCounterValue;
use windows::Win32::System::Performance::PdhOpenQueryA;
use windows::Win32::System::Performance::PdhRemoveCounter;
use windows::Win32::System::Performance::PDH_FMT;
use windows::Win32::System::Performance::PDH_FMT_COUNTERVALUE;
use windows::Win32::System::Power::CallNtPowerInformation;
use windows::Win32::System::Power::POWER_INFORMATION_LEVEL;
use windows::Win32::System::SystemInformation::GetPhysicallyInstalledSystemMemory;
use windows::Win32::System::SystemInformation::GetSystemInfo;
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::System::SystemInformation::GlobalMemoryStatusEx;
use windows::Win32::System::SystemInformation::MEMORYSTATUSEX;
use windows::Win32::System::SystemInformation::SYSTEM_INFO;
use windows::Win32::System::Threading::CreateEventA;
use windows::Win32::System::Threading::WaitForSingleObject;
use windows::Win32::System::Threading::WAIT_OBJECT_0;

use ntapi::ntpoapi::PROCESSOR_POWER_INFORMATION;

use std::error::Error;
use std::mem::MaybeUninit;
use std::mem::{size_of, zeroed};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

const FALSE: BOOL = BOOL(0);
//const TRUE:BOOL = BOOL(1);
const INFINITE: u32 = 4294967295u32;

//Credits to https://github.com/GuillaumeGomez/sysinfo
// This formula comes from linux's include/linux/sched/loadavg.h
// https://github.com/torvalds/linux/blob/345671ea0f9258f410eb057b9ced9cefbbe5dc78/include/linux/sched/loadavg.h#L20-L23
//const SAMPLING_INTERVAL: u32 = 5;
// #[allow(clippy::excessive_precision)]
// const LOADAVG_FACTOR_1F: f64 = 0.9200444146293232478931553241;
// #[allow(clippy::excessive_precision)]
// const LOADAVG_FACTOR_5F: f64 = 0.9834714538216174894737477501;
// #[allow(clippy::excessive_precision)]
// const LOADAVG_FACTOR_15F: f64 = 0.9944598480048967508795473394;
// The time interval in seconds between taking load counts, same as Linux

//This is the 2 second average calculations - above are for 5 second intervals
//Use 2 seconds for now - so the result displays quicker
#[allow(clippy::excessive_precision)]
const LOADAVG_FACTOR_1F: f64 = 0.9672161004820059020409731093;
#[allow(clippy::excessive_precision)]
const LOADAVG_FACTOR_5F: f64 = 0.9933555062550344153694146759;
#[allow(clippy::excessive_precision)]
const LOADAVG_FACTOR_15F: f64 = 0.9977802450856064174364805839;

#[derive(Default, Debug, Clone)]
pub(crate) struct Uptime {
    pub(crate) uptime_ms: u64,
    pub(crate) uptime_days: u64,
    pub(crate) uptime_hours: u64,
    pub(crate) uptime_minutes: u64,
    pub(crate) uptime_seconds: u64,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct Memory {
    pub(crate) installed_ram: u64,
    pub(crate) available_memory: u64,
    pub(crate) page_file_size: u64,
    pub(crate) available_page: u64,
    pub(crate) used_page: u64,
}

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct Load {
    pub(crate) processor_activity: f64,
    pub(crate) current_load: f64,
    pub(crate) avg_1_load: f64,
    pub(crate) avg_5_load: f64,
    pub(crate) avg_15_load: f64,
}
#[derive(Default, Debug, Clone)]
struct LoadAvg(f64, f64, f64);

#[derive(Default, Debug, Clone)]
pub(crate) struct Processor {
    pub(crate) current_mhz: u64,
    pub(crate) number_processors: u32,
}

pub trait SystemInfo {
    fn new() -> Self;
    fn info(&mut self) -> &mut Self;
}

impl SystemInfo for Processor {
    fn new() -> Self {
        return Self::default();
    }

    fn info(&mut self) -> &mut Self {
        unsafe {
            match get_processors() {
                (Some(po), Some(cm)) => {
                    self.number_processors = po.dwNumberOfProcessors;
                    //We know the vec is populated, give as the first element all Intel cores have same speed (unlike ARM)
                    self.current_mhz = cm[0].CurrentMhz as u64;
                }
                (None, None) => {
                    self.number_processors = 0_u32;
                    self.current_mhz = 0_u64;
                }
                (Some(po), None) => {
                    self.number_processors = po.dwNumberOfProcessors;
                    self.current_mhz = 0_u64;
                }
                (None, Some(cm)) => {
                    self.number_processors = 0_u32;
                    self.current_mhz = cm[0].CurrentMhz as u64;
                }
            };
        }
        return self;
    }
}

fn get_load_averages(first_load: f64, second_load: f64) -> (f64, f64, f64) {
    //Calculations derived from this article - looks like we need the previous load to calculate average
    //https://www.linuxjournal.com/article/9001
    let avg_one = (LOADAVG_FACTOR_1F + first_load * (1.0 - LOADAVG_FACTOR_1F))
        * (LOADAVG_FACTOR_1F + second_load * (1.0 - LOADAVG_FACTOR_1F));
    let avg_five = (LOADAVG_FACTOR_5F + first_load * (1.0 - LOADAVG_FACTOR_5F))
        * (LOADAVG_FACTOR_5F + second_load * (1.0 - LOADAVG_FACTOR_5F));
    let avg_fifteen = (LOADAVG_FACTOR_5F + first_load * (1.0 - LOADAVG_FACTOR_5F))
        * (LOADAVG_FACTOR_15F + second_load * (1.0 - LOADAVG_FACTOR_15F));

    return (avg_one, avg_five, avg_fifteen);
}

impl SystemInfo for Load {
    fn new() -> Self {
        return Self::default();
    }

    fn info(&mut self) -> &mut Self {
        unsafe {
            let data = Arc::new(Mutex::new(Load::default()));
            let (tx, rx): (Sender<Load>, Receiver<Load>) = channel();
            let (data1, tx1) = (Arc::clone(&data), tx.clone());
            let (data2, tx2) = (Arc::clone(&data), tx.clone());

            //let start = std::time::Instant::now();
            
            let h1 = thread::spawn(move || {

                let mut data1 = data1.lock().unwrap();
                match get_processor_activity() {
                    Ok(pa) => data1.processor_activity = pa,
                    Err(_) => data1.processor_activity = 0_f64,
                };
                
                tx1.send(*data1).unwrap();

            });

            let h2 = thread::spawn(move || {
                let mut data2 = data2.lock().unwrap();
                match get_load_activity() {
                    Ok(loadnow) => {
                        //Of the 2 load queries 5 seconds apart this is the most recent
                        data2.current_load = loadnow[1];
                        (data2.avg_1_load, data2.avg_5_load, data2.avg_15_load) =
                            get_load_averages(loadnow[0], loadnow[1]);
                    }
                    Err(_) => data2.current_load = 0_f64,
                };

                tx2.send(*data2).unwrap();
            });

            let _ = h1.join();
            let _ = h2.join();
            drop(tx);
           
            //Take the last message as this will be populated with all values
            for ret in rx {
                self.processor_activity = ret.processor_activity;
                self.current_load = ret.current_load;
                self.avg_1_load = ret.avg_1_load;
                self.avg_5_load = ret.avg_5_load;
                self.avg_15_load = ret.avg_15_load;
            }

            //p!(start.elapsed());
        }

        return self;
    }
}

impl SystemInfo for Uptime {
    fn new() -> Self {
        return Self::default();
    }

    fn info(&mut self) -> &mut Self {
        unsafe {
            self.uptime_ms = GetTickCount64();
            self.uptime_days = (self.uptime_ms) / (1000 * 60 * 60) / 24;
            self.uptime_seconds = (self.uptime_ms / 1000) % 60;
            self.uptime_minutes = self.uptime_ms / (1000 * 60) % 60;
            self.uptime_hours = (self.uptime_ms / (1000 * 3600)) % 24;
        }
        return self;
    }
}

fn return_error<T>(msg: &str, reference: &String) -> Result<T, Box<dyn Error>> {
    let mut err_str = String::new();
    err_str.push_str(msg);
    err_str.push_str(&reference);
    println!("{}", err_str);
    return Err(err_str.into());
}

unsafe fn open_query() -> Option<isize> {
    let sz_datasource = PSTR(std::mem::zeroed());
    let mut px_query: isize = std::mem::zeroed();

    if PdhOpenQueryA(sz_datasource, 0, &mut px_query) != 0 {
        None
    } else {
        Some(px_query)
    }
}

unsafe fn add_counter(px_query: isize, counter_path: PSTR) -> Option<isize> {
    let mut counter: isize = std::mem::zeroed();

    if PdhAddEnglishCounterA(px_query as isize, counter_path, 0, &mut counter) == 0 {
        Some(counter)
    } else {
        None
    }
}

unsafe fn collect_data_interval(px_query: isize, interval: u32) -> HANDLE {
    let event = CreateEventA(
        std::ptr::null_mut(),
        FALSE,
        FALSE,
        PSTR(b"LoadUpdateEvent\0" as *const u8),
    );

    PdhCollectQueryDataEx(px_query, interval, event);

    return event;
}

unsafe fn collect_data_now(px_query: isize) -> i32 {
    let _ = CreateEventA(
        std::ptr::null_mut(),
        FALSE,
        FALSE,
        PSTR(b"LoadUpdateEvent\0" as *const u8),
    );

    PdhCollectQueryData(px_query)
}

unsafe fn get_processors() -> (
    Option<SYSTEM_INFO>,
    Option<Vec<PROCESSOR_POWER_INFORMATION>>,
) {
    let mut system_info = SYSTEM_INFO::default();
    GetSystemInfo(&mut system_info);

    let size = system_info.dwNumberOfProcessors
        * std::mem::size_of::<PROCESSOR_POWER_INFORMATION>() as u32;
    let mut proc_info: Vec<PROCESSOR_POWER_INFORMATION> =
        Vec::with_capacity(system_info.dwNumberOfProcessors as usize);
    CallNtPowerInformation(
        POWER_INFORMATION_LEVEL(11),
        std::ptr::null_mut(),
        0,
        proc_info.as_mut_ptr() as _,
        size as _,
    );

    proc_info.set_len(system_info.dwNumberOfProcessors as usize);

    return (Some(system_info), Some(proc_info));
}

unsafe fn get_formatted_counter_value(counter_query: isize) -> f64 {
    let dwformat = PDH_FMT(0x00000200);
    let _lpdwtype: *mut u32 = std::ptr::null_mut();
    let mut p_value = MaybeUninit::<PDH_FMT_COUNTERVALUE>::uninit();

    PdhGetFormattedCounterValue(
        counter_query as _,
        dwformat,
        std::ptr::null_mut(),
        p_value.as_mut_ptr(),
    );

    let display_value = p_value.assume_init();
    let current_value = display_value.Anonymous.doubleValue;

    return current_value;
}

pub unsafe fn get_processor_activity() -> Result<f64, Box<dyn Error>> {
    match open_query() {
        Some(px_query) => {
            match add_counter(
                px_query,
                PSTR(b"\\Processor(_Total)\\% Processor Time\0" as *const u8),
            ) {
                Some(proc_query) => {
                    collect_data_now(px_query);
                    let mut val = get_formatted_counter_value(proc_query);

                    //Lets get 2 samples
                    //https://docs.microsoft.com/en-us/windows/win32/api/pdh/nf-pdh-pdhcollectquerydataex
                    let event = collect_data_interval(px_query, 1);
                    let wait_result = WaitForSingleObject(event, INFINITE);

                    if wait_result == WAIT_OBJECT_0 {
                        val = get_formatted_counter_value(proc_query);
                    }

                    PdhRemoveCounter(proc_query);
                    PdhCloseQuery(px_query as isize);
                    return Ok(val);
                }
                None => {
                    return return_error("Add counter failed", &"PdhAddEnglishCounterA".to_string())
                }
            };
        }
        None => return Err("Open Query failed".into()),
    };
}

pub unsafe fn get_load_activity() -> Result<Vec<f64>, Box<dyn Error>> {
    match open_query() {
        Some(px_query) => {
            match add_counter(
                px_query,
                PSTR(b"\\System\\Processor Queue Length\0" as *const u8),
            ) {
                Some(proc_query) => {
                    let mut val_second = 0_f64;

                    collect_data_now(px_query);
                    let val_first = get_formatted_counter_value(proc_query);
                    
                    //Lets get 2 samples
                    //https://docs.microsoft.com/en-us/windows/win32/api/pdh/nf-pdh-pdhcollectquerydataex
                    let event = collect_data_interval(px_query, 2);
                    let wait_result = WaitForSingleObject(event, INFINITE);

                    if wait_result == WAIT_OBJECT_0 {
                        val_second = get_formatted_counter_value(proc_query);
                    }
                
                    PdhRemoveCounter(proc_query);
                    PdhCloseQuery(px_query as isize);
                    return Ok(vec![val_first, val_second]);
                }
                None => {
                    return return_error("Open query failed", &"PdhAddEnglishCounterA".to_string())
                }
            };
        }
        None => return return_error("Open query failed", &"PdhAddEnglishCounterA".to_string()),
    };
}

impl SystemInfo for Memory {
    fn new() -> Memory {
        return Memory::default();
    }

    fn info(&mut self) -> &mut Self {
        let mut m: u64 = 0;

        unsafe {
            let mut mem_info: MEMORYSTATUSEX = zeroed();
            let data_ptr: *mut MEMORYSTATUSEX = &mut mem_info as *mut MEMORYSTATUSEX;
            GetPhysicallyInstalledSystemMemory(&mut m);
            mem_info.dwLength = size_of::<MEMORYSTATUSEX>() as u32;
            GlobalMemoryStatusEx(data_ptr);
            self.installed_ram = m / (1024);
            self.available_memory = (*data_ptr).ullAvailPhys / (1024 * 1024);
            if ((*data_ptr).ullTotalPageFile / (1024 * 1024)) < self.installed_ram {
                self.page_file_size = (*data_ptr).ullTotalPageFile / (1024 * 1024)
            } else {
                self.page_file_size =
                    (*data_ptr).ullTotalPageFile / (1024 * 1024) - self.installed_ram;
            }
            if (*data_ptr).ullAvailPageFile / (1024 * 1024) < self.installed_ram {
                self.available_page = 0_u64;
            } else {
                self.available_page =
                    (*data_ptr).ullAvailPageFile / (1024 * 1024) - self.installed_ram;
            }
            self.used_page = self.page_file_size - self.available_page;

            let mut _power_info: POWER_INFORMATION_LEVEL = zeroed();
            let _inputbuffer: *const ::core::ffi::c_void = std::ptr::null();
            let _outputbuffer: *mut ::core::ffi::c_void = std::mem::zeroed();
            let _outputbufferlength = 0_u32;
            let _inputbufferlength = 0_u32;
        }

        return self;
    }
}
