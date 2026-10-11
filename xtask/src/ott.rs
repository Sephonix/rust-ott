//! SPS OTT LV2 schema and factory preset generation.
use sps_ott_dsp::{params::*, METER_NAMES};
use std::{fs, path::Path};

const URI: &str = "https://sephonix.com/plugins/sps-ott";

pub fn metadata(directory: &Path) -> std::io::Result<()> {
    fs::create_dir_all(directory)?;
    let prefixes = "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n@prefix ui: <http://lv2plug.in/ns/extensions/ui#> .\n@prefix pset: <http://lv2plug.in/ns/ext/presets#> .\n@prefix doap: <http://usefulinc.com/ns/doap#> .\n@prefix units: <http://lv2plug.in/ns/extensions/units#> .\n@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n@prefix foaf: <http://xmlns.com/foaf/0.1/> .\n";
    fs::write(directory.join("manifest.ttl"), format!("{prefixes}\n<{URI}> a lv2:Plugin ;\n    lv2:binary <sps_ott.so> ; rdfs:seeAlso <sps-ott.ttl> .\n<{URI}#ui> a ui:X11UI ;\n    ui:binary <sps_ott.so> ; rdfs:seeAlso <sps-ott.ttl> .\n<{URI}#factory> a pset:Preset ;\n    lv2:appliesTo <{URI}> ; rdfs:label \"SPS OTT — Aggressive\" ; rdfs:seeAlso <factory.ttl> .\n"))?;
    let mut ttl = format!("{prefixes}\n<{URI}> a lv2:Plugin, lv2:CompressorPlugin, doap:Project ;\n    doap:name \"SPS OTT\" ;\n    doap:license <https://www.gnu.org/licenses/gpl-3.0.html> ;\n    doap:maintainer [ foaf:name \"Sephonix\" ; foaf:homepage <https://github.com/Sephonix/sps-dsp> ] ;\n    lv2:minorVersion 0 ; lv2:microVersion 1 ;\n    lv2:optionalFeature lv2:hardRTCapable ;\n    ui:ui <{URI}#ui> ;\n    lv2:port\n");
    let mut ports = Vec::new();
    for (i, symbol, name, input) in [
        (0, "in_l", "Input Left", true),
        (1, "in_r", "Input Right", true),
        (2, "out_l", "Output Left", false),
        (3, "out_r", "Output Right", false),
    ] {
        ports.push(format!("    [ a lv2:AudioPort, lv2:{} ; lv2:index {i} ; lv2:symbol \"{symbol}\" ; lv2:name \"{name}\" ]", if input { "InputPort" } else { "OutputPort" }));
    }
    for (i, s) in SPECS.iter().enumerate() {
        let name = if i < GLOBAL_COUNT {
            s.name.to_owned()
        } else {
            format!(
                "{} {}",
                BAND_NAMES[(i - GLOBAL_COUNT) / BAND_STRIDE],
                s.name
            )
        };
        let unit = match s.unit {
            "dB" => "db",
            "Hz" => "hz",
            "ms" => "ms",
            "%" => "pc",
            _ => "",
        };
        let extra = if unit.is_empty() {
            String::new()
        } else {
            format!(" ; units:unit units:{unit}")
        };
        let discrete = if i == DETECTION {
            " ; lv2:portProperty lv2:integer, lv2:enumeration ; lv2:scalePoint [ rdfs:label \"RMS\" ; rdf:value 0 ], [ rdfs:label \"Peak\" ; rdf:value 1 ]"
        } else if s.discrete {
            " ; lv2:portProperty lv2:integer, lv2:toggled"
        } else {
            ""
        };
        ports.push(format!("    [ a lv2:ControlPort, lv2:InputPort ; lv2:index {} ; lv2:symbol \"{}\" ; lv2:name \"{name}\" ; lv2:default {} ; lv2:minimum {} ; lv2:maximum {} ; lv2:portProperty lv2:connectionOptional{extra}{discrete} ]",4+i,s.id,s.default,s.min,s.max));
    }
    for (i, name) in METER_NAMES.iter().enumerate() {
        let maximum = if (5..8).contains(&i) { 36 } else { 60 };
        ports.push(format!("    [ a lv2:ControlPort, lv2:OutputPort ; lv2:index {} ; lv2:symbol \"meter_{i}\" ; lv2:name \"{name}\" ; lv2:minimum -120 ; lv2:maximum {maximum} ; units:unit units:db ; lv2:portProperty lv2:connectionOptional ]",4+PARAM_COUNT+i));
    }
    ttl.push_str(&ports.join(",\n"));
    ttl.push_str(" .\n\n");
    ttl.push_str(&format!("<{URI}#ui> a ui:X11UI ; ui:binary <sps_ott.so> ;\n    lv2:requiredFeature ui:parent, ui:idleInterface ;\n    lv2:optionalFeature ui:resize ;\n    lv2:extensionData ui:idleInterface ;\n    ui:portNotification\n"));
    let notifications: Vec<_> = (0..METER_NAMES.len())
        .map(|i| {
            format!(
                "    [ ui:plugin <{URI}> ; ui:portIndex {} ; ui:protocol ui:floatProtocol ]",
                4 + PARAM_COUNT + i
            )
        })
        .collect();
    ttl.push_str(&notifications.join(",\n"));
    ttl.push_str(" .\n");
    fs::write(directory.join("sps-ott.ttl"), ttl)?;
    let preset_ports: Vec<_> = SPECS
        .iter()
        .map(|s| format!("    [ lv2:symbol \"{}\" ; pset:value {} ]", s.id, s.default))
        .collect();
    fs::write(directory.join("factory.ttl"), format!("{prefixes}\n<{URI}#factory> a pset:Preset ; lv2:appliesTo <{URI}> ;\n    rdfs:label \"SPS OTT — Aggressive\" ;\n    lv2:port\n{} .\n",preset_ports.join(",\n")))?;
    Ok(())
}
