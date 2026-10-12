//! SPS {title} LV2 schema and factory preset generation.
use sps_dsp::params::Spec;
use std::{fs, path::Path};

pub fn metadata(
    directory: &Path,
    package: &str,
    title: &str,
    kind: &str,
    specs: &[Spec],
    choices: impl Fn(usize) -> &'static [&'static str],
) -> std::io::Result<()> {
    let uri = format!("https://sephonix.com/plugins/{package}");
    let binary = package.replace('-', "_");
    let count = specs.len();
    fs::create_dir_all(directory)?;
    let prefixes = "@prefix atom: <http://lv2plug.in/ns/ext/atom#> .\n@prefix time: <http://lv2plug.in/ns/ext/time#> .\n@prefix urid: <http://lv2plug.in/ns/ext/urid#> .\n@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n@prefix ui: <http://lv2plug.in/ns/extensions/ui#> .\n@prefix pset: <http://lv2plug.in/ns/ext/presets#> .\n@prefix doap: <http://usefulinc.com/ns/doap#> .\n@prefix units: <http://lv2plug.in/ns/extensions/units#> .\n@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n@prefix foaf: <http://xmlns.com/foaf/0.1/> .\n";
    fs::write(directory.join("manifest.ttl"), format!("{prefixes}\n<{uri}> a lv2:Plugin ;\n    lv2:binary <{binary}.so> ; rdfs:seeAlso <{package}.ttl> .\n<{uri}#ui> a ui:X11UI ;\n    ui:binary <{binary}.so> ; rdfs:seeAlso <{package}.ttl> .\n<{uri}#factory> a pset:Preset ;\n    lv2:appliesTo <{uri}> ; rdfs:label \"SPS {title} — Default\" ; rdfs:seeAlso <factory.ttl> .\n"))?;
    let mut ttl = format!("{prefixes}\n<{uri}> a lv2:Plugin, lv2:{kind}Plugin, doap:Project ;\n    doap:name \"SPS {title}\" ;\n    doap:license <https://www.gnu.org/licenses/gpl-3.0.html> ;\n    doap:maintainer [ foaf:name \"Sephonix\" ; foaf:homepage <https://github.com/Sephonix/sps-dsp> ] ;\n    lv2:minorVersion 0 ; lv2:microVersion 1 ;\n    lv2:optionalFeature lv2:hardRTCapable, urid:map ;\n    ui:ui <{uri}#ui> ;\n    lv2:port\n");
    let mut ports = Vec::new();
    for (i, symbol, name, input) in [
        (0, "in_l", "Input Left", true),
        (1, "in_r", "Input Right", true),
        (2, "out_l", "Output Left", false),
        (3, "out_r", "Output Right", false),
    ] {
        ports.push(format!("    [ a lv2:AudioPort, lv2:{} ; lv2:index {i} ; lv2:symbol \"{symbol}\" ; lv2:name \"{name}\" ]", if input { "InputPort" } else { "OutputPort" }));
    }
    for (i, s) in specs.iter().enumerate() {
        let name = s.name;
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
        let labels = choices(i);
        let discrete = if !labels.is_empty() {
            format!(
                " ; lv2:portProperty lv2:integer, lv2:enumeration ; lv2:scalePoint {}",
                labels
                    .iter()
                    .enumerate()
                    .map(|(i, l)| format!("[ rdfs:label \"{l}\" ; rdf:value {i} ]"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else if s.discrete {
            " ; lv2:portProperty lv2:integer, lv2:toggled".into()
        } else {
            String::new()
        };
        ports.push(format!("    [ a lv2:ControlPort, lv2:InputPort ; lv2:index {} ; lv2:symbol \"{}\" ; lv2:name \"{name}\" ; lv2:default {} ; lv2:minimum {} ; lv2:maximum {} ; lv2:portProperty lv2:connectionOptional{extra}{discrete} ]",4+i,s.id,s.default,s.min,s.max));
    }
    for (i, name) in ["Input", "Output"].iter().enumerate() {
        let maximum = 60;
        ports.push(format!("    [ a lv2:ControlPort, lv2:OutputPort ; lv2:index {} ; lv2:symbol \"meter_{i}\" ; lv2:name \"{name}\" ; lv2:minimum -120 ; lv2:maximum {maximum} ; units:unit units:db ; lv2:portProperty lv2:connectionOptional ]",4+count+i));
    }
    if package != "sps-room" {
        ports.push(format!("    [ a atom:AtomPort, lv2:InputPort ; lv2:index {} ; lv2:symbol \"transport\" ; lv2:name \"Tempo transport\" ; atom:bufferType atom:Sequence ; atom:supports time:Position ; lv2:portProperty lv2:connectionOptional ]", count+6));
    }
    ttl.push_str(&ports.join(",\n"));
    ttl.push_str(" .\n\n");
    ttl.push_str(&format!("<{uri}#ui> a ui:X11UI ; ui:binary <{binary}.so> ;\n    lv2:requiredFeature ui:parent, ui:idleInterface ;\n    lv2:optionalFeature ui:resize ;\n    lv2:extensionData ui:idleInterface ;\n    ui:portNotification\n"));
    let notifications: Vec<_> = (0..2)
        .map(|i| {
            format!(
                "    [ ui:plugin <{uri}> ; ui:portIndex {} ; ui:protocol ui:floatProtocol ]",
                4 + count + i
            )
        })
        .collect();
    ttl.push_str(&notifications.join(",\n"));
    ttl.push_str(" .\n");
    fs::write(directory.join(format!("{package}.ttl")), ttl)?;
    let preset_ports: Vec<_> = specs
        .iter()
        .map(|s| format!("    [ lv2:symbol \"{}\" ; pset:value {} ]", s.id, s.default))
        .collect();
    fs::write(directory.join("factory.ttl"), format!("{prefixes}\n<{uri}#factory> a pset:Preset ; lv2:appliesTo <{uri}> ;\n    rdfs:label \"SPS {title} — Default\" ;\n    lv2:port\n{} .\n",preset_ports.join(",\n")))?;
    Ok(())
}
