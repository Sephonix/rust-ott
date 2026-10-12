//! Shared LV2 transport ABI boundary. No plugin identities or processing policy.
#![deny(unsafe_op_in_unsafe_fn)]
use std::ffi::{c_char, c_void, CStr};
#[repr(C)]
pub struct Feature {
    pub uri: *const c_char,
    pub data: *mut c_void,
}
#[repr(C)]
struct Map {
    handle: *mut c_void,
    map: unsafe extern "C" fn(*mut c_void, *const c_char) -> u32,
}
#[derive(Default, Clone, Copy)]
pub struct TransportIds {
    sequence: u32,
    object: u32,
    blank: u32,
    float: u32,
    double: u32,
    position: u32,
    bpm: u32,
}
impl TransportIds {
    /// # Safety
    /// Features are a host-owned NULL-terminated array with valid URI strings
    /// and a valid LV2_URID_Map for urid:map. Used only during instantiate.
    pub unsafe fn from_features(features: *const *const Feature) -> Self {
        let mut ids = Self::default();
        if features.is_null() {
            return ids;
        }
        let mut cursor = features;
        unsafe {
            while !(*cursor).is_null() {
                let f = &**cursor;
                if !f.uri.is_null()
                    && CStr::from_ptr(f.uri).to_bytes() == b"http://lv2plug.in/ns/ext/urid#map"
                    && !f.data.is_null()
                {
                    let map = &*f.data.cast::<Map>();
                    let get = |uri: &CStr| (map.map)(map.handle, uri.as_ptr());
                    ids = Self {
                        sequence: get(c"http://lv2plug.in/ns/ext/atom#Sequence"),
                        object: get(c"http://lv2plug.in/ns/ext/atom#Object"),
                        blank: get(c"http://lv2plug.in/ns/ext/atom#Blank"),
                        float: get(c"http://lv2plug.in/ns/ext/atom#Float"),
                        double: get(c"http://lv2plug.in/ns/ext/atom#Double"),
                        position: get(c"http://lv2plug.in/ns/ext/time#Position"),
                        bpm: get(c"http://lv2plug.in/ns/ext/time#beatsPerMinute"),
                    };
                    break;
                }
                cursor = cursor.add(1);
            }
        }
        ids
    }
    /// # Safety
    /// A non-NULL port points to a valid LV2_Atom_Sequence whose allocation covers
    /// its declared body size plus header, as required by the LV2 atom contract.
    /// That buffer remains readable for the entire returned iterator lifetime.
    /// Header/field reads are unaligned; internal sizes are checked by the parser.
    pub unsafe fn events<'a>(&self, port: *const c_void) -> TransportEvents<'a> {
        let bytes = if port.is_null() || self.sequence == 0 {
            &[][..]
        } else {
            let p = port.cast::<u8>();
            let size = unsafe { p.cast::<u32>().read_unaligned() } as usize;
            if !(8..=65536).contains(&size) {
                &[][..]
            } else {
                unsafe { std::slice::from_raw_parts(p, size + 8) }
            }
        };
        TransportEvents::new(bytes, *self)
    }
}
fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_ne_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}
/// Validated iterator retaining sample-frame timestamps for tempo changes.
pub struct TransportEvents<'a> {
    bytes: &'a [u8],
    ids: TransportIds,
    cursor: usize,
}
impl<'a> TransportEvents<'a> {
    fn new(bytes: &'a [u8], ids: TransportIds) -> Self {
        let valid = u32_at(bytes, 4) == Some(ids.sequence)
            && ids.sequence != 0
            && u32_at(bytes, 8) == Some(0);
        Self {
            bytes: if valid { bytes } else { &[] },
            ids,
            cursor: 16,
        }
    }
}
impl Iterator for TransportEvents<'_> {
    type Item = (u32, f32);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let start = self.cursor;
            let size = u32_at(self.bytes, start + 8)? as usize;
            let atom_type = u32_at(self.bytes, start + 12)?;
            let end = start.checked_add(16)?.checked_add(size)?;
            if end > self.bytes.len() {
                return None;
            }
            self.cursor = (end + 7) & !7;
            if atom_type != self.ids.object && atom_type != self.ids.blank {
                continue;
            }
            let body = self.bytes.get(start + 16..end)?;
            if u32_at(body, 4) != Some(self.ids.position) {
                continue;
            }
            let frame = i64::from_ne_bytes(self.bytes.get(start..start + 8)?.try_into().ok()?);
            if !(0..=u32::MAX as i64).contains(&frame) {
                continue;
            }
            let mut p = 8;
            while p + 16 <= body.len() {
                let key = u32_at(body, p)?;
                let len = u32_at(body, p + 8)? as usize;
                let typ = u32_at(body, p + 12)?;
                let value = body.get(p + 16..p + 16 + len)?;
                p = (p + 16 + len + 7) & !7;
                if key != self.ids.bpm {
                    continue;
                }
                let bpm = if typ == self.ids.float && len == 4 {
                    f32::from_ne_bytes(value.try_into().ok()?)
                } else if typ == self.ids.double && len == 8 {
                    f64::from_ne_bytes(value.try_into().ok()?) as f32
                } else {
                    continue;
                };
                if bpm.is_finite() && (1.0..=999.0).contains(&bpm) {
                    return Some((frame as u32, bpm));
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tempo_and_malformed_sequences() {
        let ids = TransportIds {
            sequence: 1,
            object: 2,
            blank: 3,
            float: 4,
            double: 5,
            position: 6,
            bpm: 7,
        };
        let mut bytes = Vec::new();
        for value in [
            56u32,
            1,
            0,
            0,
            12,
            0,
            32,
            2,
            0,
            6,
            7,
            0,
            4,
            4,
            120.0f32.to_bits(),
            0,
        ] {
            bytes.extend(value.to_ne_bytes());
        }
        assert_eq!(
            TransportEvents::new(&bytes, ids).collect::<Vec<_>>(),
            [(12, 120.0)]
        );
        for n in 0..bytes.len() {
            assert_eq!(TransportEvents::new(&bytes[..n], ids).next(), None);
        }
        bytes[48..52].copy_from_slice(&u32::MAX.to_ne_bytes());
        assert_eq!(TransportEvents::new(&bytes, ids).next(), None);
    }
}
