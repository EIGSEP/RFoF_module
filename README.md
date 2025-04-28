## RFoF_module
.

## module-drivers setup for programming RFoF transmitter
## module-drivers was forked from: [module-drivers](https://gitlab.com/dsa-2000/asp/rfof/module-drivers)
## Changes to module-drivers include: linux-embedded-hal support, added Frx/Ftx classes for Rpi i2c.

1. **Create & activate a virtual environment (venv, conda, etc.)**
   ```bash
   python3 -m venv ~/rfof-env
   source rfof-env/bin/activate
   ```
2. Install Rust & activate its path (may need to add sudo if necessary)
   
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.bashrc
   # (reactivate venv)
   source ~/.cargo/env
   rustup --version
   cargo --version
   ```
3. Install python wrapper + maturin
   ```bash
   pip install git+https://gitlab.com/dsa-2000/asp/rfof/module-drivers.git
   pip install maturin
   git clone https://gitlab.com/dsa-2000/asp/rfof/module-drivers.git
   ```

4. Inside ~/module-drivers execute "maturin develop"
   ```bash
    maturin develop
   ```

5. Add linux-embedded-hal to Cargo.toml under "Python deps", example of the Cargo.toml
    ```toml
    # Python deps
    pyo3 = { version = "0.22", features = [
        "extension-module",
        "abi3-py38",
        "generate-import-lib",
    ], optional = true }
    thiserror = { version = "1", optional = true }
    ftdi-embedded-hal = { version = "0.22", features = [
        "libftd2xx-static",
    ], optional = true }
    embedded-hal-bus = { version = "0.2", optional = true }
    linux-embedded-hal = "0.4.0" # THIS ONE
    ```
6. Navigate to ~/module-drivers/src/ then add linux-embedded-hal and the following code blocks to python.rs
   #### From line 18 add:
   ```rust
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
   ```

7. Add FtxPi class to python.rs from line 281:
   
    ```rust

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
    ```

8. Add FrxPi class to python.rs from line 534:
    
    ```rust

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
    ```

9. From line 631 (in #[pymodule]) add:

    ```rust
        // added new Rpi-based classes 
        module.add_class::<FtxPi>()?;
        module.add_class::<FrxPi>()?;
    ```

10. When finished adding the above classes and modules save and exit  python.rs
   
11. navigate back to ~/module-drivers and execute "maturin develop", this time it will add
    25 packages including linux-embedded-hal (v0.4.0)

12. (may not need), run cargo build in ~/module-drivers
    ```bash
    cargo build
    ```

13. add user to sudo and i2c groups if neccessary

14. test connection to Ftx & Frx (ipython or python):

    ```python
    try:
        ftx_pi = rfof.FtxPi("/dev/i2c-1")  # can check i2c bus w/ "ls /dev/ | grep i2c"
        print("FtxPi object created successfully!")
    except Exception as e:
        print(f"Error creating FtxPi: {e}")
    ```

15. Try to read temperature:
    ```python
    ftx_pi.get_temp()
    ```


