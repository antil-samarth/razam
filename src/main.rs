use cpal::SampleFormat;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let devices = host.input_devices()?;

    println!("Available input devices:");
    let devices_vec: Vec<_> = devices.collect();
    for (device_index, device) in devices_vec.iter().enumerate() {
        let device_name = device.name()?;
        println!("  {}: {}", device_index, device_name);
    }

    if devices_vec.is_empty() {
        eprintln!("No input devices available.");
        return Ok(());
    }
    let selected_device = if devices_vec.len() == 1 {
        devices_vec[0].clone()
    } else {
        println!("Select a device to capture audio from:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        // check for empty input
        if input.trim().is_empty() {
            eprintln!("No device selected.");
            return Ok(());
        }
        let device_index: usize = input.trim().parse()?;
        if device_index >= devices_vec.len() {
            eprintln!("Invalid device index.");
            return Ok(());
        }
        devices_vec[device_index].clone()
    };
    println!(
        "Continuing with default device '{}'",
        selected_device.name()?
    );

    let config = selected_device
        .supported_input_configs()?
        .find(|c| {
            c.sample_format() == SampleFormat::I16
                && c.min_sample_rate() <= cpal::SampleRate(48000)
                && c.max_sample_rate() >= cpal::SampleRate(11025)
        })
        .ok_or("No supported I16 configuration at 48000 Hz")?
        .with_sample_rate(cpal::SampleRate(48000))
        .config();

    let captured_data = Arc::new(Mutex::new(Vec::new()));
    let captured_data_clone = captured_data.clone();

    let stream = selected_device.build_input_stream(
        &config,
        move |data: &[i16], _: &cpal::InputCallbackInfo| {
            let mut captured_data = captured_data_clone.lock().unwrap();
            captured_data.extend_from_slice(data);
        },
        move |err| {
            eprintln!("An error occurred on the stream: {}", err);
        },
        None,
    )?;

    println!("\nPress Enter to start capturing audio...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    stream.play()?;

    println!("Capturing audio for 5 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(5));

    stream.pause()?;

    let captured_data = captured_data.lock().unwrap();
    println!("Captured {} samples.", captured_data.len());

    // --- WAV Saving (using hound) ---
    let spec = hound::WavSpec {
        channels: config.channels,
        sample_rate: config.sample_rate.0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create("recorded_audio.wav", spec)?;

    for sample in captured_data.iter() {
        writer.write_sample(*sample)?;
    }

    writer.finalize()?;
    println!("Audio saved to recorded_audio.wav");

    println!("\nReading audio from recorded_audio.wav...");
    let mut reader = hound::WavReader::open("recorded_audio.wav")?;

    let read_spec = reader.spec();
    println!("  Channels: {}", read_spec.channels);
    println!("  Sample rate: {}", read_spec.sample_rate);
    println!("  Bits per sample: {}", read_spec.bits_per_sample);
    println!("  Sample format: {:?}", read_spec.sample_format);

    let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
    println!("Read {} samples.", samples.len());

    /* println!("\nSupported configurations for device '{}':", selected_device.name()?);
    let supported_configs_range = selected_device.supported_input_configs()?;
    for (config_index, config) in supported_configs_range.enumerate() {
        println!("  Config {}:", config_index);
        println!("      Sample rate: {:?}", config.min_sample_rate());
        println!("      Channels: {}", config.channels());
        println!("      Sample formate: {:?}", config.sample_format());
        println!("      Buffer size: {:?}", config.buffer_size());

    } */

    Ok(())
}