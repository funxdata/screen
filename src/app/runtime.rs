use std::thread;
use std::time::Duration;
use std::io;
use crate::service::sensor::sensor;
#[allow(dead_code)]
pub fn hw_sensor() { 
    thread::spawn(watch_sensor);
    // println!("{:?}",web.close_devtools());
}

fn watch_sensor(){
    let port_name = "COM3";
    let baud_rate = 9600;
    let port = serialport::new(port_name, baud_rate)
    .timeout(Duration::from_millis(10))
    .open();
    match port {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1];
            // println!("Receiving data on {} at {} baud:", &port_name, &baud_rate);
            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                       let buf =String::from_utf8_lossy(&serial_buf[..t]).to_string();
                       let mut signals = sensor::new();
                       signals.change(buf);
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
                thread::sleep(Duration::from_millis(10));   
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }
}

mod tests {

    // 测试初始化
    #[test]
    fn test_stone_do() {
        println!("......");
        // let st = runtime::hw_sensor();
        // println!("stones:{:?}",st);
        // println!("stones:{:?}",st.src);
        println!("..................................");
        // 
        // let data = art_goods.search("ys");
        // println!("{:?}",data);
    }
}