use pass_device::{ConnectionState, discover};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = discover()?;

    println!("pass device inspection");
    println!("displays: {}", snapshot.displays.len());

    for display in snapshot.displays {
        let state = match display.state {
            ConnectionState::Connected => "connected",
            ConnectionState::Disconnected => "disconnected",
            ConnectionState::Unknown => "unknown",
        };

        println!("  {}: {}", display.name, state);
    }

    Ok(())
}
