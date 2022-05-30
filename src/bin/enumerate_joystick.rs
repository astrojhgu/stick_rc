use sdl2::init;

fn main() -> Result<(), String> {
    let sdl_context = init().unwrap();
    let joystick_submod = sdl_context.joystick().unwrap();
    let mut joystick = joystick_submod.open(0).unwrap();

    println!("{}", joystick.name());
    
    for event in sdl_context.event_pump()?.wait_iter() {
        use sdl2::event::Event;

        match event {
            Event::JoyAxisMotion {
                axis_idx,
                value: val,
                ..
            } => {
                // Axis motion is an absolute value in the range
                // [-32768, 32767]. Let's simulate a very rough dead
                // zone to ignore spurious events.
                let dead_zone = 10_000;
                if val > dead_zone || val < -dead_zone {
                    println!("Axis {} moved to {}", axis_idx, val);
                }
            }
            Event::JoyButtonDown { button_idx, .. } => {
                println!("Button {} down", button_idx);
            }
            Event::JoyButtonUp { button_idx, .. } => {
                println!("Button {} up", button_idx);
            }
            Event::JoyHatMotion { hat_idx, state, .. } => {
                println!("Hat {} moved to {:?}", hat_idx, state)
            }
            Event::Quit { .. } => break,
            _ => (),
        }
    }
    Ok(())
}
