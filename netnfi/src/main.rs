extern crate libc;
extern crate clap;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ptr;
use libc::{ifaddrs, sockaddr, sockaddr_in, sockaddr_in6, AF_INET, AF_INET6, getifaddrs, freeifaddrs};
use clap::{App, Arg};

fn is_interface_up(ifa_flags: libc::c_uint) -> bool {
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
            .help("Display mode: brief, all, count, or ipv6")
            .required(true)
            .possible_values(&["brief", "all", "count", "ipv6"])
            .index(1))
        .get_matches();

    let mode = matches.value_of("mode").unwrap();

    let mut ifa_list: *mut ifaddrs = ptr::null_mut();

    if unsafe { getifaddrs(&mut ifa_list) } == -1 {
        panic!("Failed to get network interfaces");
    }

    let mut ifa = ifa_list;
    while !ifa.is_null() {
        let ifa_name = unsafe { (*ifa).ifa_name };
        let ifa_flags = unsafe { (*ifa).ifa_flags };

        let family = unsafe { (*ifa).ifa_addr.as_ref().unwrap().sa_family as i32 };
        let is_up = is_interface_up(ifa_flags);

        match mode {
            "brief" => {
                if family == AF_INET {
                    let sockaddr_in = unsafe { &*(unsafe { (*ifa).ifa_addr as *const sockaddr_in }) };
                    let ip = IpAddr::V4(Ipv4Addr::from(u32::from_be(sockaddr_in.sin_addr.s_addr)));
                    println!("Interface: {} - IP Address: {:?}", 
                             unsafe { std::ffi::CStr::from_ptr(ifa_name).to_str().unwrap() }, 
                             ip
                    );
                } else if family == AF_INET6 {
                    let sockaddr_in6 = unsafe { &*(unsafe { (*ifa).ifa_addr as *const sockaddr_in6 }) };
                    let ip = IpAddr::V6(Ipv6Addr::from(sockaddr_in6.sin6_addr.s6_addr));
                    println!("Interface: {} - IPv6 Address: {:?}", 
                             unsafe { std::ffi::CStr::from_ptr(ifa_name).to_str().unwrap() }, 
                             ip
                    );
                }
            }
            "all" => {
                if family == AF_INET {
                    let sockaddr_in = unsafe { &*(unsafe { (*ifa).ifa_addr as *const sockaddr_in }) };
                    let ip = IpAddr::V4(Ipv4Addr::from(u32::from_be(sockaddr_in.sin_addr.s_addr)));
                    println!("Interface: {} - IP Address: {:?} - Status: {}", 
                             unsafe { std::ffi::CStr::from_ptr(ifa_name).to_str().unwrap() }, 
                             ip, 
                             if is_up { "Up" } else { "Down" }
                    );
                } else if family == AF_INET6 {
                    let sockaddr_in6 = unsafe { &*(unsafe { (*ifa).ifa_addr as *const sockaddr_in6 }) };
                    let ip = IpAddr::V6(Ipv6Addr::from(sockaddr_in6.sin6_addr.s6_addr));
                    println!("Interface: {} - IPv6 Address: {:?} - Status: {}", 
                             unsafe { std::ffi::CStr::from_ptr(ifa_name).to_str().unwrap() }, 
                             ip, 
                             if is_up { "Up" } else { "Down" }
                    );
                }
            }
            "count" => {
                let active_count = count_active_interfaces(ifa_list);
                println!("Active Interfaces: {}", active_count);
            }
            "ipv6" => {
                if family == AF_INET6 {
                    let sockaddr_in6 = unsafe { &*(unsafe { (*ifa).ifa_addr as *const sockaddr_in6 }) };
                    let ip = IpAddr::V6(Ipv6Addr::from(sockaddr_in6.sin6_addr.s6_addr));
                    println!("Interface: {} - IPv6 Address: {:?}", 
                             unsafe { std::ffi::CStr::from_ptr(ifa_name).to_str().unwrap() }, 
                             ip
                    );
                }
            }
            _ => unreachable!(),
        }

        ifa = unsafe { (*ifa).ifa_next };
    }

    unsafe { freeifaddrs(ifa_list) };
}
