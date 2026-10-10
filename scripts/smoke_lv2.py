#!/usr/bin/env python3
"""Load the shipped LV2 ABI and metadata; optionally exercise its native UI.

No audio server or DAW is required. --ui requires X11/XWayland and OpenGL.
"""
import argparse
import ctypes as C
import ctypes.util
import math
from pathlib import Path
import re
import time

URI = b"https://sephonix.com/plugins/sps-ott"
P = C.c_void_p
U = C.c_uint32
F = C.c_float

class Feature(C.Structure):
    _fields_ = [("uri", C.c_char_p), ("data", P)]

class Descriptor(C.Structure):
    pass

Instantiate = C.CFUNCTYPE(P, C.POINTER(Descriptor), C.c_double, C.c_char_p, C.POINTER(C.POINTER(Feature)))
Connect = C.CFUNCTYPE(None, P, U, P)
Activate = C.CFUNCTYPE(None, P)
Run = C.CFUNCTYPE(None, P, U)
Cleanup = C.CFUNCTYPE(None, P)
Extension = C.CFUNCTYPE(P, C.c_char_p)
Descriptor._fields_ = [("uri", C.c_char_p), ("instantiate", Instantiate), ("connect", Connect), ("activate", Activate), ("run", Run), ("deactivate", P), ("cleanup", Cleanup), ("extension", Extension)]

Write = C.CFUNCTYPE(None, P, U, U, U, P)
class UiDescriptor(C.Structure):
    pass
UiInstantiate = C.CFUNCTYPE(P, C.POINTER(UiDescriptor), C.c_char_p, C.c_char_p, Write, P, C.POINTER(P), C.POINTER(C.POINTER(Feature)))
PortEvent = C.CFUNCTYPE(None, P, U, U, U, P)
UiDescriptor._fields_ = [("uri", C.c_char_p), ("instantiate", UiInstantiate), ("cleanup", Cleanup), ("port_event", PortEvent), ("extension", Extension)]
Idle = C.CFUNCTYPE(C.c_int, P)
class IdleInterface(C.Structure):
    _fields_ = [("idle", Idle)]

def bind(lib, name, result, *args):
    fn = getattr(lib, name)
    fn.restype, fn.argtypes = result, args
    return fn

