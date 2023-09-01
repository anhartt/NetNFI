extern crate libc;

use std::net::{IpAddr, Ipv4Addr};
use std::ptr;
use libc::{ifaddrs, sockaddr, sockaddr_in, AF_INET, getifaddrs, freeifaddrs};

fn main() {
    let mut ifa_list: *mut ifaddrs = ptr::null_mut();
    
    // Get a linked list of network interfaces and their addresses
    if unsafe { getifaddrs(&mut ifa_list) } == -1 {
        panic!("Failed to get network interfaces");
    }

    let mut ifa = ifa_list;
    while !ifa.is_null() {
        let ifa_name = unsafe { (*ifa).ifa_name };
        
        let family = unsafe { (*ifa).ifa_addr.as_ref().unwrap().sa_family as i32 };
        if family == AF_INET {
            let sockaddr_in = unsafe { &*(unsafe { (*ifa).ifa_addr as *const sockaddr_in }) };
            let ip = IpAddr::V4(Ipv4Addr::from(u32::from_be(sockaddr_in.sin_addr.s_addr)));
            println!("Interface: {} - IP Address: {:?}", unsafe { std::ffi::CStr::from_ptr(ifa_name).to_str().unwrap() }, ip);
        }

        ifa = unsafe { (*ifa).ifa_next };
    }

    // Clean up
    unsafe { freeifaddrs(ifa_list) };
}
