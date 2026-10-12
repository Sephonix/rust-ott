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
import os
import json
import subprocess
import tempfile
import struct

URI = None
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
        assert bind(lib, "lilv_plugin_get_num_ports", U, P)(plugin) == PORT_COUNT
        bind(lib, "lilv_node_free", None, P)(uri)
    finally:
        bind(lib, "lilv_world_free", None, P)(world)
    print("LV2: Lilv discovery and port metadata verified", flush=True)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", nargs="?", default=f"target/bundled/{PLUGIN}.lv2")
    parser.add_argument("--ui", action="store_true")
    parser.add_argument("--seconds", type=float, default=10.0)
    parser.add_argument("--exercise-ui", action="store_true", help="send X11 events to test Mix editing and factory load")
    parser.add_argument("--screenshot", help="capture the native editor to this PNG path (requires ImageMagick)")
    parser.add_argument("--exercise-presets", action="store_true", help="test editor save/load in an isolated temporary Documents directory")
    args = parser.parse_args()
    global SCREENSHOT, EXERCISE_PRESETS, PRESET_DIRECTORY
    SCREENSHOT, EXERCISE_PRESETS = args.screenshot, args.exercise_presets
    if args.exercise_presets:
        assert args.ui and args.seconds >= 6, "preset UI test requires --ui --seconds 6"
        temporary = tempfile.TemporaryDirectory(prefix="sps-editor-presets-")
        config = Path(temporary.name)
        documents = config / "Documents"
        (config / "user-dirs.dirs").write_text(f'XDG_DOCUMENTS_DIR="{documents}"\n')
        os.environ["XDG_CONFIG_HOME"] = str(config)
        PRESET_DIRECTORY = documents / "SPS" / PLUGIN.removeprefix("sps-").title()
    else:
        temporary = None
    bundle = Path(args.bundle).resolve()
    metadata_check(bundle)
    controls = {}
    symbols = {}
    for index, symbol, default in re.findall(r'lv2:index (\d+) ; lv2:symbol "([^"]+)" ; lv2:name "[^"]+" ; lv2:default ([-.\d]+)', (bundle / f"{PLUGIN}.ttl").read_text()):
        controls[int(index)] = F(float(default))
        symbols[symbol] = int(index)
    assert len(controls) == CONTROL_COUNT
    lib = C.CDLL(str(bundle / (PLUGIN.replace("-", "_")+".so")))
    get = bind(lib, "lv2_descriptor", C.POINTER(Descriptor), U)
    descriptor = get(0)
    assert descriptor and descriptor.contents.uri == URI and not get(1)
    d = descriptor.contents
    features = (C.POINTER(Feature) * 1)()
    handle = d.instantiate(descriptor, 48000.0, str(bundle).encode() + b"/", features)
    assert handle
    count = 256
    left, right, out_l, out_r = [(F * count)() for _ in range(4)]
    meters = [F(-120.0) for _ in range(2)]
    for i, buffer in enumerate([left, right, out_l, out_r]):
        d.connect(handle, i, C.cast(buffer, P))
    for i, value in controls.items():
        d.connect(handle, i, C.cast(C.pointer(value), P))
    for i, value in enumerate(meters):
        d.connect(handle, 4+CONTROL_COUNT+i, C.cast(C.pointer(value), P))
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
            assert max(abs(x) for x in [*out_l,*out_r]) < 20.0
            changed |= list(out_l) != list(left)
        assert changed
        assert all(math.isfinite(m.value) for m in meters)
        d.run(handle, 0)
        print("LV2: exact bypass, audio processing, finite stereo, meters, and zero-size block passed", flush=True)
        if args.ui:
            run_ui(lib, bundle, args.seconds, controls, symbols, meters, d, handle, left, right, count, args.exercise_ui)
    finally:
        d.cleanup(handle)
        if PLUGIN != "sps-room":
            transport_check(d, descriptor, bundle, controls, symbols)
        if temporary:
            temporary.cleanup()

