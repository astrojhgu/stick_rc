use paho_mqtt::{Client, ConnectOptionsBuilder, CreateOptionsBuilder, Message};

use sdl2::{event::Event, init, joystick::HatState};

use std::time::Duration;

pub fn camera_gcode(angle: f64) -> String {
    let p_value = 1.5 - ((angle - 90.0) / 90.0);
    let p_value = p_value.max(0.5).min(2.5);
    println!("{}", p_value);
    format!("G2 S8 P{}", p_value)
}

pub fn previous_hat_state(h: HatState) -> Option<HatState> {
    use HatState::*;
    match h {
        Down => Some(RightDown),
        RightDown => Some(Right),
        Right => Some(RightUp),
        RightUp => Some(Up),
        Up => Some(LeftUp),
        LeftUp => Some(Left),
        Left => Some(LeftDown),
        LeftDown => Some(Down),
        _ => None,
    }
}

pub fn next_hat_state(h: HatState) -> Option<HatState> {
    use HatState::*;
    match h {
        Down => Some(LeftDown),
        LeftDown => Some(Left),
        Left => Some(LeftUp),
        LeftUp => Some(Up),
        Up => Some(RightUp),
        RightUp => Some(Right),
        Right => Some(RightDown),
        RightDown => Some(Down),
        _ => None,
    }
}

fn main() -> Result<(), String> {
    let copt = CreateOptionsBuilder::new()
        .client_id("agv")
        .mqtt_version(0)
        .server_uri("tcp://192.168.1.249:1883")
        .finalize();
    let conn_option = ConnectOptionsBuilder::new()
        .user_name("any")
        .password("mqtt31415926")
        .server_uris(&["tcp://192.168.1.249:1883"])
        .finalize();
    let client = Client::new(copt).unwrap();
    client.connect(conn_option).unwrap();
    client.subscribe("agv/feedback/#", 1).unwrap();
    //let msg = Message::new("agv/gcode", "G0 L40 R40 S6000", 0);
    //client.publish(msg).unwrap();

    let sdl_context = init().unwrap();
    let joystick_submod = sdl_context.joystick().unwrap();
    let joystick = joystick_submod.open(0).unwrap();

    /*
    loop{
        joystick_submod.update();
        println!("{}", joystick.axis(3).unwrap());
        if(joystick.button(1).unwrap()){
            while joystick.button(1).unwrap(){
                std::thread::sleep(Duration::from_millis(10));
            }

        }
        println!("{}", joystick.button(4).unwrap());
        std::thread::sleep(Duration::from_millis(100));
    }*/

    let mut event_pump = sdl_context.event_pump()?;

    let get_speed = || {
        joystick_submod.update();
        let speed = -(joystick.axis(3).unwrap() as f64 / 32768.0 * 100.0);
        let diff_speed = {
            let x = joystick.axis(0).unwrap() as f64 / 32768.0 * 30.0;
            if x.abs() > 5.0 {
                x
            } else {
                0.0
            }
        };

        let speed_l = (speed + diff_speed * speed.signum()) as i32;
        let speed_r = (speed - diff_speed * speed.signum()) as i32;
        (speed_l, speed_r)
    };

    let mut old_hat_state = HatState::Centered;
    let mut camera_angle: f64 = 90.0;

    loop {
        let (speed_l, speed_r) = get_speed();
        if let Some(event) = event_pump.wait_event_timeout(100) {
            match event {
                Event::JoyAxisMotion {
                    axis_idx,
                    value: val,
                    ..
                } => {
                    // Axis motion is an absolute value in the range
                    // [-32768, 32767]. Let's simulate a very rough dead
                    // zone to ignore spurious events.
                    //
                    if axis_idx == 0 || axis_idx == 3 {
                        let (speed_l, speed_r) = get_speed();
                        println!("{} {}", speed_l, speed_r);
                    } /* else if axis_idx == 2 {
                          let camera_dir = ((val as f64 / 32768.0) * 90.0 + 90.0) as i32;
                          if old_camera_dir != camera_dir {
                              let p_value = 1.5 - ((camera_dir - 90) as f64 / 90.0);
                              let payload = format!("G2 S8 P{}", p_value);
                              let msg = Message::new("agv/gcode", payload.as_str(), 0);
                              client.publish(msg).unwrap();
                              old_camera_dir = camera_dir;
                          }
                      }*/
                }
                Event::JoyButtonDown { button_idx, .. } if button_idx == 1 => {
                    let (speed_l, speed_r) = get_speed();
                    let payload = format!("G0 L{} R{} S20000", speed_l, speed_r);
                    println!("{}", payload);
                    let msg = Message::new("agv/gcode", payload.as_str(), 0);
                    client.publish(msg).unwrap()
                }
                Event::JoyButtonUp { button_idx, .. } if button_idx == 1 => {
                    let msg = Message::new("agv/gcode", "Q", 1);
                    client.publish(msg).unwrap()
                }
                Event::JoyButtonDown { button_idx, .. } if button_idx==0 => {
                    camera_angle = 90.0;
                    let payload = camera_gcode(camera_angle);
                    let msg = Message::new("agv/gcode", payload.as_str(), 0);
                    client.publish(msg).unwrap();
                }
                Event::JoyButtonDown { button_idx, .. }  if button_idx==2||button_idx==3=> {
                    camera_angle = if button_idx==2 {0.0} else{180.0};
                    let payload = camera_gcode(camera_angle);
                    let msg = Message::new("agv/gcode", payload.as_str(), 0);
                    client.publish(msg).unwrap();
                }
                Event::JoyHatMotion { hat_idx, state, .. } => {
                    if old_hat_state != HatState::Centered {
                        if state == next_hat_state(old_hat_state).unwrap() {
                            camera_angle += 2.5;
                        } else if state == previous_hat_state(old_hat_state).unwrap() {
                            camera_angle -= 2.5;
                        }
                        camera_angle = camera_angle.min(180.0).max(0.0);
                        println!("{}", camera_angle);
                        let payload = camera_gcode(camera_angle);
                        println!("{}", payload);
                        let msg = Message::new("agv/gcode", payload.as_str(), 0);
                        client.publish(msg).unwrap();
                    }
                    old_hat_state = state;
                }
                Event::Quit { .. } => break,
                _ => (),
            }
        } else {
            if joystick.button(1).unwrap() {
                let (speed_l, speed_r) = get_speed();
                let payload = format!("G0 L{} R{} S2000", speed_l, speed_r);
                println!("{}", payload);
                let msg = Message::new("agv/gcode", payload.as_str(), 0);
                client.publish(msg).unwrap()
            }
        }
    }

    Ok(())
}
