use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound;
use std::sync::{Arc, Mutex}; // Import Arc and Mutex

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let devices = host.input_devices()?;

    println!("Available input devices:");
    let devices_vec: Vec<_> = devices.collect();
    for (device_index, device) in devices_vec.iter().enumerate() {
        let device_name = device.name()?;
        println!("  {}: {}", device_index, device_name);
    }

    let selected_device_index = 0; // Change this if needed
    let selected_device = devices_vec
        .get(selected_device_index)
        .ok_or("Invalid device index")?;

    let mut supported_configs_range = selected_device.supported_input_configs()?;
    let config = supported_configs_range
        .next()
        .expect("No supported config?!")
        .with_sample_rate(cpal::SampleRate(48000))
        .config();

    // Use Arc<Mutex<Vec<i16>>> to share the captured data.
    let captured_data = Arc::new(Mutex::new(Vec::new()));
    let captured_data_clone = captured_data.clone(); // Clone the Arc for the closure.

    let stream = selected_device.build_input_stream(
        &config,
        move |data: &[i16], _: &cpal::InputCallbackInfo| {
            // Lock the Mutex to get exclusive access to the Vec.
            let mut captured_data = captured_data_clone.lock().unwrap();
            captured_data.extend_from_slice(data);
        },
        move |err| {
            eprintln!("An error occurred on the stream: {}", err);
        },
        None,
    )?;

    stream.play()?;

    println!("Capturing audio for 5 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(5));

    stream.pause()?;

    // Lock the Mutex one last time to access the data after the stream is done.
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

    for sample in captured_data.iter() { // Iterate over the captured data.
        writer.write_sample(*sample)?; // Dereference the sample.
    }

    writer.finalize()?;
    println!("Audio saved to recorded_audio.wav");

    // --- WAV Reading (using hound) ---
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