def transport_check(d, descriptor, bundle, controls, symbols):
    # Build a real urid:map feature and time:Position atom, then verify the
    # rendered repeat uses 60 BPM rather than the 120 BPM fallback.
    MapCallback = C.CFUNCTYPE(U, P, C.c_char_p)
    class UridMap(C.Structure):
        _fields_ = [("handle", P), ("map", MapCallback)]
    urids = {}
    @MapCallback
    def map_uri(_, uri):
        if uri not in urids:
            urids[uri] = len(urids) + 1
        return urids[uri]
    mapper = UridMap(None, map_uri)
    feature = Feature(b"http://lv2plug.in/ns/ext/urid#map", C.cast(C.pointer(mapper), P))
    features = (C.POINTER(Feature) * 2)(C.pointer(feature), None)
    handle = d.instantiate(descriptor, 48000.0, str(bundle).encode()+b"/", features)
    assert handle
    try:
        ttl = (bundle / f"{PLUGIN}.ttl").read_text()
        defaults = {int(i):float(v) for i,v in re.findall(r'lv2:index (\d+) ; lv2:symbol "[^"]+" ; lv2:name "[^"]+" ; lv2:default ([-.\d]+)',ttl)}
        for port,v in controls.items():
            v.value = defaults[port]
            d.connect(handle,port,C.cast(C.pointer(v),P))
        set_control = lambda symbol,value: setattr(controls[symbols[symbol]], "value",value)
        set_control("mix",100);set_control("feedback",0);set_control("low_cut",0);set_control("high_cut",20000)
        set_control("sync_l" if PLUGIN=="sps-delay" else "sync",1)
        set_control("division_l" if PLUGIN=="sps-delay" else "division",3)
        if PLUGIN=="sps-delay":
            set_control("mode",12);set_control("era",2)
        atom = b"http://lv2plug.in/ns/ext/atom#"
        time_uri = b"http://lv2plug.in/ns/ext/time#"
        get = lambda uri: urids[uri]
        body = struct.pack("=II",0,get(time_uri+b"Position")) + struct.pack("=IIII f I",get(time_uri+b"beatsPerMinute"),0,4,get(atom+b"Float"),60.0,0)
        event = struct.pack("=qII",0,len(body),get(atom+b"Object"))+body
        sequence = struct.pack("=IIII",len(event)+8,get(atom+b"Sequence"),0,0)+event
        storage = C.create_string_buffer(sequence)
        d.connect(handle,PORT_COUNT-1,C.cast(storage,P))
        count=256
        buffers=[(F*count)() for _ in range(4)]
        for i,b in enumerate(buffers):d.connect(handle,i,C.cast(b,P))
        d.activate(handle)
        peak=(0,0.0)
        for block in range(110):
            for i in range(count):buffers[0][i] = 1.0 if block==0 and i==0 else 0.0
            if block==1:
                # Subsequent blocks have an empty sequence, preserving last tempo.
                storage[0:4]=struct.pack("=I",8)
            d.run(handle,count)
            for i,value in enumerate(buffers[2]):
                if abs(value)>peak[1]:peak=(block*count+i,abs(value))
        assert peak[0]==24000, f"60 BPM eighth note repeat was at {peak}"
        print("LV2: host tempo atom produced exact 500 ms repeat",flush=True)
    finally:
        d.cleanup(handle)

