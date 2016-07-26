extern crate sdl2;
use std::fs::File;
use sdl2::rect::*;
use sdl2::keyboard::*;
use sdl2::event::*;
use sdl2::audio::*;
use sdl2::EventPump;
extern crate chip_8_core;
use chip_8_core::*;

fn hex_to_scan_code(hex: u8) -> Result<Scancode, &'static str> {
    match hex {
        0 => Ok(Scancode::X),
        1 => Ok(Scancode::Num1),
        2 => Ok(Scancode::Num2),
        3 => Ok(Scancode::Num3),
        4 => Ok(Scancode::Q),
        5 => Ok(Scancode::W),
        6 => Ok(Scancode::E),
        7 => Ok(Scancode::A),
        8 => Ok(Scancode::S),
        9 => Ok(Scancode::D),
        0xA => Ok(Scancode::Z),
        0xB => Ok(Scancode::C),
        0xC => Ok(Scancode::Num4),
        0xD => Ok(Scancode::R),
        0xE => Ok(Scancode::F),
        0xF => Ok(Scancode::V),
        _ => Err("Key tested for does not exist"),
    }
}

fn scan_code_to_hex(scancode: Scancode) -> Option<u8> {
    match scancode {
        Scancode::X => Some(0),
        Scancode::Num1 => Some(1),
        Scancode::Num2 => Some(2),
        Scancode::Num3 => Some(3),
        Scancode::Q => Some(4),
        Scancode::W => Some(5),
        Scancode::E => Some(6),
        Scancode::A => Some(7),
        Scancode::S => Some(8),
        Scancode::D => Some(9),
        Scancode::Z => Some(0xA),
        Scancode::C => Some(0xB),
        Scancode::Num4 => Some(0xC),
        Scancode::R => Some(0xD),
        Scancode::F => Some(0xE),
        Scancode::V => Some(0xF),
        _ => None,
    }
}

struct SdlKeyWrapper(EventPump);

impl KeyWrapper for SdlKeyWrapper {
    fn is_pushed(&self, key: u8) -> Result<bool, &'static str> {
        let scancode = try!(hex_to_scan_code(key));
        Ok(self.0.keyboard_state().is_scancode_pressed(scancode))
    }
    fn get_key(&self) -> Option<u8> {
        for key in self.0.keyboard_state().pressed_scancodes() {
            if let Some(hex) = scan_code_to_hex(key) {
                return Some(hex);
            }
        }
        None
    }
}

struct SimpleAudioDevice {
    current_feq: f32,
    feq_inc: f32,
    feq_target: f32,
    volume: f32,
}

impl AudioCallback for SimpleAudioDevice {
    type Channel = f32;
    
    fn callback(&mut self, out: &mut [f32]) {
        for x in out {
            if self.feq_inc.signum() == 1.0 {
                if (self.current_feq - self.feq_target) > 0.000005 {
                    self.feq_target = -self.feq_target;
                    self.feq_inc = -self.feq_inc;
                }
            } else if (self.current_feq - self.feq_target) < 0.000005 {
                self.feq_target = -self.feq_target;
                self.feq_inc = -self.feq_inc;
            }
            *x = self.volume / self.current_feq;
            self.current_feq += self.feq_inc;
        }
    }
}

struct SdlAudioWrapper<CB: AudioCallback>(AudioDevice<CB>);

impl<CB: AudioCallback> AudioWrapper for SdlAudioWrapper<CB> {
    fn play(&mut self) {
        self.0.resume();
    }
    
    fn stop(&mut self) {
        self.0.pause();
    }
}

fn main() {
    let mut args = std::env::args();
    args.next(); // We do not need the path of the executable.
    let sdl = sdl2::init().unwrap();
    let sdl_video = sdl.video().unwrap();
    let sdl_window = sdl_video.window("Chip-8", 64 * 8, 32 * 8)
        .resizable()
        .build()
        .unwrap();
    let sdl_event_pump = sdl.event_pump().unwrap();
    let sdl_key_wrapper = SdlKeyWrapper(sdl_event_pump);
    let sdl_audio = sdl.audio().unwrap();
    let spec = AudioSpecDesired {
        freq: None,
        channels: Some(1),
        samples: None
    };
    let mut sdl_renderer = sdl_window.renderer().present_vsync().build().unwrap();
    sdl_renderer.set_logical_size(64, 32).unwrap();
    sdl_renderer.present();
    let sdl_audio_device = sdl_audio.open_playback(None, &spec, |spec| { 
        SimpleAudioDevice {
            current_feq: 587.33,
            feq_inc: spec.freq as f32 / 587.33,
            feq_target: 587.33,
            volume: 1.00,
        }
    }).unwrap();
    let sdl_audio_wrapper = SdlAudioWrapper(sdl_audio_device);
    let mut chip8 = Chip8::new(sdl_key_wrapper, sdl_audio_wrapper);
    if let Some(file) = args.next() {
        match File::open(file) {
            Ok(mut input_file) => {
                chip8.load_prog_from_file(&mut input_file).unwrap();
            }
            Err(error) => {
                println!("{}", error);
                return;
            }
        }
    } else {
        println!("Please provide the program you want to run");
        return;
    }
    loop {
        if let Err(err) = chip8.run_vblank() {
            println!("{}", err);
            break;
        }
        sdl_renderer.clear();
        for event in chip8.key_wrapper.0.poll_iter() {
            if let Event::Quit { .. } = event {
                return;
            }
        }
        for i in chip8.frame_buffer.iter().enumerate() {
            let (y_cord, y) = i;
            for i in y.iter().enumerate() {
                let (x_cord, x) = i;
                if x & 1 != 0 {
                    sdl_renderer.draw_point(Point::new(x_cord as i32, y_cord as i32))
                        .unwrap();
                }
            }
        }
        sdl_renderer.present();
    }
}
