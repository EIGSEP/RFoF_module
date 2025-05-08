import argparse
import sys
from rfof import FtxPi

"""
CLI tool for controlling EIGSEP Ftx module.
example usage: python3 transmitter.py --atten 15.0 --read_temp
"""
DEFAULT_I2C_DEV = "/dev/i2c-1"

def main():
    parser = argparse.ArgumentParser(description="EIGSEP transmitter Control Script")

    parser.add_argument(
        "--device",
        type=str,
        default=DEFAULT_I2C_DEV,
        help="Path to I2C device (default: /dev/i2c-1)"
    )
    
    parser.add_argument(
        "--atten",
        type=float,
        help="Set attenuation in dB (0 - 31.5)"
    )

    parser.add_argument(
        "--read_temp",
        action="store_true",
        help="Reads temperature of Ftx board."
    )

    parser.add_argument(
        "--read_atten",
        action="store_true",
        help="Reads attenuation of Ftx board."

    )
    # Future parameters can be added here, e.g.,
    # parser.add_argument("--freq", type=float, help="Frequency in MHz") - though we'd use different params...
    # look in python.rs in module-drivers/src/ path for additional methods within FtxPi class.

    args = parser.parse_args()

    tx = FtxPi(args.device) # init

    # Set attenuation MAX ATTENUATION 31.5 dB
    if args.atten is not None:
        tx.set_atten(args.atten)
        print(f"Set attenuation to {args.atten} dB")


    # Grab temp in C˚
    if args.read_temp:
        temp = tx.get_temp()
        print(f"Transmitter Temp: {temp:.2f} C˚")


    # Readout attenuation of Ftx board (dB).
    if args.read_atten:
        atten = tx.get_atten()
        print(f"Attenuation: {atten} dB")

    

if __name__ == "__main__":
    main()

