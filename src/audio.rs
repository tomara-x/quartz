use bevy::prelude::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::hacker32::*;

use crate::components::*;

pub fn default_out_device(mut commands: Commands) {
    let slot = Slot32::new(Box::new(dc(0.) | dc(0.)));
    commands.insert_resource(Slot(slot.0));
    let host = cpal::default_host();
    if let Some(device) = host.default_output_device() {
        let config = device.default_output_config().unwrap();
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), slot.1),
            cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), slot.1),
            cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), slot.1),
            _ => panic!("unsupported format"),
        };
        if let Some(stream) = stream {
            commands.insert_resource(OutStream(stream.into_inner()));
        }
    }
}

pub fn set_out_device(
    mut commands: Commands,
    mut out_device_event: EventReader<OutDeviceCommand>,
) {
    for e in out_device_event.read() {
        let slot = Slot32::new(Box::new(dc(0.) | dc(0.)));
        commands.insert_resource(Slot(slot.0));
        let (h, d) = e.0;
        if let Some(host_id) = cpal::platform::ALL_HOSTS.get(h) {
            if let Ok(host) = cpal::platform::host_from_id(*host_id) {
                if let Ok(mut devices) = host.output_devices() {
                    if let Some(device) = devices.nth(d) {
                        let config = device.default_output_config().unwrap();
                        let stream = match config.sample_format() {
                            cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), slot.1),
                            cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), slot.1),
                            cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), slot.1),
                            _ => panic!("unsupported format"),
                        };
                        if let Some(stream) = stream {
                            commands.insert_resource(OutStream(stream.into_inner()));
                        }
                    }
                }
            }
        }
    }
}

fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut slot: SlotBackend32,
) -> Option<cpal::Stream> where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f64;
    let channels = config.channels as usize;

    slot.set_sample_rate(sample_rate);
    let mut slot = BlockRateAdapter32::new(Box::new(slot));

    let mut next_value = move || {
        let (l, r) = slot.get_stereo();
        (
            if l.is_normal() { l.clamp(-1., 1.) } else { 0. },
            if r.is_normal() { r.clamp(-1., 1.) } else { 0. },
        )
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
            return Some(stream);
        }
    }
    None
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
