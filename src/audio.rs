use bevy::prelude::*;

use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::hacker32::*;

use crate::components::*;

pub fn ext_thread(mut commands: Commands) {
    // create slot for output
    let slot = Slot32::new(Box::new(dc(0.) | dc(0.)));
    // save its frontend in a bevy resource
    commands.insert_resource(Slot(slot.0));
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("Failed to find a default output device");
        let config = device.default_output_config().unwrap();
        match config.sample_format() {
            // passing the slot's backend inside
            cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), slot.1),
            cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), slot.1),
            cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), slot.1),
            _ => panic!("Unsupported format"),
        }
    });
}

fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut slot: SlotBackend32,
) where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f64;
    let channels = config.channels as usize;

    slot.set_sample_rate(sample_rate);
    let mut slot = BlockRateAdapter32::new(Box::new(slot));

    let mut next_value = move || {
        assert_no_alloc(|| {
            let (l, r) = slot.get_stereo();
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
        let right = T::from_sample(sample.1);

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}
