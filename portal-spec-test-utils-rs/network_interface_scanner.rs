use pnet::datalink;

fn main() {
    let interfaces = datalink::interfaces();
    
    println!("Available network interfaces:");
    for interface in interfaces {
        println!("  {}", interface.name);
        if let Some(mac) = interface.mac {
            println!("    MAC: {}", mac);
        }
        for ip in interface.ips {
            println!("    IP: {}", ip);
        }
        println!("    Index: {}", interface.index);
        println!("    Flags: {:?}", interface.flags);
        println!();
    }
}