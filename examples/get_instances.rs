extern crate ms_ssrp;
use ms_ssrp::InstanceInfo;
use std::net::IpAddr;
use std::net::IpAddr::V4;
use std::net::Ipv4Addr;
use std::time::Duration;

fn main() {
    let ip = IpAddr::V4(Ipv4Addr::new(127,0,0,1));
    let lst = InstanceInfo::get_instance_list(ip, 1434, Some(Duration::new(30, 0))).unwrap();
    for instance in &lst {
        println!("result = {:?}", instance);
    }
}