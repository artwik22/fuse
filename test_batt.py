import subprocess
import os

try:
    print(subprocess.run(["upower", "-e"], capture_output=True, text=True).stdout)
except Exception as e:
    print(e)

try:
    print("Listing power supply:")
    print(" ".join(os.listdir("/sys/class/power_supply")))
except Exception as e:
    print(e)
