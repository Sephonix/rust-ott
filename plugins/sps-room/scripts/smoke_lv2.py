#!/usr/bin/env python3
"""Dedicated SPS Room runtime smoke test using its identities and schema."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "scripts"))
import smoke_time_lv2 as smoke
smoke.PLUGIN = "sps-room"
smoke.CONTROL_COUNT = 24
smoke.PORT_COUNT = 30
smoke.URI = b"https://sephonix.com/plugins/sps-room"
if __name__ == "__main__":
    smoke.main()
