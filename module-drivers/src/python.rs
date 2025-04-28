//! Python interface for FT4232H boards

use crate::{
    modules::frx::Frx as InnerFrx, modules::ftx::Ftx as InnerFtx, peripherals::atten::Attenuation,
};
use embedded_hal::i2c::{ErrorType, I2c as I2cTrait};
use ftdi_embedded_hal::{
    self as hal,
    libftd2xx::{self, Ft4232ha, Ftdi},
    I2c,
};
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};
use std::sync::{Arc, Mutex};

type SharedFTDIDevice = Arc<Mutex<I2c<Ft4232ha>>>;

struct SharedDeivce(SharedFTDIDevice);

impl SharedDeivce {
    fn new(bus: &SharedFTDIDevice) -> Self {
        Self(Arc::clone(bus))
    }
}

impl ErrorType for SharedDeivce {
    type Error = <I2c<Ft4232ha> as embedded_hal::i2c::ErrorType>::Error;
}

impl I2cTrait for SharedDeivce {
    #[inline]
    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        let bus = &mut *self.0.lock().unwrap();
        bus.read(address, read)
    }

    #[inline]
    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        let bus = &mut *self.0.lock().unwrap();
        bus.write(address, write)
    }

    #[inline]
    fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        let bus = &mut *self.0.lock().unwrap();
        bus.write_read(address, write, read)
    }

    #[inline]
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        let bus = &mut *self.0.lock().unwrap();
        bus.transaction(address, operations)
    }
}

// ------ Adding support for native Rpi i2c using linux-embedded-hal -------
use linux_embedded_hal::{I2cdev};

type SharedLinuxI2C = Arc<Mutex<I2cdev>>;

struct SharedLinuxI2CDevice(SharedLinuxI2C);

impl SharedLinuxI2CDevice {
    fn new(bus: &SharedLinuxI2C) -> Self {
        Self(Arc::clone(bus))
    }
}

impl embedded_hal::i2c::ErrorType for SharedLinuxI2CDevice {
    type Error = <I2cdev as embedded_hal::i2c::ErrorType>::Error;
}

impl I2cTrait for SharedLinuxI2CDevice {
    fn read(&mut self, address: u8, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let mut guard = self.0.lock().unwrap();
        guard.read(address, bytes)
    }

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        let mut guard = self.0.lock().unwrap();
        guard.write(address, bytes)
    }

    fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        let mut guard = self.0.lock().unwrap();
        guard.write_read(address, write, read)
    }

    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        let mut guard = self.0.lock().unwrap();
        guard.transaction(address, operations)
    }
}
// ------------------------------------------

#[pyfunction]
/// List available FTDI devices
pub fn list_devices() -> PyResult<Vec<String>> {
    Ok(libftd2xx::list_devices()
        .map_err(|_| PyRuntimeError::new_err("Could not get devices from the FTDI driver"))?
        .iter()
        .map(|d| d.description.to_string())
        .collect())
}

#[pyclass]
struct Ftx(InnerFtx<SharedDeivce>);

