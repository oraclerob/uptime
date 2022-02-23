

use std::mem::{size_of};

use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Foundation::PWSTR;
use windows::Win32::Storage::FileSystem::CreateFileW;
use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
use windows::Win32::Storage::FileSystem::GetDriveTypeW;
use windows::Win32::Storage::FileSystem::GetLogicalDrives;
use windows::Win32::Storage::FileSystem::FILE_ACCESS_FLAGS;
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::Storage::FileSystem::FILE_SHARE_READ;
use windows::Win32::Storage::FileSystem::FILE_SHARE_WRITE;
use windows::Win32::Storage::FileSystem::OPEN_EXISTING;
use windows::Win32::System::Ioctl::IOCTL_DISK_GET_PARTITION_INFO_EX;
use windows::Win32::System::Ioctl::PARTITION_INFORMATION_EX;
use windows::Win32::System::IO::DeviceIoControl;

pub unsafe fn open_drive(drive_name: &[u16], open_rights: FILE_ACCESS_FLAGS) -> HANDLE {
    CreateFileW(
        PWSTR(drive_name.as_ptr()),
        open_rights,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        std::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_FLAGS_AND_ATTRIBUTES(0),
        HANDLE::default(),
    )
}

pub unsafe fn get_drive_size(handle: HANDLE) -> u64 {
    let mut pdg: PARTITION_INFORMATION_EX = std::mem::zeroed();
    let mut junk = 0;
    let result = DeviceIoControl(
        handle,
        IOCTL_DISK_GET_PARTITION_INFO_EX,
        std::ptr::null_mut(),
        0,
        &mut pdg as *mut PARTITION_INFORMATION_EX as *mut std::os::raw::c_void,
        size_of::<PARTITION_INFORMATION_EX>() as u32,
        &mut junk,
        std::ptr::null_mut(),
    );

    if BOOL::as_bool(result) {
       pdg.PartitionLength as u64
    } else {
        0
    }
}

#[derive(Default, Debug, Clone)]
pub struct Disks {
    pub drive: String,
    pub drive_type: DriveType,
    pub total_bytes: u64,
    pub total_gigabytes: u64,
    pub available_bytes: u64,
    pub available_gigabytes: u64,
}
#[warn(non_camel_case_types)]
#[derive(Default, Debug, Clone)]
pub enum DriveType {
    #[default]
    DriveUnknown,
    DriveNoRootDir,
    DriveRemovable,
    DriveFixed,
    DriveRemote,
    DriveCdrom,
    DriveRamdisk,
}

impl DriveType {

    fn get(dt: u32) -> DriveType {
        match dt {
            0 => DriveType::DriveUnknown,
            1 => DriveType::DriveNoRootDir,
            2 => DriveType::DriveRemovable,
            3 => DriveType::DriveFixed,
            4 => DriveType::DriveRemote,
            5 => DriveType::DriveCdrom,
            6 => DriveType::DriveRamdisk,
            _ => DriveType::default(),
        }
    }
}

impl Disks {
    pub fn new() -> Self {
        return Self::default();
    }

    pub unsafe fn info(&mut self) -> Vec<Disks> {
        let mut fs: Vec<String> = vec![];
        let mut disks: Vec<Disks> = vec![];

        for x in 1..GetLogicalDrives() {
            let mount_point = [b'A' as u16 + (x - 1) as u16, b':' as u16, b'\\' as u16, 0];
            //Remove the extra 0 so we can push the an extra /
            let str_mount_point = String::from_utf16_lossy(&mount_point[..&mount_point.len() - 1]).to_uppercase();

            let drive_type = GetDriveTypeW(str_mount_point.clone());

            if fs.contains(&str_mount_point) {
                //duplicate lowercase drive letter so break
                break;
            }

            fs.push(str_mount_point.clone());

            match GetDriveTypeW(str_mount_point.clone()) {
                2 | 3 => {
                    let drive_name = [
                        b'\\' as u16,
                        b'\\' as u16,
                        b'.' as u16,
                        b'\\' as u16,
                        b'A' as u16 + (x - 1) as u16,
                        b':' as u16,
                        0,
                    ];

                    let handle = open_drive(&drive_name, FILE_ACCESS_FLAGS(0));

                    self.total_bytes = get_drive_size(handle);
                    self.total_gigabytes = self.total_bytes / (1024 * 1024 * 1024);

                    let mut tmp: u64 = std::mem::zeroed();

                    GetDiskFreeSpaceExW(
                        str_mount_point.clone(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        &mut tmp,
                    );

                    self.available_bytes = tmp;
                    self.available_gigabytes = self.available_bytes / (1024 * 1024 * 1024);
                    self.drive = str_mount_point;
                    self.drive_type = DriveType::get(drive_type);

                    disks.push(self.to_owned());
                }
                _ => {}
            };
        }

        return disks;
    }
}
