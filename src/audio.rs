use bevy::prelude::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::hacker32::*;

use crossbeam_channel::{bounded, Sender};

use crate::components::*;

pub fn default_out_device(world: &mut World) {
    let slot = Slot::new(Box::new(dc(0.) | dc(0.)));
    world.insert_resource(SlotRes(slot.0));
    let host = cpal::default_host();
    if let Some(device) = host.default_output_device() {
        let default_config = device.default_output_config().unwrap();
        let mut config = default_config.config();
        config.channels = 2;
        let stream = match default_config.sample_format() {
            cpal::SampleFormat::F32 => run::<f32>(&device, &config, slot.1),
            cpal::SampleFormat::I16 => run::<i16>(&device, &config, slot.1),
            cpal::SampleFormat::U16 => run::<u16>(&device, &config, slot.1),
            format => {
                error!("unsupported sample format: {}", format);
                None
            },
        };
        if let Some(stream) = stream {
            world.insert_non_send_resource(OutStream(stream));
        } else {
            error!("couldn't build stream");
        }
    }
}

pub fn set_out_device(world: &mut World) {
    let mut out_events = world.resource_mut::<Events<OutDeviceCommand>>();
    let events: Vec<OutDeviceCommand> = out_events.drain().collect();
    for e in events {
        let slot = Slot::new(Box::new(dc(0.) | dc(0.)));
        world.insert_resource(SlotRes(slot.0));
        let OutDeviceCommand(h, d, sr, b) = e;
        if let Some(host_id) = cpal::platform::ALL_HOSTS.get(h) {
            if let Ok(host) = cpal::platform::host_from_id(*host_id) {
                if let Ok(mut devices) = host.output_devices() {
                    if let Some(device) = devices.nth(d) {
                        let default_config = device.default_output_config().unwrap();
                        let mut config = default_config.config();
                        config.channels = 2;
                        if let Some(sr) = sr { config.sample_rate = cpal::SampleRate(sr); }
                        if let Some(b) = b { config.buffer_size = cpal::BufferSize::Fixed(b); }
                        let stream = match default_config.sample_format() {
                            cpal::SampleFormat::F32 => run::<f32>(&device, &config, slot.1),
                            cpal::SampleFormat::I16 => run::<i16>(&device, &config, slot.1),
                            cpal::SampleFormat::U16 => run::<u16>(&device, &config, slot.1),
                            format => {
                                error!("unsupported sample format: {}", format);
                                None
                            },
                        };
                        if let Some(stream) = stream {
                            world.insert_non_send_resource(OutStream(stream));
                        } else {
                            error!("couldn't build stream");
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
    slot: SlotBackend,
) -> Option<cpal::Stream> where
    T: SizedSample + FromSample<f32>,
{
    let mut slot = BlockRateAdapter::new(Box::new(slot));

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
            write_data(data, &mut next_value)
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

fn write_data<T>(output: &mut [T], next_sample: &mut dyn FnMut() -> (f32, f32))
where
    T: SizedSample + FromSample<f32>,
{
    for frame in output.chunks_mut(2) {
        let sample = next_sample();
        frame[0] = T::from_sample(sample.0);
        frame[1] = T::from_sample(sample.1);
    }
}



pub fn default_in_device(world: &mut World) {
    let (ls, lr) = bounded(64);
    let (rs, rr) = bounded(64);
    world.insert_resource(InputReceivers(lr, rr));
    let host = cpal::default_host();
    if let Some(device) = host.default_input_device() {
        let config = device.default_input_config().unwrap();
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => run_in::<f32>(&device, &config.into(), ls, rs),
            cpal::SampleFormat::I16 => run_in::<i16>(&device, &config.into(), ls, rs),
            cpal::SampleFormat::U16 => run_in::<u16>(&device, &config.into(), ls, rs),
            format => {
                error!("unsupported sample format: {}", format);
                None
            },
        };
        if let Some(stream) = stream {
            world.insert_non_send_resource(InStream(stream));
        } else {
            error!("couldn't build stream");
        }
    }
}

pub fn set_in_device(world: &mut World) {
    let mut out_events = world.resource_mut::<Events<InDeviceCommand>>();
    let events: Vec<InDeviceCommand> = out_events.drain().collect();
    for e in events {
        let (ls, lr) = bounded(64);
        let (rs, rr) = bounded(64);
        world.insert_resource(InputReceivers(lr, rr));
        let InDeviceCommand(h, d, sr, b) = e;
        if let Some(host_id) = cpal::platform::ALL_HOSTS.get(h) {
            if let Ok(host) = cpal::platform::host_from_id(*host_id) {
                if let Ok(mut devices) = host.input_devices() {
                    if let Some(device) = devices.nth(d) {
                        let default_config = device.default_input_config().unwrap();
                        let mut config = default_config.config();
                        if let Some(sr) = sr { config.sample_rate = cpal::SampleRate(sr); }
                        if let Some(b) = b { config.buffer_size = cpal::BufferSize::Fixed(b); }
                        let stream = match default_config.sample_format() {
                            cpal::SampleFormat::F32 => run_in::<f32>(&device, &config, ls, rs),
                            cpal::SampleFormat::I16 => run_in::<i16>(&device, &config, ls, rs),
                            cpal::SampleFormat::U16 => run_in::<u16>(&device, &config, ls, rs),
                            format => {
                                error!("unsupported sample format: {}", format);
                                None
                            },
                        };
                        if let Some(stream) = stream {
                            world.insert_non_send_resource(InStream(stream));
                        } else {
                            error!("couldn't build stream");
                        }
                    }
                }
            }
        }
    }
}

fn run_in<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    ls: Sender<f32>,
    rs: Sender<f32>,
) -> Option<cpal::Stream> where
    T: SizedSample, f32: FromSample<T>
{
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            read_data(data, channels, ls.clone(), rs.clone())
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

fn read_data<T>(input: &[T], channels: usize, ls: Sender<f32>, rs: Sender<f32>)
where
    T: SizedSample, f32: FromSample<T>
{
    for frame in input.chunks(channels) {
        for (channel, sample) in frame.iter().enumerate() {
            if channel & 1 == 0 {
                let _ = ls.send(sample.to_sample::<f32>());
            } else {
                let _ = rs.send(sample.to_sample::<f32>());
            }
        }
    }
}
