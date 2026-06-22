use ferret_core::{obfuscate as obfuscate_core, ObfuscationOptions, ObfuscationResult, Preset};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const DEFAULT_SEED: u64 = 0xF3EE_2026;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmOptions {
    seed: Option<u64>,
    preset: Option<Preset>,
    allow_dynamic_loaders: bool,
}

#[wasm_bindgen(js_name = obfuscate)]
pub fn obfuscate_wasm(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let options = parse_options(options)?;
    let result = obfuscate_inner(source, options).map_err(|error| JsValue::from_str(&error))?;
    let serializer =
        serde_wasm_bindgen::Serializer::new().serialize_large_number_types_as_bigints(true);

    result.serialize(&serializer).map_err(js_error)
}

fn parse_options(value: JsValue) -> Result<WasmOptions, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(WasmOptions::default());
    }
    serde_wasm_bindgen::from_value(value).map_err(js_error)
}

fn obfuscate_inner(source: &str, options: WasmOptions) -> Result<ObfuscationResult, String> {
    let options = ObfuscationOptions {
        seed: options.seed.unwrap_or(DEFAULT_SEED),
        preset: options.preset.unwrap_or_default(),
        allow_dynamic_loaders: options.allow_dynamic_loaders,
    };
    obfuscate_core(source, options).map_err(|error| error.to_string())
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obfuscates_with_default_options() {
        let result = obfuscate_inner("local value = 41 + 1\nreturn value", WasmOptions::default())
            .expect("obfuscation should succeed");

        assert!(!result.code.is_empty());
        assert_eq!(result.metadata.seed, DEFAULT_SEED);
        assert_eq!(result.metadata.preset, Preset::Strong);
        assert!(result.metadata.vm_only);
    }

    #[test]
    fn accepts_explicit_options() {
        let result = obfuscate_inner(
            "return 1",
            WasmOptions {
                seed: Some(7),
                preset: Some(Preset::Balanced),
                allow_dynamic_loaders: false,
            },
        )
        .expect("obfuscation should succeed");

        assert_eq!(result.metadata.seed, 7);
        assert_eq!(result.metadata.preset, Preset::Balanced);
    }
}