#[pymethods]
impl Ftx {
    #[new]
    fn new(idx: i32) -> PyResult<Self> {
        // Open the FTDI device
        let device: Ft4232ha = Ftdi::with_index(idx)
            .map_err(|_| PyValueError::new_err("Could not find a device with that index"))?
            .try_into()
            .map_err(|_| PyRuntimeError::new_err("Device was not an FT4232HA"))?;

        // Extract the I2C interface
        let hal = hal::FtHal::init_freq(device, 100_000)
            .map_err(|_| PyRuntimeError::new_err("Could not start the interface"))?
            .i2c()
            .map_err(|_| PyRuntimeError::new_err("Could not instantiate I2C"))?;

        // Construct the bus (move to the heap behind a mutex with reference counting)
        let bus = Arc::new(Mutex::new(hal));

        // Construct the inner device
        let mut inner = InnerFtx::new(
            SharedDeivce::new(&bus),
            SharedDeivce::new(&bus),
            SharedDeivce::new(&bus),
            SharedDeivce::new(&bus),
        );

        // Initialize
        inner
            .init()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))?;

        Ok(Self(inner))
    }

    /// Get the board temperature in deg C
    pub fn get_temp(&mut self) -> PyResult<f32> {
        self.0
            .temp
            .temp()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the unique ID as an integer
    pub fn get_uid(&mut self) -> PyResult<u64> {
        self.0
            .temp
            .uid()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the current attenuator state in dB
    pub fn get_atten(&mut self) -> PyResult<f32> {
        let atten = self
            .0
            .atten
            .get()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))? as u8;
        let atten = atten as f32 * 0.25;
        Ok(atten)
    }

    /// Set the state of the digital step attenuator in dB
    pub fn set_atten(&mut self, atten: f32) -> PyResult<()> {
        if !(0.0..=31.75).contains(&atten) {
            Err(PyValueError::new_err("attenuation out of bounds"))
        } else {
            let word = (atten / 0.25) as u8;
            let atten = unsafe { core::mem::transmute::<u8, Attenuation>(word & 127) };
            self.0
                .atten
                .set(atten)
                .map_err(|_| PyRuntimeError::new_err("I2C Error"))?;
            Ok(())
        }
    }

    /// Get the RF current at the power detector in dBm
    pub fn get_rf_power(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .rf_power()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the DC monitor photodiode current in uA
    pub fn get_pd_current(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .pd_current()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the DC laser current in mA
    pub fn get_ld_current(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .ld_current()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the analog supply (VDDA) voltage in V
    pub fn get_vdda_voltage(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .analog_voltage()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the digital supply (VDD) voltage in V
    pub fn get_vdd_voltage(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .digital_voltage()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the LNA voltage in V
    pub fn get_lna_voltage(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .lna_voltage()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the LNA current in mA
    pub fn get_lna_current(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .lna_current()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Control the load switch for the LNA bias
    pub fn set_lna_enable(&mut self, enable: bool) -> PyResult<()> {
        self.0.adc.enable_lna(enable).unwrap();
        //.map_err(|_| PyRuntimeError::new_err("I2C Error"))?;
        Ok(())
    }

    /// Set the laser current in mA (0-50)
    pub fn set_ld_current(&mut self, current: f32) -> PyResult<()> {
        self.0
            .digipot
            .set(current)
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))?;
        Ok(())
    }
}



// ----------- Adding Support for Rpi, Ftx class --------
#[pyclass]
struct FtxPi(InnerFtx<SharedLinuxI2CDevice>);

#[pymethods]
impl FtxPi {
    // new constuctor for Rpi or other linux i2c.
    // ex. usage from python: 'FtxPi("/dev/i2c-1")'
    #[new]
    fn new(bus_path: &str) -> PyResult<Self> {
        // 1. opens the linux i2c bus (e.g. /dev/i2c-1)
        let i2cdev = I2cdev::new(bus_path)
            .map_err(|_| PyRuntimeError::new_err("Could not open I2C bus"))?;
        
        // 2. wrap it in Arc<mutex<...>>
        let bus = Arc::new(Mutex::new(i2cdev));

        // 3. Construct the existing 'InnerFtx' with 4 copies if needed 
        let mut inner = InnerFtx::new(
            SharedLinuxI2CDevice::new(&bus),
            SharedLinuxI2CDevice::new(&bus),
            SharedLinuxI2CDevice::new(&bus),
            SharedLinuxI2CDevice::new(&bus),
        );

        // 4) Initialize
        inner
            .init()
            .map_err(|_| PyRuntimeError::new_err("I2C Error initializing Ftx"))?;

        Ok(Self(inner))
    }

    /// The same exact methods as the Ftx class

    /// Get the board temperature in deg C
    pub fn get_temp(&mut self) -> PyResult<f32> {
        self.0
            .temp
            .temp()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the unique ID as an integer
    pub fn get_uid(&mut self) -> PyResult<u64> {
        self.0
            .temp
            .uid()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the current attenuator state in dB
    pub fn get_atten(&mut self) -> PyResult<f32> {
        let atten = self
            .0
            .atten
            .get()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))? as u8;
        let atten = atten as f32 * 0.25;
        Ok(atten)
    }

    /// Set the state of the digital step attenuator in dB
    pub fn set_atten(&mut self, atten: f32) -> PyResult<()> {
        if !(0.0..=31.75).contains(&atten) {
            Err(PyValueError::new_err("attenuation out of bounds"))
        } else {
            let word = (atten / 0.25) as u8;
            let atten = unsafe { core::mem::transmute::<u8, Attenuation>(word & 127) };
            self.0
                .atten
                .set(atten)
                .map_err(|_| PyRuntimeError::new_err("I2C Error"))?;
            Ok(())
        }
    }

    /// Get the RF current at the power detector in dBm
    pub fn get_rf_power(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .rf_power()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the DC monitor photodiode current in uA
    pub fn get_pd_current(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .pd_current()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the DC laser current in mA
    pub fn get_ld_current(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .ld_current()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the analog supply (VDDA) voltage in V
    pub fn get_vdda_voltage(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .analog_voltage()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the digital supply (VDD) voltage in V
    pub fn get_vdd_voltage(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .digital_voltage()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the LNA voltage in V
    pub fn get_lna_voltage(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .lna_voltage()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the LNA current in mA
    pub fn get_lna_current(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .lna_current()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Control the load switch for the LNA bias
    pub fn set_lna_enable(&mut self, enable: bool) -> PyResult<()> {
        self.0.adc.enable_lna(enable).unwrap();
        //.map_err(|_| PyRuntimeError::new_err("I2C Error"))?;
        Ok(())
    }

    /// Set the laser current in mA (0-50)
    pub fn set_ld_current(&mut self, current: f32) -> PyResult<()> {
        self.0
            .digipot
            .set(current)
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))?;
        Ok(())
    }
}

// -----------------------------------------------------

#[pyclass]
struct Frx(InnerFrx<SharedDeivce>);

#[pymethods]
impl Frx {
    #[new]
    fn new(idx: i32) -> PyResult<Self> {
        // Open the FTDI device
        let device: Ft4232ha = Ftdi::with_index(idx)
            .map_err(|_| PyValueError::new_err("Could not find a device with that index"))?
            .try_into()
            .map_err(|_| PyRuntimeError::new_err("Device was not an FT4232HA"))?;

        // Extract the I2C interface
        let hal = hal::FtHal::init_freq(device, 10_000)
            .map_err(|_| PyRuntimeError::new_err("Could not start the interface"))?
            .i2c()
            .map_err(|_| PyRuntimeError::new_err("Could not instantiate I2C"))?;

        // Construct the bus (move to the heap behind a mutex with reference counting)
        let bus = Arc::new(Mutex::new(hal));

        // Construct the inner device
        let mut inner = InnerFrx::new(
            SharedDeivce::new(&bus),
            SharedDeivce::new(&bus),
            SharedDeivce::new(&bus),
        );

        // Initialize
        inner
            .init()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))?;

        Ok(Self(inner))
    }

    /// Get the board temperature in deg C
    pub fn get_temp(&mut self) -> PyResult<f32> {
        self.0
            .temp
            .temp()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the RF current at the power detector in dBm
    pub fn get_rf_power(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .rf_power()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the DC photodiode current in mA
    pub fn get_pd_current(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .pd_current()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the unique ID as an integer
    pub fn get_uid(&mut self) -> PyResult<u64> {
        self.0
            .temp
            .uid()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the current attenuator state in dB
    pub fn get_atten(&mut self) -> PyResult<f32> {
        let atten = self
            .0
            .atten
            .get()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))? as u8;
        let atten = atten as f32 * 0.25;
        Ok(atten)
    }

    /// Set the state of the digital step attenuator in dB
    pub fn set_atten(&mut self, atten: f32) -> PyResult<()> {
        if !(0.0..=31.75).contains(&atten) {
            Err(PyValueError::new_err("attenuation out of bounds"))
        } else {
            let word = (atten / 0.25) as u8;
            let atten = unsafe { core::mem::transmute::<u8, Attenuation>(word & 127) };
            self.0
                .atten
                .set(atten)
                .map_err(|_| PyRuntimeError::new_err("I2C Error"))?;
            Ok(())
        }
    }
}


// ---------------------- Adding Support for Rpi, FrxPi Class ------------
#[pyclass]
struct FrxPi(InnerFrx<SharedLinuxI2CDevice>);

#[pymethods]
impl FrxPi {
    #[new]
    fn new(bus_path: &str) -> PyResult<Self> {
        let i2cdev = I2cdev::new(bus_path)
            .map_err(|_| PyRuntimeError::new_err("Could not open I2C bus"))?;

        let bus = Arc::new(Mutex::new(i2cdev));
        let mut inner = InnerFrx::new(
            SharedLinuxI2CDevice::new(&bus),
            SharedLinuxI2CDevice::new(&bus),
            SharedLinuxI2CDevice::new(&bus),
        );

        inner
            .init()
            .map_err(|_| PyRuntimeError::new_err("I2C Error initializing Frx"))?;

        Ok(Self(inner))
    }


    /// Get the board temperature in deg C
    pub fn get_temp(&mut self) -> PyResult<f32> {
        self.0
            .temp
            .temp()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the RF current at the power detector in dBm
    pub fn get_rf_power(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .rf_power()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the DC photodiode current in mA
    pub fn get_pd_current(&mut self) -> PyResult<f32> {
        self.0
            .adc
            .pd_current()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the unique ID as an integer
    pub fn get_uid(&mut self) -> PyResult<u64> {
        self.0
            .temp
            .uid()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))
    }

    /// Get the current attenuator state in dB
    pub fn get_atten(&mut self) -> PyResult<f32> {
        let atten = self
            .0
            .atten
            .get()
            .map_err(|_| PyRuntimeError::new_err("I2C Error"))? as u8;
        let atten = atten as f32 * 0.25;
        Ok(atten)
    }

    /// Set the state of the digital step attenuator in dB
    pub fn set_atten(&mut self, atten: f32) -> PyResult<()> {
        if !(0.0..=31.75).contains(&atten) {
            Err(PyValueError::new_err("attenuation out of bounds"))
        } else {
            let word = (atten / 0.25) as u8;
            let atten = unsafe { core::mem::transmute::<u8, Attenuation>(word & 127) };
            self.0
                .atten
                .set(atten)
                .map_err(|_| PyRuntimeError::new_err("I2C Error"))?;
            Ok(())
        }
    }
}
// ------------------------------------------------

#[pymodule]
pub fn rfof(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(list_devices, module)?)?;
    module.add_class::<Ftx>()?;
    module.add_class::<Frx>()?;
    // added new Rpi-based classes 
    module.add_class::<FtxPi>()?;
    module.add_class::<FrxPi>()?;
    Ok(())
}
