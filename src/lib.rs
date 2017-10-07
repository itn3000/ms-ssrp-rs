#[macro_use]
extern crate log;

use std::net::UdpSocket;
use std::io::Error;
use std::net::IpAddr;
use std::net::{Ipv4Addr, Ipv6Addr};
// use std::net::IpAddr::{V4, V6};
use std::net::SocketAddr;
use std::time::Duration;
use std::io::ErrorKind;
// use std::io::ErrorKind::Other;
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct InstanceInfo {
    pub server_name: String,
    pub instance_name: String,
    pub is_clustered: bool,
    pub version: String,
    pub protocol_info: HashMap<String, String>
}
impl InstanceInfo {
    pub fn get_instance(ip: IpAddr, port: u16, timeout: Option<Duration>, name: &str) -> Result<InstanceInfo, Error> {
        let reqdata = Self::get_specified_instance_request_data(name);
        let lst = Self::get_instance_list_internal(ip, port, timeout, &reqdata)?;
        if lst.len() > 0 {
            Ok(lst[0].clone())
        }else {
            Err(Error::new(ErrorKind::Other, "cannot find instance"))
        }
    }
    pub fn get_instance_list(
        ip: IpAddr,
        port: u16,
        timeout: Option<Duration>,
    ) -> Result<Vec<InstanceInfo>, Error> {
        let reqdata = Self::get_instance_list_request_data();
        Self::get_instance_list_internal(ip, port, timeout, &reqdata)
    }
    fn get_instance_list_internal(ip: IpAddr, port: u16, timeout: Option<Duration>, reqdata: &[u8]) -> Result<Vec<InstanceInfo>, Error> {
        debug!("creating local udp socket");
        let sock = Self::create_local_udp_socket(ip.is_ipv4())?;
        debug!("timeouts");
        sock.set_read_timeout(timeout)?;
        sock.set_write_timeout(timeout)?;
        debug!("bind to {:?}", sock);
        Self::connect_server(&sock, ip, port)?;
        trace!("sending data: {:?}", reqdata);
        sock.send(&reqdata)?;
        debug!("receiving data");
        let recvdata = Self::read_server_response_bytes(&sock)?;
        trace!("receiving data done:{:?}", recvdata);
        // data response body begins fourth byte
        Ok(Self::deserialize_server_response(&recvdata[3..])?)
    }
    fn connect_server(sock: &UdpSocket, remote_ip: IpAddr, remote_port: u16) -> Result<(), Error> {
        let remotesockaddr = SocketAddr::new(remote_ip, remote_port);
        debug!("connecting server: {:?}", remotesockaddr);
        Ok(sock.connect(remotesockaddr)?)
    }
    fn deserialize_server_response(data: &[u8]) -> Result<Vec<InstanceInfo>, Error> {
        let mut ret: Vec<InstanceInfo> = Vec::new();
        let datastr = String::from_utf8_lossy(data);
        for instancestr in datastr.split(";;") {
            trace!("instancestr is {}", instancestr);
            if instancestr.is_empty() {
                break;
            }
            let mut param_name : &str = "";
            let mut server_name : String = String::new();
            let mut instance_name : String = String::new();
            let mut is_clustered : bool = false;
            let mut version : String = String::new();
            let mut protocols : HashMap<String, String> = HashMap::new();
            for s in instancestr.split(";") {
                if param_name.is_empty() {
                    param_name = s;
                }else{
                    if "ServerName" == param_name {
                        server_name = String::from(s);
                    }
                    else if "InstanceName" == param_name {
                        instance_name = String::from(s);
                    }
                    else if "IsClustered" == param_name {
                        is_clustered = s == "Yes";
                    }
                    else if "Version" == param_name {
                        version = String::from(s);
                    }
                    else if "np" == param_name {
                        protocols.insert(String::from("np"), String::from(s));
                    }
                    else if "tcp" == param_name {
                        protocols.insert(String::from("tcp"), String::from(s));
                    }
                    param_name = "";
                }
            }
            ret.push(InstanceInfo {
                server_name : server_name,
                instance_name : instance_name,
                is_clustered : is_clustered,
                version : version,
                protocol_info : protocols
            });
        }
        Ok(ret)
    }

    fn read_server_response_bytes(sock: &UdpSocket) -> Result<Vec<u8>, Error> {
        const SVR_RESP_ID : u8 = 0x5;
        let mut ret: Vec<u8> = Vec::new();
        ret.reserve(512);
        let mut recvbuf = [0u8; 512];
        let mut is_header_received: bool = false;
        let mut datalength: i32 = -1;
        while datalength < 0 || ret.len() < datalength as usize {
            trace!("current data length:{},total = {}", ret.len(), datalength);
            let bytesreceived = sock.recv(&mut recvbuf)?;
            trace!("bytes received:{}", bytesreceived);
            ret.extend_from_slice(&recvbuf[0..bytesreceived]);
            if !is_header_received && ret.len() >= 1 {
                if ret[0] != SVR_RESP_ID {
                    return Err(Error::new(ErrorKind::Other, "invalid server response header id"));
                }
                is_header_received = true;
            }
            if datalength < 0 && ret.len() >= 3 {
                // ushort little endian
                datalength = ((ret[2] as i32) << 8) | ret[1] as i32;
            }
        }
        Ok(ret)
    }
    fn create_local_udp_socket(is_ipv4: bool) -> Result<UdpSocket, Error> {
        let localsockaddr = SocketAddr::new(
            match is_ipv4 {
                true => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                false => IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
            },
            0,
        );
        let sock = UdpSocket::bind(localsockaddr)?;
        Ok(sock)
    }
    fn get_specified_instance_request_data(name: &str) -> Vec<u8> {
        const CLNT_UCAST_INST_ID: u8 = 0x04;
        let mut ret = vec!(CLNT_UCAST_INST_ID);
        if name.len() <= 32 {
            ret.extend(name.as_bytes());
        }else {
            let bytes = name.as_bytes();
            ret.extend(&bytes[0..32]);
        }
        // must be terminated in '\0'
        ret.push(0);
        ret
    }
    fn get_instance_list_request_data() -> Vec<u8> {
        const CLNT_UCAST_EX_ID: u8 = 0x03;
        vec![CLNT_UCAST_EX_ID]
    }
}

#[cfg(test)]
mod tests {
    extern crate env_logger;
    use std::sync::Once;
    use std::sync::ONCE_INIT;
    use std::time::Duration;
    use super::InstanceInfo;
    use std::net::Ipv4Addr;
    use std::net::IpAddr::V4;
    use std::net::IpAddr;
    static INIT: Once = ONCE_INIT;
    fn setup() {
        INIT.call_once(|| {
            env_logger::init().unwrap();
        });
    }
    #[test]
    fn get_instances() {
        setup();
        let ip = IpAddr::V4(Ipv4Addr::new(127,0,0,1));
        let lst = InstanceInfo::get_instance_list(ip, 1434, Some(Duration::new(30, 0))).unwrap();
        for instance in &lst {
            debug!("result = {:?}", instance);
        }
        assert!(lst.len() != 0);
    }
    #[test]
    fn get_specified_instance() {
        setup();
        let ip = IpAddr::V4(Ipv4Addr::new(127,0,0,1));
        let ret = InstanceInfo::get_instance(ip, 1434, Some(Duration::new(30, 0)), "SQLEXPRESS").unwrap();
        assert!(ret.instance_name == "SQLEXPRESS");
    }
}
