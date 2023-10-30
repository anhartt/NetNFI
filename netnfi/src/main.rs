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

fn get_mac_address(interface_name: &str) -> Option<String> {
    let mut ifa_list: *mut ifaddrs = ptr::null_mut();

    if unsafe { getifaddrs(&mut ifa_list) } == -1 {
        panic!("Failed to get network interfaces");
    }

    let mut ifa = ifa_list;
    while !ifa.is_null() {
        let ifa_name = unsafe { (*ifa).ifa_name };
        let ifa_flags = unsafe { (*ifa).ifa_flags };

        if let Some(mac) = extract_mac_address(ifa_name) {
            return Some(mac);
        }

        ifa = unsafe { (*ifa).ifa_next };
    }

    None
}

fn extract_mac_address(interface_name: *const i8) -> Option<String> {
    let mac_path = format!("/sys/class/net/{}/address", unsafe { std::ffi::CStr::from_ptr(interface_name).to_str().unwrap() });
    match std::fs::read_to_string(mac_path) {
        Ok(mac_address) => Some(mac_address.trim().to_string()),
        Err(_) => None,
    }
}

fn main() {
    let matches = App::new("netnfi")
        .version("1.0")
        .author("Alex Harttree")
        .about("Display network interface information")
        .subcommand(
            App::new("show")
                .about("Show network interface information")
                .subcommand(App::new("brief").about("Show brief interface information"))
                .subcommand(App::new("all").about("Show all interface information"))
                .subcommand(App::new("ipv6").about("Show IPv6 interface information"))
                .subcommand(App::new("mac").about("Show MAC address"))
		.subcommand(App::new("activeip").about("Shows the ipv4 address for the active interface"))
        )
        .subcommand(
            App::new("count")
                .about("Count active network interfaces")
                .subcommand(App::new("all").about("Count all active interfaces"))
                // Add more count subcommands here for future expansion
        )
        .get_matches();

    let show_matches = matches.subcommand_matches("show");
    let count_matches = matches.subcommand_matches("count");

    match show_matches {
        Some(subcommand_matches) => {
            if let Some(mode) = subcommand_matches.subcommand_name() {
                match mode {
                    "brief" | "all" | "ipv6" => {
                        let mut ifa_list: *mut ifaddrs = ptr::null_mut();

                        if unsafe { getifaddrs(&mut ifa_list) } == -1 {
                            panic!("Failed to fetch network interface information");
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
                "activeip" => {
                    let mut ifa_list: *mut ifaddrs = ptr::null_mut();

                    if unsafe { getifaddrs(&mut ifa_list) } == -1 {
                        panic!("Failed to fetch network interface information");
                    }

                    let mut ifa = ifa_list;
                    while !ifa.is_null() {
                        let ifa_name = unsafe { (*ifa).ifa_name };
                        let ifa_flags = unsafe { (*ifa).ifa_flags };

                        if is_interface_up(ifa_flags) {
                            let family = unsafe { (*ifa).ifa_addr.as_ref().unwrap().sa_family as i32 };

                            if family == AF_INET {
                                let sockaddr_in = unsafe { &*(unsafe { (*ifa).ifa_addr as *const sockaddr_in }) };
                                let ip = IpAddr::V4(Ipv4Addr::from(u32::from_be(sockaddr_in.sin_addr.s_addr)));
                                println!("{:?}", ip);
                                break; // Print only the first active interface's IPv4 address
                            }
                        }
                        ifa = unsafe { (*ifa).ifa_next };
                    }

                    unsafe { freeifaddrs(ifa_list) };
                }
                    "mac" => {
                        if let Some(mac_address) = get_mac_address("eth0") {
                            println!("MAC Address: {}", mac_address);
                        } else {
                            println!("MAC Address not found for eth0");
                        }
                    }
                    _ => unreachable!(),
                }
            } else {
                println!("Incomplete command. Please specify 'brief', 'all', 'ipv6', or 'mac'.");
            }
        }
        None => {
            println!("Incomplete command. Please specify 'show' and one of: 'brief', 'all', 'ipv6', or 'mac'.");
        }
    }

    match count_matches {
        Some(subcommand_matches) => {
            if let Some(count_mode) = subcommand_matches.subcommand_name() {
                if count_mode == "all" {
                    let mut ifa_list: *mut ifaddrs = ptr::null_mut();

                    if unsafe { getifaddrs(&mut ifa_list) } == -1 {
                        panic!("Failed to get network interfaces");
                    }

                    let active_count = count_active_interfaces(ifa_list);
                    println!("Active Interfaces: {}", active_count);
                    unsafe { freeifaddrs(ifa_list) };
                } else {
                    println!("Invalid count subcommand. Supported subcommands: 'all'.");
                }
            } else {
                println!("Incomplete count command. Please specify a subcommand (e.g., 'all').");
            }
        }
        None => {}
    }
}
