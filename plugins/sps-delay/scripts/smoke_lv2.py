#!/usr/bin/env python3
"""Dedicated SPS Delay runtime smoke test using its identities and schema."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "scripts"))
import smoke_time_lv2 as smoke
smoke.PLUGIN = "sps-delay"
smoke.CONTROL_COUNT = 38
smoke.PORT_COUNT = 45
smoke.URI = b"https://sephonix.com/plugins/sps-delay"
if __name__ == "__main__":
    smoke.main()
