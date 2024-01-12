use bevy::prelude::*;

use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::hacker32::*;

use crate::components::*;

pub fn ext_thread(mut commands: Commands) {
    // create slots for outputs
    let slot_l = Slot32::new(Box::new(dc(0.)));
    let slot_r = Slot32::new(Box::new(dc(0.)));
    // save thier frontends in a bevy resource
    commands.insert_resource(Slot(slot_l.0, slot_r.0));
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("Failed to find a default output device");
        let config = device.default_output_config().unwrap();
        match config.sample_format() {
            // passing the slot backends inside
            cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), slot_l.1, slot_r.1),
            cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), slot_l.1, slot_r.1),
            cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), slot_l.1, slot_r.1),
            _ => panic!("Unsupported format"),
        }
    });
}

fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut slot_l: SlotBackend32,
    mut slot_r: SlotBackend32,
) where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f64;
    let channels = config.channels as usize;

    slot_l.set_sample_rate(sample_rate);
    slot_r.set_sample_rate(sample_rate);
    let mut next_value = move || {
        assert_no_alloc(|| {
            let l = slot_l.get_mono();
            let r = slot_r.get_mono();
            (
                if l.is_normal() { l.clamp(-1., 1.) } else { 0. },
                if r.is_normal() { r.clamp(-1., 1.) } else { 0. },
            )
        })
    };
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    );
    if let Ok(stream) = stream {
        if let Ok(()) = stream.play() {
            std::thread::sleep(std::time::Duration::from_secs(u64::MAX));
        }
    }
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f32, f32))
where
    T: SizedSample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left = T::from_sample(sample.0);
        let right: T = T::from_sample(sample.1);

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}
