import argparse
import sys

try:
    from rfof import FtxPi
except ImportError:
    print("Error: Could not import transmitter control module.")
    sys.exit(1)

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
        required=True,
        help="Attenuation to set in dB."
    )

    parser.add_argument(
        "--read_temp",
        type=float,
        help="Reads temperature of Ftx board."
    )

    parser.add_argument(
        "--read_atten",
        type=float,
        help="Reads attenuation of Ftx board."

    )
    # Future parameters can be added here, e.g.,
    # parser.add_argument("--freq", type=float, help="Frequency in MHz") - though we'd use different params...
    # look in python.rs in module-drivers/src/ for additional methods within FtxPi class.

    args = parser.parse_args()

    # Initialize the transmitter
    try:
        tx = FtxPi(args.device)
    except Exception as e:
        print(f"Failed to initialize transmitter: {e}")
        sys.exit(1)

    # Set attenuation MAX ATTENUATION 31.5 dB
    try:
        tx.set_atten(args.atten)
        print(f"Successfully set attenuation to {args.atten} dB")
    except Exception as e:
        print(f"Error setting attenuation: {e}")
        sys.exit(1)

    # Grab temp in C˚
    try:
        tx.get_temp(args.read_temp)
        print(f"Transmitter Temp: {args.read_temp} C˚")
    except Exception as e:
        print(f"Failed to read temperature: {e}")
        sys.exit(1)

    # Readout attenuation of Ftx board (dB).
    try:
        tx.get_atten(args.read_atten)
    except Exception as e:
        print(f"Failed to read Ftx attenuation. {e}")
        sys.exit(1)
    

if __name__ == "__main__":
    main()

