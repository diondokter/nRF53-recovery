use anyhow::Result;
use probe_rs::{architecture::arm::ArmProbeInterface, Probe};

fn main() -> Result<()> {
    // Get a list of all available debug probes.
    let probes = Probe::list_all();

    // Use the first probe found.
    let mut probe = probes[0].open()?;
    
    println!("Opened probe {}", probe.get_name());

    probe.attach_to_unspecified()?;

    println!("Attached");

    let mut iface = probe.try_into_arm_interface().unwrap();

    // Do a "recover" operation (erase+unlock a locked chip) on an nRF5340 target on both the app and net core.

    let app_port = 2;
    let net_port = 3;

    println!("Starting recovery");
    recover_cores(&mut iface, &[app_port, net_port])?;
    println!("Done");

    Ok(())
}

fn recover_cores(iface: &mut Box<dyn ArmProbeInterface>, ports: &[u8]) -> Result<()> {
    const RESET: u8 = 0x00;
    const ERASEALL: u8 = 0x04;
    const ERASEALLSTATUS: u8 = 0x08;
    const APPROTECTDISABLE: u8 = 0x10;
    const SECUREAPPROTECTDISABLE: u8 = 0x14;

    println!("  Reset");
    for port in ports {
        println!("    {}", port);
        iface.write_raw_ap_register(*port, RESET, 1)?;
        iface.write_raw_ap_register(*port, RESET, 0)?;
    }

    println!("  Erase");
    for port in ports {
        println!("    {}", port);
        iface.write_raw_ap_register(*port, ERASEALL, 1)?;
        // Wait for erase done
        while iface.read_raw_ap_register(*port, ERASEALLSTATUS)? != 0 {}
    }

    // println!("  Reset again");
    // for port in ports {
    //     println!("    {}", port);
    //     iface.write_raw_ap_register(*port, RESET, 1)?;
    //     iface.write_raw_ap_register(*port, RESET, 0)?;
    // }

    println!("  Checks");
    for port in ports {
        println!("    {}", port);
        println!("      {:0X}", iface.read_raw_ap_register(*port, APPROTECTDISABLE)?);
        println!("      {:0X}", iface.read_raw_ap_register(*port, SECUREAPPROTECTDISABLE)?);
    }

    Ok(())
}
