extern crate libc;
extern crate clap;


use std::net::{IpAddr, Ipv4Addr};
use std::ptr;
use libc::{ifaddrs, sockaddr, sockaddr_in, AF_INET, getifaddrs, freeifaddrs};
use clap::{App, Arg};

fn is_interface_up(ifa_flags: libc::c_uint) -> bool {
    // Convert libc::IFF_UP to u32 and then perform the bitwise AND operation
    (ifa_flags & libc::IFF_UP as libc::c_uint) != 0
}

fn count_active_interfaces(ifa_list: *mut ifaddrs) -> usize {
    let mut count = 0;
    let mut ifa = ifa_list;

    while !ifa.is_null() {
        let ifa_flags = unsafe { (*ifa).ifa_flags };
        if is_interface_up(ifa_flags) {
            count += 1;
        }
        ifa = unsafe { (*ifa).ifa_next };
    }

    count
}

fn main() {
    let matches = App::new("netnfi")
        .version("1.0")
        .author("Alex Harttree")
        .about("Display network interface information")
        .arg(Arg::with_name("mode")
            .help("Display mode: brief or all")
            .required(true)
            .possible_values(&["brief", "all", "count"])
            .index(1))
        .get_matches();

    let mode = matches.value_of("mode").unwrap();

    let mut ifa_list: *mut ifaddrs = ptr::null_mut();
    
    // Get a linked list of network interfaces and their addresses
    if unsafe { getifaddrs(&mut ifa_list) } == -1 {
        panic!("Failed to get network interfaces");
    }

    let mut ifa = ifa_list;
    while !ifa.is_null() {
        let ifa_name = unsafe { (*ifa).ifa_name };
        let ifa_flags = unsafe { (*ifa).ifa_flags };

        let family = unsafe { (*ifa).ifa_addr.as_ref().unwrap().sa_family as i32 };
        if family == AF_INET {
            let sockaddr_in = unsafe { &*(unsafe { (*ifa).ifa_addr as *const sockaddr_in }) };
            let ip = IpAddr::V4(Ipv4Addr::from(u32::from_be(sockaddr_in.sin_addr.s_addr)));
            let is_up = is_interface_up(ifa_flags);

            match mode {
                "brief" => {
                    println!("Interface: {} - IP Address: {:?}", 
                        unsafe { std::ffi::CStr::from_ptr(ifa_name).to_str().unwrap() }, 
                        ip
                    );
                }
		"count" => {
		    let active_count = count_active_interfaces(ifa_list);
             	    println!("Active Interfaces: {}", active_count);
		}
                "all" => {
                    println!("Interface: {} - IP Address: {:?} - Status: {}", 
                        unsafe { std::ffi::CStr::from_ptr(ifa_name).to_str().unwrap() }, 
                        ip, 
                        if is_up { "Up" } else { "Down" }
                    );
                }
                _ => unreachable!(), // This should never happen due to clap's argument validation
            }
        }

        ifa = unsafe { (*ifa).ifa_next };
    }

    // Clean up
    unsafe { freeifaddrs(ifa_list) };
}