def metadata_check(bundle):
    lib = C.CDLL(ctypes.util.find_library("lilv-0"))
    world = bind(lib, "lilv_world_new", P)()
    try:
        file_uri = (bundle.as_uri() + "/").encode()
        node = bind(lib, "lilv_new_file_uri", P, P, C.c_char_p, C.c_char_p)(world, None, str(bundle).encode() + b"/")
        bind(lib, "lilv_world_load_bundle", None, P, P)(world, node)
        bind(lib, "lilv_node_free", None, P)(node)
        plugins = bind(lib, "lilv_world_get_all_plugins", P, P)(world)
        uri = bind(lib, "lilv_new_uri", P, P, C.c_char_p)(world, URI)
        plugin = bind(lib, "lilv_plugins_get_by_uri", P, P, P)(plugins, uri)
        assert plugin, f"Lilv did not discover {file_uri!r}"
        assert bind(lib, "lilv_plugin_verify", C.c_bool, P)(plugin)
        assert bind(lib, "lilv_plugin_get_num_ports", U, P)(plugin) == 53
        bind(lib, "lilv_node_free", None, P)(uri)
    finally:
        bind(lib, "lilv_world_free", None, P)(world)
    print("LV2: Lilv discovery and 53-port metadata verified", flush=True)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", nargs="?", default="target/bundled/sps-ott.lv2")
    parser.add_argument("--ui", action="store_true")
    parser.add_argument("--seconds", type=float, default=10.0)
    parser.add_argument("--exercise-ui", action="store_true", help="send X11 events to test Amount editing and factory reset")
    args = parser.parse_args()
    bundle = Path(args.bundle).resolve()
    metadata_check(bundle)
    controls = {}
    symbols = {}
    for index, symbol, default in re.findall(r'lv2:index (\d+) ; lv2:symbol "([^"]+)" ; lv2:name "[^"]+" ; lv2:default ([-.\d]+)', (bundle / "sps-ott.ttl").read_text()):
        controls[int(index)] = F(float(default))
        symbols[symbol] = int(index)
    assert len(controls) == 38
    lib = C.CDLL(str(bundle / "sps_ott.so"))
    get = bind(lib, "lv2_descriptor", C.POINTER(Descriptor), U)
    descriptor = get(0)
    assert descriptor and descriptor.contents.uri == URI and not get(1)
    d = descriptor.contents
    features = (C.POINTER(Feature) * 1)()
    handle = d.instantiate(descriptor, 48000.0, str(bundle).encode() + b"/", features)
    assert handle
    count = 256
    left, right, out_l, out_r = [(F * count)() for _ in range(4)]
    meters = [F(-120.0) for _ in range(11)]
    for i, buffer in enumerate([left, right, out_l, out_r]):
        d.connect(handle, i, C.cast(buffer, P))
    for i, value in controls.items():
        d.connect(handle, i, C.cast(C.pointer(value), P))
    for i, value in enumerate(meters):
        d.connect(handle, 42+i, C.cast(C.pointer(value), P))
    try:
        controls[symbols["bypass"]].value = 1.0
        d.activate(handle)
        for i in range(count):
            left[i], right[i] = math.sin(i * 0.1) * 0.1, -math.sin(i * 0.1) * 0.1
        d.run(handle, count)
        assert list(left) == list(out_l) and list(right) == list(out_r), "Bypass differs from input"
        controls[symbols["bypass"]].value = 0.0
        d.activate(handle)
        changed = False
        for block in range(200):
            for i in range(count):
                left[i] = math.sin((block * count + i) * 0.05) * 0.1
                right[i] = -left[i]
            d.run(handle, count)
            assert all(math.isfinite(x) for x in [*out_l, *out_r])
            assert max(abs(a+b) for a, b in zip(out_l, out_r)) < 1e-6
            changed |= list(out_l) != list(left)
        assert changed
        assert all(math.isfinite(m.value) for m in meters)
        d.run(handle, 0)
        print("LV2: exact bypass, audio processing, anti-phase stereo, meters, and zero-size block passed", flush=True)
        if args.ui:
            run_ui(lib, bundle, args.seconds, controls, symbols, meters, d, handle, left, right, count, args.exercise_ui)
    finally:
        d.cleanup(handle)