def run_ui(lib, bundle, seconds, controls, symbols, meters, d, audio_handle, left, right, count, exercise):
    x = C.CDLL(ctypes.util.find_library("X11"))
    bind(x, "XInitThreads", C.c_int)()
    display = bind(x, "XOpenDisplay", P, C.c_char_p)(None)
    assert display, "--ui needs DISPLAY"
    root = bind(x, "XDefaultRootWindow", C.c_ulong, P)(display)
    window = bind(x, "XCreateSimpleWindow", C.c_ulong, P, C.c_ulong, C.c_int, C.c_int, U, U, U, C.c_ulong, C.c_ulong)(display, root, 50, 50, 980, 760, 0, 0, 0)
    bind(x, "XStoreName", C.c_int, P, C.c_ulong, C.c_char_p)(display, window, "SPS Time Effect — LV2 Smoke Test".encode())
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
        # A software OpenGL renderer can spend several seconds compiling its
        # first frame. Wait for actual paint before starting input gestures;
        # otherwise press/move/release can all arrive in one initial egui frame.
        wait_for_paint(x, display, widget.value)
        for port, v in controls.items():
            u.port_event(handle, port, 4, 0, C.cast(C.pointer(v), P))
        if exercise:
            assert seconds >= 2.0, "--exercise-ui requires at least two seconds"
        started = time.monotonic()
        until = started + seconds
        actions = [(0.3, 6, 80, 186), (0.4, 4, 80, 186), (0.7, 6, 150, 186), (0.9, 5, 150, 186), (1.2, 6, 225, 83), (1.3, 4, 225, 83), (1.5, 5, 225, 83)]
        if EXERCISE_PRESETS:
            actions += [(1.8,6,180,115),(1.9,4,180,115),(2.0,5,180,115),(2.2,"type",0,0),(2.5,6,390,115),(2.6,4,390,115),(2.8,5,390,115),(3.1,6,150,186),(3.2,4,150,186),(3.4,5,150,186),(3.8,6,540,83),(3.9,4,540,83),(4.1,5,540,83)]
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
                u.port_event(handle, 4+CONTROL_COUNT+i, 4, 0, C.cast(C.pointer(v), P))
            if exercise and actions and time.monotonic() - started >= actions[0][0]:
                _, kind, px, py = actions.pop(0)
                if kind == "type":
                    subprocess.run(["xdotool","type","--window",str(widget.value),"--clearmodifiers","SPS Validation"],check=True)
                else:
                    send_event(x, display, widget.value, root, kind, px, py)
            assert idle(handle) == 0
            time.sleep(0.02)
        if exercise:
            mix_edits = [value for port, value in edits if port == symbols["mix"]]
            assert any(abs(value-35.0)>1.0 for value in mix_edits), "Mix edit did not reach the host"
            assert any(value == 35.0 for value in mix_edits), "Factory load did not restore Mix"
            if EXERCISE_PRESETS:
                preset=json.loads((PRESET_DIRECTORY / "SPS Validation.json").read_text())
                assert preset["plugin"] == "com.sephonix."+PLUGIN
                assert preset["parameters"]["mix"] == 35.0
                assert mix_edits[-2] != 35.0 and mix_edits[-1] == 35.0, "User preset did not restore the edited Mix"
                print("LV2 UI: saved and loaded Documents/SPS preset",flush=True)
            else:
                assert mix_edits[-1] == 35.0
        if SCREENSHOT:
            subprocess.run(["import","-window",str(widget.value),SCREENSHOT],check=True)
        print(f"LV2 UI: idle loop, live meter events, cleanup; {len(edits)} host writes", flush=True)
    finally:
        u.cleanup(handle)
        bind(x, "XDestroyWindow", C.c_int, P, C.c_ulong)(display, window)
        bind(x, "XCloseDisplay", C.c_int, P)(display)

def wait_for_paint(xlib, display, window):
    get_image = bind(xlib, "XGetImage", P, P, C.c_ulong, C.c_int, C.c_int, U, U, C.c_ulong, C.c_int)
    get_pixel = bind(xlib, "XGetPixel", C.c_ulong, P, C.c_int, C.c_int)
    destroy_image = bind(xlib, "XDestroyImage", C.c_int, P)
    deadline = time.monotonic() + 15.0
    # Read the solid lime SPS badge. Window creation alone does not imply that
    # the first egui frame has rendered; a blank GL framebuffer is not ready.
    while time.monotonic() < deadline:
        image = get_image(display, window, 10, 10, 1, 1, C.c_ulong(-1).value, 2)
        if image:
            pixel = get_pixel(image, 0, 0) & 0xFFFFFF
            destroy_image(image)
            if pixel == 0x12171E:
                print("LV2 UI: first native frame painted", flush=True)
                return
        time.sleep(0.05)
    raise AssertionError("Native editor did not paint within 15 seconds")

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
