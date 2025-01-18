use std::{io::{stdout, Write}, time::Duration};

use probe_rs::{
    config::TargetSelector,
    probe::list::Lister,
    rtt::{Rtt, RttChannel, ScanRegion},
    Permissions,
};
use tokio::time::sleep;

pub mod impls;

#[tokio::main]
async fn main() {
    let lister = Lister::new();
    let probes = lister.list_all();
    for p in probes.iter() {
        println!("{p:?}");
    }
    let probe = probes[0].open().unwrap();
    let mut session = probe
        .attach(TargetSelector::from("RP2040"), Permissions::default())
        .unwrap();
    let mut core = session.core(0).unwrap();

    eprintln!("Attaching to RTT...");

    let mut rtt = Rtt::attach_region(&mut core, &ScanRegion::Ram).unwrap();
    eprintln!("Found control block at {:#010x}", rtt.ptr());

    println!("Up channels:");
    list_channels(rtt.up_channels());

    println!("Down channels:");
    list_channels(rtt.down_channels());

    let up_channel = 0;
    let down_channel = 0;

    // let stdin = rtt.down_channel(down_channel).unwrap();
    core.reset().unwrap();
    sleep(Duration::from_millis(50)).await;
    let up = rtt.up_channel(0).unwrap();

    let mut buf = [0u8; 1024];
    let mut ctr = 0;
    loop {
        let read = up.read(&mut core, &mut buf).unwrap();
        if read == 0 {
            sleep(Duration::from_millis(5)).await;
            continue;
        }
        for b in &buf[..read] {
            print!("{b:02X} ");
            ctr += 1;
            if ctr >= 16 {
                println!();
                ctr = 0;
            }
        }
        stdout().lock().flush().unwrap();
    }
}

fn list_channels(channels: &[impl RttChannel]) {
    if channels.is_empty() {
        println!("  (none)");
        return;
    }

    for chan in channels.iter() {
        println!(
            "  {}: {} (buffer size {})",
            chan.number(),
            chan.name().unwrap_or("(no name)"),
            chan.buffer_size(),
        );
    }
}