def run_ui(lib, bundle, seconds, controls, symbols, meters, d, audio_handle, left, right, count, exercise):
    x = C.CDLL(ctypes.util.find_library("X11"))
    bind(x, "XInitThreads", C.c_int)()
    display = bind(x, "XOpenDisplay", P, C.c_char_p)(None)
    assert display, "--ui needs DISPLAY"
    root = bind(x, "XDefaultRootWindow", C.c_ulong, P)(display)
    window = bind(x, "XCreateSimpleWindow", C.c_ulong, P, C.c_ulong, C.c_int, C.c_int, U, U, U, C.c_ulong, C.c_ulong)(display, root, 50, 50, 900, 760, 0, 0, 0)
    bind(x, "XStoreName", C.c_int, P, C.c_ulong, C.c_char_p)(display, window, "SPS OTT — LV2 Smoke Test".encode())
    bind(x, "XMapWindow", C.c_int, P, C.c_ulong)(display, window)
    bind(x, "XFlush", C.c_int, P)(display)
    get = bind(lib, "lv2ui_descriptor", C.POINTER(UiDescriptor), U)
    descriptor = get(0)
    assert descriptor and not get(1)
    u = descriptor.contents
    edits = []
    @Write
    def write(controller, port, size, protocol, data):
        assert size == 4 and protocol == 0 and port in controls
        controls[port].value = C.cast(data, C.POINTER(F)).contents.value
        edits.append((port, controls[port].value))
        print(f"LV2 UI write: {port} = {controls[port].value:.3f}", flush=True)
    parent = Feature(b"http://lv2plug.in/ns/extensions/ui#parent", window)
    idle_feature = Feature(b"http://lv2plug.in/ns/extensions/ui#idleInterface", None)
    features = (C.POINTER(Feature) * 3)(C.pointer(parent), C.pointer(idle_feature), None)
    widget = P()
    handle = u.instantiate(descriptor, URI, str(bundle).encode()+b"/", write, None, C.byref(widget), features)
    assert handle and widget.value
    idle = C.cast(u.extension(b"http://lv2plug.in/ns/extensions/ui#idleInterface"), C.POINTER(IdleInterface)).contents.idle
    print(f"LV2 UI: embedded X11 widget {widget.value} in parent {window}", flush=True)
    try:
        for port, v in controls.items():
            u.port_event(handle, port, 4, 0, C.cast(C.pointer(v), P))
        if exercise:
            assert seconds >= 2.0, "--exercise-ui requires at least two seconds"
        started = time.monotonic()
        until = started + seconds
        actions = [(0.3, 6, 85, 140), (0.4, 4, 85, 140), (0.7, 6, 85, 220), (0.9, 5, 85, 220), (1.2, 6, 820, 32), (1.3, 4, 820, 32), (1.5, 5, 820, 32)]
        frame = 0
        while time.monotonic() < until:
            for _ in range(6):
                for i in range(count):
                    t = (frame+i)/48000.0
                    signal = 0.06*math.sin(2*math.pi*55*t)+0.04*math.sin(2*math.pi*730*t)+0.02*math.sin(2*math.pi*6700*t)
                    left[i], right[i] = signal, signal*0.9
                d.run(audio_handle, count)
                frame += count
            for i, v in enumerate(meters):
                u.port_event(handle, 42+i, 4, 0, C.cast(C.pointer(v), P))
            if exercise and actions and time.monotonic() - started >= actions[0][0]:
                _, kind, px, py = actions.pop(0)
                send_event(x, display, widget.value, root, kind, px, py)
            assert idle(handle) == 0
            time.sleep(0.02)
        if exercise:
            amount_edits = [value for port, value in edits if port == symbols["amount"]]
            assert any(value < 99.0 for value in amount_edits), "Amount drag did not reach the host"
            assert amount_edits[-1] == 100.0, "Factory reset did not restore Amount"
        print(f"LV2 UI: idle loop, live meter events, cleanup; {len(edits)} host writes", flush=True)
    finally:
        u.cleanup(handle)
        bind(x, "XDestroyWindow", C.c_int, P, C.c_ulong)(display, window)
        bind(x, "XCloseDisplay", C.c_int, P)(display)

class XPointerEvent(C.Structure):
    _fields_ = [("type", C.c_int), ("serial", C.c_ulong), ("send_event", C.c_int), ("display", P),
                ("window", C.c_ulong), ("root", C.c_ulong), ("subwindow", C.c_ulong), ("time", C.c_ulong),
                ("x", C.c_int), ("y", C.c_int), ("x_root", C.c_int), ("y_root", C.c_int),
                ("state", U), ("detail", U), ("same_screen", C.c_int)]
class XEvent(C.Union):
    _fields_ = [("pointer", XPointerEvent), ("padding", C.c_long * 24)]

def send_event(xlib, display, window, root, kind, px, py):
    # Motion and button events share their layout through state; button occupies
    # four bytes where motion stores its one-byte is_hint plus padding.
    event = XEvent()
    event.pointer = XPointerEvent(kind, 0, 1, display, window, root, 0, 0, px, py, px, py, 0, 1 if kind in [4, 5] else 0, 1)
    mask = {4: 1 << 2, 5: 1 << 3, 6: 1 << 6}[kind]
    assert bind(xlib, "XSendEvent", C.c_int, P, C.c_ulong, C.c_int, C.c_long, C.POINTER(XEvent))(display, window, 0, mask, C.byref(event))
    bind(xlib, "XFlush", C.c_int, P)(display)

if __name__ == "__main__":
    main()
