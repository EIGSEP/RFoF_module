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

    # Future parameters can be added here, e.g.,
    # parser.add_argument("--freq", type=float, help="Frequency in MHz")

    args = parser.parse_args()

    # Initialize the transmitter
    try:
        tx = FtxPi(args.device)
    except Exception as e:
        print(f"Failed to initialize transmitter: {e}")
        sys.exit(1)

    # Set attenuation
    try:
        tx.set_attenuation(args.atten)
        print(f"Successfully set attenuation to {args.atten} dB")
    except Exception as e:
        print(f"Error setting attenuation: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()

