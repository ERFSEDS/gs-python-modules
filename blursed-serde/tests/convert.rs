use blursed_serde::{AltitudeData, ImuData, SDDataFile, SDDataFrame, TemperatureData};
use heapless::Vec;
use postcard::{from_bytes, to_vec};
use std::io::{Read, Write};

#[test]
fn conversion() {
    write_binary();
    read_binary();
    convert_json();
}

fn write_binary() {
    let imu_data = ImuData {
        raw_x: 0.6,
        raw_y: 0.3,
        raw_z: 0.4,
        raw_pitch: 0.45,
        raw_yaw: 0.31,
        raw_roll: 0.04,
        x: 60.0,
        y: 30.0,
        z: 40.0,
        pitch: 45.0,
        yaw: 32.0,
        roll: 4.0,
    };

    let alt_data = AltitudeData {
        raw_pressure: 235325424,
        altitude: 2500,
    };

    let temp_data = TemperatureData {
        temp_f: 92.3,
        temp_c: 33.5,
    };

    let data_frame = SDDataFrame {
        imu_data,
        alt_data,
        temp_data,
    };

    let mut data = SDDataFile::new();
    data.add(data_frame);
    data.add(data_frame);
    data.add(data_frame);
    data.add(data_frame);
    data.add(data_frame);

    let header_buffer: Vec<u8, 512> = to_vec(&data.data_header).unwrap();

    let mut file_1 = std::fs::File::create("header.bin").expect("Could not create header.bin");

    file_1
        .write_all(header_buffer.as_slice())
        .expect("Could not write to header.bin");

    file_1.flush().expect("Could not flush header.bin write");

    let data_buffer: Vec<u8, 2048> = to_vec(&data.data_frames).unwrap();

    let mut file_2 = std::fs::File::create("frames.bin").expect(".");

    file_2.write_all(data_buffer.as_slice()).expect(".3");

    file_2.flush().expect(".2");

    let frame_buffer: Vec<u8, 512> = to_vec(&data.data_frames.get(0).unwrap()).unwrap();

    let mut file_3 = std::fs::File::create("frame.bin").expect(".4");

    file_3.write_all(frame_buffer.as_slice()).expect(".5");

    file_3.flush().expect(".6");

    let file_buffer: Vec<u8, 2048> = to_vec(&data).unwrap();

    let mut file = std::fs::File::create("sd_data.bin").expect("Could not create sd_data.bin");

    file.write_all(file_buffer.as_slice())
        .expect("Could not write to sd_data.bin");

    file.flush().expect("Could not flush sd_data.bin write");
}

fn read_binary() {
    let mut read_file = std::fs::File::open("sd_data.bin").expect("Could not open sd_data.bin");

    let mut read_buffer: std::vec::Vec<u8> = std::vec::Vec::new();

    read_file
        .read_to_end(&mut read_buffer)
        .expect("Could not read sd_data.bin");

    let read_sd_data: SDDataFile = from_bytes(&read_buffer).expect("Could not parse sd_data.bin");

    println!("{:#?}", read_sd_data);
}

fn convert_json() {
    let mut read_file = std::fs::File::open("sd_data.bin").expect("Could not open sd_data.bin");

    let mut read_buffer: std::vec::Vec<u8> = std::vec::Vec::new();

    read_file
        .read_to_end(&mut read_buffer)
        .expect("Could not read sd_data.bin");

    let read_sd_data: SDDataFile = from_bytes(&read_buffer).expect("Could not parse sd_data.bin");

    let json_string =
        serde_json::to_string_pretty(&read_sd_data).expect("Could not serialize struct into json");

    let mut json_file =
        std::fs::File::create("sd_data.json").expect("Could not create sd_data.json");

    json_file
        .write_all(json_string.as_bytes())
        .expect("Could not write to sd_data.json");
}
