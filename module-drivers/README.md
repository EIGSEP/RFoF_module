# RFoF Module Drivers

Rust support (no-std) library for DSA2000 RFoF hardware (FTX/FRX modules)

## Python Wrapper

Included in this project is a python wrapper that exposes a high-level interface to the `Frx` and `Ftx` modules.
Note: This should _not_ be used as a production interface to the RFoF hardware, only for testing.

On Courtney Keeler's FT4232HA dual controller board, interface `A` is used for the FRX and interface `B` is used for the FTX.

### Installation

You can install the wrapper directly with pip via

```sh
pip install git+https://gitlab.com/dsa-2000/asp/rfof/module-drivers.git
```

but note this will build the rust project and will therefore need a rust compiler.

To try to use a prebuild wheel (which hopefully we can build for all reasonable platforms), use

```sh
pip install rfof --index-url https://gitlab.com/api/v4/projects/61004894/packages/pypi/simple
```

Note: The rust library does *not* use PyFTDI and contains its own implementation of the FTDI USB interface.
This is backed by the [D2XX](https://ftdichip.com/drivers/d2xx-drivers/) drivers and has the same requirements.
Notably, on linux, you need to unload the `ftdi_sio` driver with

```sh
sudo rmmod ftdi_sio
```

### FRX Example

```py
from rfof import Ftx, Frx

frx = Frx(0) # Interface A

# Monitor
frx.get_temp()       # C
frx.get_uid()        # int
frx.get_atten()      # dB
frx.get_rf_power()   # dBm
frx.get_pd_current() # mA

# Control
frx.set_atten(12.25) # dB
```

### FTX Example

```py
from rfof import Ftx, Frx

ftx = Ftx(1) # Interface B

# Monitor
ftx.get_temp()         # C
ftx.get_uid()          # int
ftx.get_atten()        # dB
ftx.get_rf_power()     # dBm
ftx.get_pd_current()   # uA
ftx.get_ld_current()   # mA
ftx.get_lna_current()  # mA
ftx.get_lna_voltage()  # V
ftx.get_vdd_voltage()  # V
ftx.get_vdda_voltage() # V

# Control
ftx.set_atten(12.25)
ftx.set_lna_enable(True)
ftx.set_ld_current(31.5)
```
