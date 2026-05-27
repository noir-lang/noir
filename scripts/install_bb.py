import os
import sys
import subprocess
import shutil
import platform
import urllib.request
from pathlib import Path

VERSION = "5.0.0-nightly.20260522"

def main():
    home = Path.home()
    bb_dir = home / ".bb"
    bb_dir.mkdir(exist_ok=True)
    
    bbup_path = bb_dir / ("bbup.exe" if platform.system() == "Windows" else "bbup")
    
    if not bbup_path.exists():
        print("bbup not found, installing...")
        if platform.system() == "Windows":
            # For Windows, we might need a different approach or download a pre-built bbup
            # Aztec's bbup installation usually involves bash. 
            # As a fallback, we can try to download bb directly or use a known install method.
            print("Windows installation of bbup via Python is not fully implemented.")
            print("Please follow Aztec's official Windows installation guide for Barretenberg.")
            # However, let's try to download it if we find a URL.
            # For now, let's assume we need bbup to manage versions.
            pass
        else:
            install_url = "https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/next/barretenberg/bbup/install"
            try:
                with urllib.request.urlopen(install_url) as response:
                    install_script = response.read().decode('utf-8')
                
                # Execute the bash script
                subprocess.run(["bash"], input=install_script, text=True, check=True)
            except Exception as e:
                print(f"Failed to install bbup: {e}")
                sys.exit(1)

    if bbup_path.exists():
        subprocess.run([str(bbup_path), "-v", VERSION], check=True)
    else:
        print(f"Error: {bbup_path} still not found after installation attempt.")
        sys.exit(1)

if __name__ == "__main__":
    main()
