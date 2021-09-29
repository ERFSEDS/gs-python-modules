use postcard::take_from_bytes;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::{iter::IterNextOutput, prelude::*, PyIterProtocol};
use serde::{Deserialize, Serialize};
use std::io::Read;

create_exception!(blursed_serde, TelemetryDecodeException, PyException);

#[pymodule]
fn blursed_serde(py: Python, m: &PyModule) -> PyResult<()> {
    m.add(
        "TelemetryDecodeException",
        py.get_type::<TelemetryDecodeException>(),
    )?;
    m.add_class::<TelemetryFile>()?;
    m.add_class::<ImuData>()?;

    Ok(())
}

#[pyclass]
#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub struct ImuData {
    #[pyo3(get, set)]
    pub raw_x: f64,
    #[pyo3(get, set)]
    pub raw_y: f64,
    #[pyo3(get, set)]
    pub raw_z: f64,
    #[pyo3(get, set)]
    pub raw_pitch: f64,
    #[pyo3(get, set)]
    pub raw_yaw: f64,
    #[pyo3(get, set)]
    pub raw_roll: f64,
    #[pyo3(get, set)]
    pub x: f64,
    #[pyo3(get, set)]
    pub y: f64,
    #[pyo3(get, set)]
    pub z: f64,
    #[pyo3(get, set)]
    pub pitch: f64,
    #[pyo3(get, set)]
    pub yaw: f64,
    #[pyo3(get, set)]
    pub roll: f64,
}

#[pymethods]
impl ImuData {
    #[new]
    pub fn new() -> Self {
        ImuData {
            raw_x: 0.0,
            raw_y: 0.0,
            raw_z: 0.0,
            raw_pitch: 0.0,
            raw_yaw: 0.0,
            raw_roll: 0.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
        }
    }
}

#[pyclass]
#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub struct AltitudeData {
    #[pyo3(get, set)]
    pub raw_pressure: i64,
    #[pyo3(get, set)]
    pub altitude: i64,
}

#[pymethods]
impl AltitudeData {
    #[new]
    pub fn new() -> Self {
        AltitudeData {
            raw_pressure: 0,
            altitude: 0,
        }
    }
}

#[pyclass]
#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub struct TemperatureData {
    #[pyo3(get, set)]
    pub temp_f: f64,
    #[pyo3(get, set)]
    pub temp_c: f64,
}

#[pymethods]
impl TemperatureData {
    #[new]
    pub fn new() -> Self {
        TemperatureData {
            temp_f: 0.0,
            temp_c: 0.0,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub struct SDDataFrame {
    pub imu_data: ImuData,
    pub alt_data: AltitudeData,
    pub temp_data: TemperatureData,
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub struct SDDataHeader {
    pub data_count: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SDDataFile {
    pub data_header: SDDataHeader,
    pub data_frames: Vec<SDDataFrame>,
}

impl SDDataFile {
    pub fn new() -> Self {
        let data_header = SDDataHeader { data_count: 0 };

        let data_frames = Vec::new();

        Self {
            data_header,
            data_frames,
        }
    }

    pub fn add(&mut self, frame: SDDataFrame) {
        self.data_frames.push(frame);
        self.data_header.data_count += 1;
    }

    pub fn count(&self) -> u32 {
        self.data_header.data_count
    }
}

#[pyclass]
#[derive(Debug, Clone)]
struct TelemetryFile {
    bytes: Vec<u8>,
    header: SDDataHeader,
}

#[pymethods]
impl TelemetryFile {
    #[new]
    pub fn new(path: &str) -> PyResult<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut bytes = Vec::new();

        file.read_to_end(&mut bytes)?;

        let (header, rest) = take_from_bytes::<SDDataHeader>(&bytes).map_err(|_| {
            TelemetryDecodeException::new_err("Could not decode telemetry data header")
        })?;

        if header.data_count < u8::MAX as u32 {
            bytes = rest[1..].to_vec();
        } else if header.data_count < u16::MAX as u32 {
            bytes = rest[2..].to_vec();
        } else {
            bytes = rest[4..].to_vec();
        }

        Ok(Self { bytes, header })
    }
}

#[pyproto]
impl PyIterProtocol for TelemetryFile {
    fn __iter__(slf: PyRefMut<Self>) -> Self {
        slf.clone()
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<IterNextOutput<String, &'static str>> {
        if slf.header.data_count > 0 {
            if let Ok((sd_dataframe, rest)) = take_from_bytes::<SDDataFrame>(&slf.bytes) {
                let json_string = serde_json::to_string_pretty(&sd_dataframe)
                    .expect("Could not serialize struct into json");

                slf.bytes = rest.to_vec();
                slf.header.data_count -= 1;

                Ok(IterNextOutput::Yield(json_string))
            } else {
                Err(TelemetryDecodeException::new_err(
                    "Could not decode telemetry data frame",
                ))
            }
        } else {
            Ok(IterNextOutput::Return("Ended"))
        }
    }
}
