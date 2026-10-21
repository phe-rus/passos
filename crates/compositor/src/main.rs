use std::{error::Error, thread, time::Duration};

use smithay::{
    backend::{
        renderer::{Color32F, Frame, Renderer, gles::GlesRenderer},
        winit::{self, WinitEvent},
    },
    reexports::winit::event_loop::pump_events::PumpStatus,
    utils::{Rectangle, Transform},
};

const BACKGROUND: Color32F = Color32F::new(0.055, 0.063, 0.078, 1.0);

fn main() -> Result<(), Box<dyn Error>> {
    println!("pass compositor");

    let (mut backend, mut event_loop) = winit::init::<GlesRenderer>()?;

    let size = backend.window_size();

    println!("display: {}x{}", size.w, size.h);

    let mut needs_redraw = true;

    loop {
        let status = event_loop.dispatch_new_events(|event| {
            match event {
                WinitEvent::Resized { size, .. } => {
                    println!("resize: {}x{}", size.w, size.h);
                    needs_redraw = true;
                }

                WinitEvent::Input(_) => {
                    // Input handling comes next.
                }

                _ => {}
            }
        });

        if matches!(status, PumpStatus::Exit(_)) {
            break;
        }

        if needs_redraw {
            let size = backend.window_size();

            if size.w > 0 && size.h > 0 {
                let damage = Rectangle::from_size(size);

                {
                    let (renderer, mut framebuffer) = backend.bind()?;

                    let mut frame =
                        renderer.render(&mut framebuffer, size, Transform::Flipped180)?;

                    frame.clear(BACKGROUND, &[damage])?;
                    let _ = frame.finish()?;
                }

                backend.submit(Some(&[damage]))?;
                needs_redraw = false;
            }
        }

        thread::sleep(Duration::from_millis(16));
    }

    println!("compositor stopped");
    Ok(())
}
